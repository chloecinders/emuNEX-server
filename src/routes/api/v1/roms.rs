use std::path::Path;

use rocket::{
    delete,
    form::{Form, FromForm},
    fs::TempFile,
    get, post, put,
    serde::json::Json,
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
    utils::s3::{compute_md5, upload_object},
};

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct V1RomListResponse {
    pub id: String,
    pub title: String,
    pub image_path: String,
    pub console: String,
    pub category: Option<String>,
    pub region: Option<String>,
    pub release_year: Option<i32>,
    pub serial: Option<String>,
}
impl V1ApiResponseTrait for Vec<V1RomListResponse> {}

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
        "SELECT id, title, image_path, console, category, region, release_year, serial FROM roms
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

#[derive(Serialize)]
pub struct V1SearchOverviewResponse(pub std::collections::HashMap<String, Vec<V1RomListResponse>>);
impl V1ApiResponseTrait for V1SearchOverviewResponse {}

#[get("/api/v1/search/overview")]
pub async fn get_search_overview(
    _user: AuthenticatedUser,
) -> V1ApiResponseType<V1SearchOverviewResponse> {
    let mut overview = std::collections::HashMap::new();

    let most_played = sqlx::query_as!(
        V1RomListResponse,
        r#"SELECT r.id, r.title, r.image_path, r.console, r.category, r.region, r.release_year, r.serial
         FROM roms r
         LEFT JOIN (
             SELECT rom_id, SUM(play_count) as total_play_count
             FROM user_roms
             GROUP BY rom_id
         ) ur ON r.id = ur.rom_id
         ORDER BY COALESCE(ur.total_play_count, 0) DESC, r.title ASC
         LIMIT 50"#
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to fetch most played: {:?}", e);
        V1ApiError::InternalError
    })?;
    if !most_played.is_empty() {
        overview.insert("Most Played".to_string(), most_played);
    }

    let recently_added = sqlx::query_as!(
        V1RomListResponse,
        "SELECT id, title, image_path, console, category, region, release_year, serial FROM roms
         ORDER BY created_at DESC NULLS LAST
         LIMIT 50"
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to fetch recently added: {:?}", e);
        V1ApiError::InternalError
    })?;
    if !recently_added.is_empty() {
        overview.insert("Recently Added".to_string(), recently_added);
    }

    let categories = sqlx::query!(
        r#"SELECT DISTINCT category as "category!" FROM roms WHERE category IS NOT NULL"#
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to fetch distinct categories: {:?}", e);
        V1ApiError::InternalError
    })?;

    for record in categories {
        let category = record.category;
        let cat_roms = sqlx::query_as!(
            V1RomListResponse,
            r#"SELECT id, title, image_path, console, category, region, release_year, serial FROM roms
                 WHERE category = $1
                 ORDER BY title ASC
                 LIMIT 50"#,
            category
        )
        .fetch_all(&*SQL)
        .await
        .map_err(|e| {
            error!("Failed to fetch roms for category {}: {:?}", category, e);
            V1ApiError::InternalError
        })?;

        if !cat_roms.is_empty() {
            overview.insert(category, cat_roms);
        }
    }

    Ok(V1ApiResponse(V1SearchOverviewResponse(overview)))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct V1RomFullResponse {
    pub id: String,
    pub title: String,
    pub console: String,
    pub region: Option<String>,
    pub category: String,
    pub serial: Option<String>,
    pub rom_path: String,
    pub image_path: String,
    pub file_extension: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub md5_hash: Option<String>,
    pub release_year: Option<i32>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
impl V1ApiResponseTrait for V1RomFullResponse {}

#[get("/api/v1/roms/<id>")]
pub async fn get_rom_single(
    id: &str,
    _user: AuthenticatedUser,
) -> V1ApiResponseType<V1RomFullResponse> {
    let rom = sqlx::query_as!(V1RomFullResponse, "SELECT id, title, console, region, category, serial, rom_path, image_path, file_extension, file_size_bytes, md5_hash, release_year, created_at FROM roms WHERE id = $1", id)
        .fetch_optional(&*SQL)
        .await
        .map_err(|e| {
            error!("Failed to fetch rom id {}: {:?}", id, e);
            V1ApiError::InternalError
        })?
        .ok_or(V1ApiError::NotFound)?;

    Ok(V1ApiResponse(rom))
}

#[get("/api/v1/roms/search?<query>&<category>&<console>&<offset>&<limit>")]
pub async fn search_roms(
    query: String,
    category: Option<String>,
    console: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
    _user: AuthenticatedUser,
) -> V1ApiResponseType<Vec<V1RomListResponse>> {
    let final_limit = limit.unwrap_or(50).min(50);
    let final_offset = offset.unwrap_or(0);

    let results = sqlx::query_as!(
        V1RomListResponse,
        r#"SELECT id, title, image_path, console, category, region, release_year, serial FROM roms
         WHERE (
             title ILIKE '%' || $1 || '%'
             OR serial ILIKE $1 || '%'
         )
         AND (category = $2 OR $2 IS NULL)
         AND (console = $3 OR $3 IS NULL)
         ORDER BY
             CASE WHEN serial ILIKE $1 THEN 0 ELSE 1 END,
             title ASC
         LIMIT $4 OFFSET $5"#,
        query,
        category,
        console,
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

#[derive(FromForm)]
pub struct V1RomUpload<'r> {
    title: String,
    console: String,
    category: String,
    region: Option<String>,
    release_year: Option<i32>,
    serial: Option<String>,
    md5_hash: Option<String>,
    rom_file: TempFile<'r>,
    image_file: TempFile<'r>,
}

#[post("/api/v1/roms/upload", format = "multipart/form-data", data = "<data>")]
pub async fn upload_rom(
    data: Form<V1RomUpload<'_>>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<String> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::NotAuthorized);
    }

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

    let rom_bytes = tokio::fs::read(data.rom_file.path().unwrap())
        .await
        .map_err(|e| {
            error!("Failed to read rom file from temp storage: {:?}", e);
            V1ApiError::InternalError
        })?;

    let rom_md5 = compute_md5(&rom_bytes);

    if let Some(provided_md5) = data.md5_hash.as_deref().filter(|s| !s.is_empty()) {
        if provided_md5.to_lowercase() != rom_md5.to_lowercase() {
            error!(
                "MD5 mismatch for ROM upload: expected {}, got {}",
                provided_md5, rom_md5
            );
            return Err(V1ApiError::BadRequest);
        }
    }

    let rom_path = format!("/roms/{}/{}.{}", data.console, rom_md5, rom_ext);

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

    let img_md5 = compute_md5(&img_bytes);
    let img_path = format!("/covers/{}/{}.{}", data.console, img_md5, img_ext);

    upload_object(&img_path, &img_bytes).await.map_err(|e| {
        error!("Failed to upload image to '{}': {:?}", img_path, e);
        V1ApiError::InternalError
    })?;

    let rom_id = crate::utils::snowflake::next_id();

    sqlx::query!(
        "INSERT INTO roms (id, title, console, category, region, serial, release_year, rom_path, image_path, file_extension, md5_hash, file_size_bytes)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
         ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            console = EXCLUDED.console,
            category = EXCLUDED.category,
            region = EXCLUDED.region,
            serial = EXCLUDED.serial,
            release_year = EXCLUDED.release_year,
            rom_path = EXCLUDED.rom_path,
            image_path = EXCLUDED.image_path,
            file_extension = EXCLUDED.file_extension,
            md5_hash = EXCLUDED.md5_hash,
            file_size_bytes = EXCLUDED.file_size_bytes",
        rom_id.to_string(),
        data.title,
        data.console,
        data.category,
        data.region,
        data.serial,
        data.release_year,
        rom_path,
        img_path,
        rom_ext,
        rom_md5,
        rom_bytes.len() as i64
    )
    .execute(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to upsert rom into database: {:?}", e);
        V1ApiError::InternalError
    })?;

    let user_rom_id = crate::utils::snowflake::next_id();

    sqlx::query!(
        "INSERT INTO user_roms (id, user_id, rom_id, play_count, last_played, is_favorite)
         VALUES ($1, $2, $3, 0, NULL, FALSE)
         ON CONFLICT (user_id, rom_id) DO NOTHING",
        user_rom_id,
        user.id.value(),
        rom_id.to_string()
    )
    .execute(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to insert user_roms: {:?}", e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(rom_id.to_string()))
}

#[derive(serde::Deserialize)]
pub struct V1RomUpdateRequest {
    pub title: String,
    pub console: String,
    pub category: String,
    pub region: Option<String>,
    pub release_year: Option<i32>,
    pub serial: Option<String>,
}

#[put("/api/v1/roms/<id>", format = "json", data = "<data>")]
pub async fn update_rom(
    id: String,
    data: Json<V1RomUpdateRequest>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<String> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::NotAuthorized);
    }

    sqlx::query!(
        "UPDATE roms SET title = $1, console = $2, category = $3, region = $4, release_year = $5, serial = $6 WHERE id = $7",
        data.title,
        data.console,
        data.category,
        data.region,
        data.release_year,
        data.serial,
        id
    )
    .execute(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to update rom id {}: {:?}", id, e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(id))
}

#[derive(FromForm)]
pub struct V1FileUpdate<'r> {
    file: TempFile<'r>,
}

#[post(
    "/api/v1/roms/<id>/file",
    format = "multipart/form-data",
    data = "<data>"
)]
pub async fn update_rom_file(
    id: String,
    data: Form<V1FileUpdate<'_>>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<()> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::NotAuthorized);
    }

    let rom = sqlx::query!("SELECT rom_path FROM roms WHERE id = $1", id)
        .fetch_optional(&*SQL)
        .await
        .map_err(|e| {
            error!("{:?}", e);
            V1ApiError::InternalError
        })?
        .ok_or(V1ApiError::NotFound)?;

    let rom_bytes = tokio::fs::read(data.file.path().unwrap())
        .await
        .map_err(|e| {
            error!("{:?}", e);
            V1ApiError::InternalError
        })?;

    let new_md5 = compute_md5(&rom_bytes);

    upload_object(&rom.rom_path, &rom_bytes)
        .await
        .map_err(|e| {
            error!("Failed to upload rom: {:?}", e);
            V1ApiError::InternalError
        })?;

    sqlx::query!(
        "UPDATE roms SET md5_hash = $1, file_size_bytes = $2 WHERE id = $3",
        new_md5,
        rom_bytes.len() as i64,
        id
    )
    .execute(&*SQL)
    .await
    .map_err(|e| {
        error!("{:?}", e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(()))
}

#[derive(FromForm)]
pub struct V1ImageUpdate<'r> {
    image: TempFile<'r>,
}

#[post(
    "/api/v1/roms/<id>/image",
    format = "multipart/form-data",
    data = "<data>"
)]
pub async fn update_rom_image(
    id: String,
    data: Form<V1ImageUpdate<'_>>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<()> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::NotAuthorized);
    }

    let rom = sqlx::query!("SELECT image_path FROM roms WHERE id = $1", id)
        .fetch_optional(&*SQL)
        .await
        .map_err(|e| {
            error!("{:?}", e);
            V1ApiError::InternalError
        })?
        .ok_or(V1ApiError::NotFound)?;

    let img_bytes = tokio::fs::read(data.image.path().unwrap())
        .await
        .map_err(|e| {
            error!("{:?}", e);
            V1ApiError::InternalError
        })?;

    upload_object(&rom.image_path, &img_bytes)
        .await
        .map_err(|e| {
            error!("Failed to upload image: {:?}", e);
            V1ApiError::InternalError
        })?;

    Ok(V1ApiResponse(()))
}

#[delete("/api/v1/roms/<id>")]
pub async fn delete_rom(id: String, user: AuthenticatedUser) -> V1ApiResponseType<()> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::NotAuthorized);
    }

    let rom = sqlx::query!("SELECT rom_path, image_path FROM roms WHERE id = $1", id)
        .fetch_optional(&*SQL)
        .await
        .map_err(|e| {
            error!("Failed to fetch rom for deletion: {:?}", e);
            V1ApiError::InternalError
        })?
        .ok_or(V1ApiError::NotFound)?;

    let _ = crate::utils::s3::delete_object(&rom.rom_path).await;
    let _ = crate::utils::s3::delete_object(&rom.image_path).await;

    sqlx::query!("DELETE FROM roms WHERE id = $1", id)
        .execute(&*SQL)
        .await
        .map_err(|e| {
            error!("Failed to delete rom id {}: {:?}", id, e);
            V1ApiError::InternalError
        })?;

    Ok(V1ApiResponse(()))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct V1CategoryResponse {
    pub name: String,
}
impl V1ApiResponseTrait for Vec<V1CategoryResponse> {}

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
pub async fn start_game(id: String, user: AuthenticatedUser) -> V1ApiResponseType<()> {
    let user_rom_id = crate::utils::snowflake::next_id();

    sqlx::query!(
        "INSERT INTO user_roms (id, user_id, rom_id, play_count, last_played, is_favorite)
         VALUES ($1, $2, $3, 1, NOW(), FALSE)
         ON CONFLICT (user_id, rom_id) DO UPDATE SET
            play_count = user_roms.play_count + 1,
            last_played = EXCLUDED.last_played",
        user_rom_id,
        user.id.value(),
        id
    )
    .execute(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to update play stats for id {}: {:?}", id, e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(()))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct V1UserLibraryResponse {
    pub id: String,
    pub rom_id: String,
    pub title: String,
    pub image_path: String,
    pub console: String,
    pub region: Option<String>,
    pub play_count: i32,
    pub last_played: Option<chrono::DateTime<chrono::Utc>>,
}
impl V1ApiResponseTrait for Vec<V1UserLibraryResponse> {}

#[get("/api/v1/library")]
pub async fn get_user_library(
    user: AuthenticatedUser,
) -> V1ApiResponseType<Vec<V1UserLibraryResponse>> {
    let rows = sqlx::query!(
        r#"SELECT ur.id as "ur_id!", ur.rom_id, r.title, r.image_path, r.console, ur.play_count, ur.last_played, r.region
         FROM user_roms ur
         INNER JOIN roms r ON ur.rom_id = r.id
         WHERE ur.user_id = $1
         ORDER BY ur.last_played DESC NULLS LAST"#,
        user.id.value()
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to fetch user library: {:?}", e);
        V1ApiError::InternalError
    })?;

    let library = rows
        .into_iter()
        .map(|r| V1UserLibraryResponse {
            id: r.rom_id.clone(),
            region: r.region,
            rom_id: r.rom_id,
            title: r.title,
            image_path: r.image_path,
            console: r.console,
            play_count: r.play_count,
            last_played: r.last_played,
        })
        .collect();

    Ok(V1ApiResponse(library))
}
