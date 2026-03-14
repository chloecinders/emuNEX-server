use std::path::Path;

use rocket::{
    form::{Form, FromForm},
    fs::TempFile,
    get, post,
};
use serde::Serialize;
use tracing::error;

use crate::{
    SQL,
    routes::api::{
        V1ApiError, V1ApiResponse, V1ApiResponseType,
        api_response::V1ApiResponseTrait,
        v1::guards::{AuthenticatedUser, UserRole},
    },
    utils::s3::upload_object,
};

#[derive(FromForm)]
pub struct RomUpload<'r> {
    title: String,
    console: String,
    category: String,
    region: Option<String>,
    release_year: Option<i32>,
    rom_file: TempFile<'r>,
    image_file: TempFile<'r>,
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct V1RomListResponse {
    pub id: i32,
    pub title: String,
    pub image_path: String,
    pub console: String,
}
impl V1ApiResponseTrait for Vec<V1RomListResponse> {}

#[derive(Serialize, sqlx::FromRow)]
pub struct V1RomFullResponse {
    pub id: i32,
    pub title: String,
    pub console: String,
    pub region: Option<String>,
    pub category: String,
    pub rom_path: String,
    pub image_path: String,
    pub file_extension: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub md5_hash: Option<String>,
    pub release_year: Option<i32>,
    pub is_favorite: Option<bool>,
    pub play_count: Option<i32>,
    pub last_played: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
impl V1ApiResponseTrait for V1RomFullResponse {}

#[derive(Serialize, sqlx::FromRow)]
pub struct V1CategoryResponse {
    pub name: String,
}
impl V1ApiResponseTrait for Vec<V1CategoryResponse> {}

#[derive(Serialize, sqlx::FromRow)]
pub struct V1UserLibraryResponse {
    pub id: i32,
    pub title: String,
    pub image_path: String,
    pub console: String,
    pub play_count: i32,
    pub last_played: Option<chrono::DateTime<chrono::Utc>>,
}
impl V1ApiResponseTrait for Vec<V1UserLibraryResponse> {}

#[get("/api/v1/roms/list?<category>&<console>&<offset>&<limit>")]
pub async fn get_rom_list(
    category: Option<String>,
    console: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
    _user: AuthenticatedUser,
) -> V1ApiResponseType<Vec<V1RomListResponse>> {
    let final_limit = limit.unwrap_or(50).min(50);
    let final_offset = offset.unwrap_or(0);

    let roms = sqlx::query_as!(
        V1RomListResponse,
        "SELECT id, title, image_path, console FROM roms
         WHERE (category = $1 OR $1 IS NULL)
         AND (console = $2 OR $2 IS NULL)
         ORDER BY title ASC
         LIMIT $3 OFFSET $4",
        category,
        console,
        final_limit,
        final_offset
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to fetch rom list: {:?}", e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(roms))
}

#[get("/api/v1/roms/<id>")]
pub async fn get_rom_single(
    id: i32,
    _user: AuthenticatedUser,
) -> V1ApiResponseType<V1RomFullResponse> {
    let rom = sqlx::query_as!(V1RomFullResponse, "SELECT * FROM roms WHERE id = $1", id)
        .fetch_optional(&*SQL)
        .await
        .map_err(|e| {
            error!("Failed to fetch rom id {}: {:?}", id, e);
            V1ApiError::InternalError
        })?
        .ok_or(V1ApiError::NotFound)?;

    Ok(V1ApiResponse(rom))
}

#[get("/api/v1/roms/search?<query>&<offset>&<limit>")]
pub async fn search_roms(
    query: String,
    offset: Option<i64>,
    limit: Option<i64>,
    _user: AuthenticatedUser,
) -> V1ApiResponseType<Vec<V1RomListResponse>> {
    let final_limit = limit.unwrap_or(50).min(50);
    let final_offset = offset.unwrap_or(0);
    let search_pattern = format!("%{}%", query);

    let results = sqlx::query_as!(
        V1RomListResponse,
        "SELECT id, title, image_path, console FROM roms
         WHERE title ILIKE $1
         ORDER BY title ASC
         LIMIT $2 OFFSET $3",
        search_pattern,
        final_limit,
        final_offset
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to search roms with query '{}': {:?}", query, e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(results))
}

#[post("/api/v1/roms/upload", format = "multipart/form-data", data = "<data>")]
pub async fn upload_rom(
    data: Form<RomUpload<'_>>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<i32> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::NotAuthorized);
    }

    let sanitize = |s: &str| {
        s.replace(" ", "_")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
            .collect::<String>()
    };

    let safe_title = sanitize(&data.title);

    let rom_filename = data
        .rom_file
        .raw_name()
        .map(|n| n.dangerous_unsafe_unsanitized_raw())
        .unwrap_or("file.bin".into());

    let rom_ext = Path::new(rom_filename.as_str())
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("bin");

    let img_filename = data
        .image_file
        .raw_name()
        .map(|n| n.dangerous_unsafe_unsanitized_raw())
        .unwrap_or("cover.jpg".into());

    let img_ext = Path::new(img_filename.as_str())
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("jpg");

    let rom_path = format!("/roms/{}/{}.{}", data.console, safe_title, rom_ext);

    let img_path = format!("/covers/{}/{}.{}", data.console, safe_title, img_ext);

    let rom_bytes = tokio::fs::read(data.rom_file.path().unwrap())
        .await
        .map_err(|e| {
            error!("Failed to read rom file from temp storage: {:?}", e);
            V1ApiError::InternalError
        })?;

    upload_object(&rom_path, &rom_bytes).await.map_err(|e| {
        error!("Failed to upload rom to '{}': {:?}", rom_path, e);
        V1ApiError::InternalError
    })?;

    let img_bytes = tokio::fs::read(data.image_file.path().unwrap())
        .await
        .map_err(|e| {
            error!("Failed to read image file from temp storage: {:?}", e);
            V1ApiError::InternalError
        })?;

    upload_object(&img_path, &img_bytes).await.map_err(|e| {
        error!("Failed to upload image to '{}': {:?}", img_path, e);
        V1ApiError::InternalError
    })?;

    let rec = sqlx::query!(
        "INSERT INTO roms (title, console, category, region, release_year, rom_path, image_path, file_extension)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
        data.title, data.console, data.category, data.region, data.release_year, rom_path, img_path, rom_ext
    )
    .fetch_one(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to insert rom into database: {:?}", e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(rec.id))
}

#[get("/api/v1/roms/categories")]
pub async fn get_categories(
    _user: AuthenticatedUser,
) -> V1ApiResponseType<Vec<V1CategoryResponse>> {
    let categories = sqlx::query_as!(
        V1CategoryResponse,
        "SELECT DISTINCT category as name FROM roms WHERE category IS NOT NULL ORDER BY category ASC"
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to fetch categories: {:?}", e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(categories))
}

#[post("/api/v1/roms/<id>/start")]
pub async fn start_game(id: i32, user: AuthenticatedUser) -> V1ApiResponseType<()> {
    sqlx::query!(
        "INSERT INTO user_roms (user_id, rom_id, play_count, last_played)
         VALUES ($1, $2, 1, NOW())
         ON CONFLICT (user_id, rom_id)
         DO UPDATE SET
            play_count = user_roms.play_count + 1,
            last_played = EXCLUDED.last_played",
        user.id,
        id
    )
    .execute(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to update user_roms for id {}: {:?}", id, e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(()))
}

#[get("/api/v1/library")]
pub async fn get_user_library(
    user: AuthenticatedUser,
) -> V1ApiResponseType<Vec<V1UserLibraryResponse>> {
    let library = sqlx::query_as!(
        V1UserLibraryResponse,
        "SELECT r.id, r.title, r.image_path, r.console, ur.play_count, ur.last_played
         FROM roms r
         INNER JOIN user_roms ur ON r.id = ur.rom_id
         WHERE ur.user_id = $1
         ORDER BY ur.last_played DESC NULLS LAST",
        user.id
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to fetch user library: {:?}", e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(library))
}
