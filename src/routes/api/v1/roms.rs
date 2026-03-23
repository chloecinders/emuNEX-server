use std::{
    io::{Cursor, Read},
    path::Path,
};

use image::ImageFormat;
use rocket::{
    delete,
    form::{Form, FromForm},
    fs::TempFile,
    get, post, put,
    serde::json::Json,
};
use serde::Serialize;
use sqlx::types::chrono;
use tracing::{error, info};

use crate::{
    SQL,
    routes::api::{
        V1ApiError, V1ApiResponse, V1ApiResponseType,
        api_response::V1ApiResponseTrait,
        v1::guards::{AuthenticatedUser, UserRole},
    },
    utils::s3::{compute_md5, delete_object, upload_object},
};

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct V1RomListResponse {
    pub id: String,
    pub title: String,
    pub realname: Option<String>,
    pub image_path: String,
    pub console: String,
    pub category: Option<String>,
    pub region: Option<String>,
    pub release_year: Option<i32>,
    pub serial: Option<String>,
    pub languages: Option<String>,
    pub versions_count: i64,
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
        r#"SELECT id, title, realname, '/covers_small/' || image_hash || '.webp' as "image_path!", console, category, region, release_year, serial, languages, "versions_count!" FROM (
             SELECT DISTINCT ON (console, title) id, title, realname, image_hash, console, category, region, release_year, serial, languages,
             (SELECT COUNT(*) FROM roms r2 WHERE r2.title = roms.title AND r2.console = roms.console) as "versions_count!"
             FROM roms
             WHERE (category = $1 OR $1 IS NULL)
             AND (console = $2 OR $2 IS NULL)
             ORDER BY console, title, region NULLS LAST, id DESC
         ) sub
         ORDER BY title ASC
         LIMIT $3 OFFSET $4"#,
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
        r#"SELECT id, title, realname, '/covers_small/' || image_hash || '.webp' as "image_path!", console, category, region, release_year, serial, languages, "versions_count!" FROM (
             SELECT DISTINCT ON (r.console, r.title) r.id, r.title, r.realname, r.image_hash, r.console, r.category, r.region, r.release_year, r.serial, r.languages,
             (SELECT COUNT(*) FROM roms r2 WHERE r2.title = r.title AND r2.console = r.console) as "versions_count!",
             COALESCE(ur.total_play_count, 0) as total_play_count
             FROM roms r
             LEFT JOIN (
                 SELECT rom_id, SUM(play_count) as total_play_count
                 FROM user_roms
                 GROUP BY rom_id
             ) ur ON r.id = ur.rom_id
             ORDER BY r.console, r.title, r.region NULLS LAST, r.id DESC
         ) sub
         ORDER BY total_play_count DESC, title ASC
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
        r#"SELECT id, title, realname, '/covers_small/' || image_hash || '.webp' as "image_path!", console, category, region, release_year, serial, languages, "versions_count!" FROM (
             SELECT DISTINCT ON (console, title) id, title, realname, image_hash, console, category, region, release_year, serial, languages,
             (SELECT COUNT(*) FROM roms r2 WHERE r2.title = roms.title AND r2.console = roms.console) as "versions_count!",
             created_at
             FROM roms
             ORDER BY console, title, region NULLS LAST, id DESC
         ) sub
         ORDER BY created_at DESC NULLS LAST
         LIMIT 50"#
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
            r#"SELECT id, title, realname, '/covers_small/' || image_hash || '.webp' as "image_path!", console, category, region, release_year, serial, languages, "versions_count!" FROM (
                 SELECT DISTINCT ON (console, title) id, title, realname, image_hash, console, category, region, release_year, serial, languages,
                 (SELECT COUNT(*) FROM roms r2 WHERE r2.title = roms.title AND r2.console = roms.console) as "versions_count!"
                 FROM roms
                 WHERE category = $1
                 ORDER BY console, title, region NULLS LAST, id DESC
             ) sub
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
    pub realname: Option<String>,
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
    pub languages: Option<String>,
    pub versions_count: i64,
}
impl V1ApiResponseTrait for V1RomFullResponse {}

#[get("/api/v1/roms/<id>")]
pub async fn get_rom_single(
    id: &str,
    _user: AuthenticatedUser,
) -> V1ApiResponseType<V1RomFullResponse> {
    let rom = sqlx::query_as!(
        V1RomFullResponse,
        r#"SELECT r.id, r.title, r.realname, r.console, r.region, r.category, r.serial, r.rom_path, '/covers/' || r.image_hash || '.webp' as "image_path!", r.file_extension, r.file_size_bytes, r.md5_hash, r.release_year, r.created_at, r.languages,
            (SELECT COUNT(*) FROM roms r2 WHERE r2.title = r.title AND r2.console = r.console) as "versions_count!"
           FROM roms r WHERE r.id = $1"#, 
        id
    )
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
        r#"SELECT id, title, realname, '/covers_small/' || image_hash || '.webp' as "image_path!", console, category, region, release_year, serial, languages, "versions_count!" FROM (
             SELECT DISTINCT ON (console, title) id, title, realname, image_hash, console, category, region, release_year, serial, languages,
             (SELECT COUNT(*) FROM roms r2 WHERE r2.title = roms.title AND r2.console = roms.console) as "versions_count!"
             FROM roms
             WHERE (
                 title ILIKE '%' || $1 || '%'
                 OR serial ILIKE $1 || '%'
                 OR realname ILIKE '%' || $1 || '%'
             )
             AND (category = $2 OR $2 IS NULL)
             AND (console = $3 OR $3 IS NULL)
             ORDER BY console, title, region NULLS LAST, id DESC
         ) sub
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

#[get("/api/v1/roms/<id>/versions")]
pub async fn get_rom_versions(
    id: String,
    _user: AuthenticatedUser,
) -> V1ApiResponseType<Vec<V1RomListResponse>> {
    let base_rom = sqlx::query!("SELECT title, console FROM roms WHERE id = $1", id)
        .fetch_optional(&*SQL)
        .await
        .map_err(|_| V1ApiError::InternalError)?
        .ok_or(V1ApiError::NotFound)?;

    let versions = sqlx::query_as!(
        V1RomListResponse,
        r#"SELECT id, title, realname, '/covers_small/' || image_hash || '.webp' as "image_path!", console, category, region, release_year, serial, languages,
           1::bigint as "versions_count!"
           FROM roms WHERE title = $1 AND console = $2 ORDER BY region NULLS LAST"#,
        base_rom.title,
        base_rom.console
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| {
        error!("{:?}", e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(versions))
}

#[derive(FromForm)]
pub struct V1RomUpload<'r> {
    title: String,
    realname: Option<String>,
    console: String,
    category: String,
    region: Option<String>,
    release_year: Option<i32>,
    serial: Option<String>,
    md5_hash: Option<String>,
    languages: Option<String>,
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

    let img = image::load_from_memory(&img_bytes).map_err(|e| {
        error!("Failed to load image: {}", e);
        V1ApiError::InternalError
    })?;
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageFormat::WebP).map_err(|e| {
        error!("Failed to encode WebP: {}", e);
        V1ApiError::InternalError
    })?;

    let img_webp_bytes = buf.into_inner();
    let img_hash = compute_md5(&img_webp_bytes);
    let img_path = format!("/covers/{}.webp", img_hash);

    upload_object(&img_path, &img_webp_bytes)
        .await
        .map_err(|e| {
            error!("Failed to upload image to '{}': {:?}", img_path, e);
            V1ApiError::InternalError
        })?;

    let rom_id = crate::utils::snowflake::next_id();

    sqlx::query!(
        "INSERT INTO roms (id, title, realname, console, category, region, serial, release_year, rom_path, image_hash, file_extension, md5_hash, file_size_bytes, languages)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
         ON CONFLICT (id) DO UPDATE SET
            title = EXCLUDED.title,
            realname = EXCLUDED.realname,
            console = EXCLUDED.console,
            category = EXCLUDED.category,
            region = EXCLUDED.region,
            serial = EXCLUDED.serial,
            release_year = EXCLUDED.release_year,
            rom_path = EXCLUDED.rom_path,
            image_hash = EXCLUDED.image_hash,
            file_extension = EXCLUDED.file_extension,
            md5_hash = EXCLUDED.md5_hash,
            file_size_bytes = EXCLUDED.file_size_bytes,
            languages = EXCLUDED.languages",
        rom_id.to_string(),
        data.title,
        data.realname,
        data.console,
        data.category,
        data.region,
        data.serial,
        data.release_year,
        rom_path,
        img_hash,
        rom_ext,
        rom_md5,
        rom_bytes.len() as i64,
        data.languages
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
    pub realname: Option<String>,
    pub console: String,
    pub category: String,
    pub region: Option<String>,
    pub release_year: Option<i32>,
    pub serial: Option<String>,
    pub languages: Option<String>,
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
        "UPDATE roms SET title = $1, realname = $2, console = $3, category = $4, region = $5, release_year = $6, serial = $7, languages = $8 WHERE id = $9",
        data.title,
        data.realname,
        data.console,
        data.category,
        data.region,
        data.release_year,
        data.serial,
        data.languages,
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

    let rom = sqlx::query!("SELECT console, rom_path FROM roms WHERE id = $1", id)
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

    let ext = std::path::Path::new(&rom.rom_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");

    let new_rom_path = format!("/roms/{}/{}.{}", rom.console, new_md5, ext);

    upload_object(&new_rom_path, &rom_bytes)
        .await
        .map_err(|e| {
            error!("Failed to upload rom: {:?}", e);
            V1ApiError::InternalError
        })?;

    if new_rom_path != rom.rom_path {
        let _ = crate::utils::s3::delete_object(&rom.rom_path).await;
    }

    sqlx::query!(
        "UPDATE roms SET rom_path = $1, md5_hash = $2, file_size_bytes = $3 WHERE id = $4",
        new_rom_path,
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

    let _rom = sqlx::query!("SELECT image_hash FROM roms WHERE id = $1", id)
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

    let img = image::load_from_memory(&img_bytes).map_err(|e| {
        error!("Failed to load image: {}", e);
        V1ApiError::InternalError
    })?;

    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageFormat::WebP).map_err(|e| {
        error!("Failed to encode WebP: {}", e);
        V1ApiError::InternalError
    })?;

    let img_webp_bytes = buf.into_inner();
    let img_hash = compute_md5(&img_webp_bytes);
    let img_path = format!("/covers/{}.webp", img_hash);

    upload_object(&img_path, &img_webp_bytes)
        .await
        .map_err(|e| {
            error!("Failed to upload image: {:?}", e);
            V1ApiError::InternalError
        })?;

    let _ = crate::utils::s3::delete_object(&format!("/covers/{}.webp", _rom.image_hash)).await;
    let _ =
        crate::utils::s3::delete_object(&format!("/covers_small/{}.webp", _rom.image_hash)).await;
    let _ = crate::utils::s3::delete_object(&format!("/icons/{}.webp", _rom.image_hash)).await;

    sqlx::query!(
        "UPDATE roms SET image_hash = $1 WHERE id = $2",
        img_hash,
        id
    )
    .execute(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to update image path in DB: {:?}", e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(()))
}

#[delete("/api/v1/roms/<id>")]
pub async fn delete_rom(id: String, user: AuthenticatedUser) -> V1ApiResponseType<()> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::NotAuthorized);
    }

    let rom = sqlx::query!("SELECT rom_path, image_hash FROM roms WHERE id = $1", id)
        .fetch_optional(&*SQL)
        .await
        .map_err(|e| {
            error!("Failed to fetch rom for deletion: {:?}", e);
            V1ApiError::InternalError
        })?
        .ok_or(V1ApiError::NotFound)?;

    let _ = crate::utils::s3::delete_object(&rom.rom_path).await;
    let _ = crate::utils::s3::delete_object(&format!("/covers/{}.webp", rom.image_hash)).await;
    let _ =
        crate::utils::s3::delete_object(&format!("/covers_small/{}.webp", rom.image_hash)).await;
    let _ = crate::utils::s3::delete_object(&format!("/icons/{}.webp", rom.image_hash)).await;

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
    pub realname: Option<String>,
    pub image_path: String,
    pub console: String,
    pub region: Option<String>,
    pub play_count: i32,
    pub last_played: Option<chrono::DateTime<chrono::Utc>>,
    pub versions_count: i64,
}
impl V1ApiResponseTrait for Vec<V1UserLibraryResponse> {}

#[get("/api/v1/library")]
pub async fn get_user_library(
    user: AuthenticatedUser,
) -> V1ApiResponseType<Vec<V1UserLibraryResponse>> {
    let rows = sqlx::query!(
        r#"SELECT id as "ur_id!", rom_id, title, realname, image_hash, console, play_count, last_played, region, "versions_count!" FROM (
             SELECT DISTINCT ON (r.console, r.title) ur.id, ur.rom_id, r.title, r.realname, r.image_hash, r.console, ur.play_count, ur.last_played, r.region,
             (SELECT COUNT(*) FROM roms r2 WHERE r2.title = r.title AND r2.console = r.console) as "versions_count!"
             FROM user_roms ur
             INNER JOIN roms r ON ur.rom_id = r.id
             WHERE ur.user_id = $1
             ORDER BY r.console, r.title, r.region NULLS LAST
         ) sub
         ORDER BY last_played DESC NULLS LAST"#,
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
            realname: r.realname,
            image_path: format!("/covers_small/{}.webp", r.image_hash),
            console: r.console,
            play_count: r.play_count,
            last_played: r.last_played,
            versions_count: r.versions_count,
        })
        .collect();

    Ok(V1ApiResponse(library))
}

#[derive(FromForm)]
pub struct V1BulkUpload<'r> {
    zip_file: TempFile<'r>,
}

#[derive(serde::Deserialize, Debug)]
struct BulkInfoJson {
    title: Option<String>,
    md5: Option<String>,
    console: Option<String>,
    category: Option<String>,
    region: Option<String>,
    year: Option<i32>,
    serial: Option<String>,
    rom: Option<String>,
    cover: Option<String>,
    languages: Option<String>,
}

#[post(
    "/api/v1/roms/bulk_upload",
    format = "multipart/form-data",
    data = "<data>"
)]
pub async fn bulk_upload_roms(
    mut data: Form<V1BulkUpload<'_>>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<String> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::NotAuthorized);
    }

    let temp_path =
        std::env::temp_dir().join(format!("upload_{}.zip", crate::utils::snowflake::next_id()));
    info!("Starting bulk upload. Persisting zip to {:?}", temp_path);

    data.zip_file.persist_to(&temp_path).await.map_err(|e| {
        error!("Failed to persist zip: {:?}", e);
        V1ApiError::InternalError
    })?;

    let entries = tokio::task::spawn_blocking({
        let tp = temp_path.clone();
        move || -> Result<Vec<(BulkInfoJson, Vec<u8>, Vec<u8>)>, String> {
            let file = std::fs::File::open(&tp).map_err(|e| e.to_string())?;
            let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
            let mut results = Vec::new();

            let info_paths: Vec<String> = (0..archive.len())
                .filter_map(|i| {
                    let f = archive.by_index(i).ok()?;
                    if f.name().ends_with("info.json") {
                        Some(f.name().to_string())
                    } else {
                        None
                    }
                })
                .collect();

            info!("Found {} games to process in zip", info_paths.len());

            for path in info_paths {
                let folder = path.rfind('/').map(|i| &path[..=i]).unwrap_or("");
                let info: BulkInfoJson = {
                    let mut info_file = archive.by_name(&path).map_err(|_| "Missing info")?;
                    serde_json::from_reader(&mut info_file).map_err(|_| "JSON error")?
                };

                let rom_p = format!("{}{}", folder, info.rom.as_deref().unwrap_or(""));
                let cov_p = format!("{}{}", folder, info.cover.as_deref().unwrap_or(""));

                let mut rb = Vec::new();
                archive
                    .by_name(&rom_p)
                    .map_err(|_| "No ROM")?
                    .read_to_end(&mut rb)
                    .map_err(|_| "Read error")?;
                let mut cb = Vec::new();
                archive
                    .by_name(&cov_p)
                    .map_err(|_| "No Cover")?
                    .read_to_end(&mut cb)
                    .map_err(|_| "Read error")?;

                results.push((info, rb, cb));
            }
            Ok(results)
        }
    })
    .await
    .map_err(|_| {
        let _ = std::fs::remove_file(&temp_path);
        V1ApiError::InternalError
    })?
    .map_err(|e| {
        error!("Extraction failed: {}", e);
        let _ = std::fs::remove_file(&temp_path);
        V1ApiError::InternalError
    })?;

    let mut success_count = 0;
    let mut fail_count = 0;

    for (info, rom_bytes, cover_bytes) in entries {
        let title = info.title.clone().unwrap_or_else(|| "Unknown".into());
        let rom_md5 = compute_md5(&rom_bytes);
        let img = match image::load_from_memory(&cover_bytes) {
            Ok(i) => i,
            Err(e) => {
                error!("Failed to load image for {}: {}", title, e);
                fail_count += 1;
                continue;
            }
        };
        let mut buf = Cursor::new(Vec::new());
        if let Err(e) = img.write_to(&mut buf, ImageFormat::WebP) {
            error!("Failed to encode WebP for {}: {}", title, e);
            fail_count += 1;
            continue;
        }
        let img_webp_bytes = buf.into_inner();
        let img_hash = compute_md5(&img_webp_bytes);

        let console = info.console.clone().unwrap_or_else(|| "Unknown".into());
        let rom_ext = info
            .rom
            .as_ref()
            .and_then(|r| Path::new(r).extension())
            .and_then(|e| e.to_str())
            .unwrap_or("bin");

        let s3_rom = format!("/roms/{}/{}.{}", console, rom_md5, rom_ext);
        let s3_img = format!("/covers/{}.webp", img_hash);

        info!("Uploading files for: {}", title);
        if tokio::try_join!(
            upload_object(&s3_rom, &rom_bytes),
            upload_object(&s3_img, &img_webp_bytes)
        )
        .is_err()
        {
            error!("S3 upload failed for: {}", title);
            fail_count += 1;
            continue;
        }

        let mut tx = match SQL.begin().await {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to start DB transaction: {:?}", e);
                let _ = tokio::join!(delete_object(&s3_rom), delete_object(&s3_img));
                fail_count += 1;
                continue;
            }
        };

        let rom_id = crate::utils::snowflake::next_id().to_string();
        let db_res = sqlx::query!(
            "INSERT INTO roms (id, title, console, category, region, serial, release_year, rom_path, image_hash, file_extension, md5_hash, file_size_bytes, languages)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
             ON CONFLICT (id) DO UPDATE SET title = EXCLUDED.title",
            rom_id, info.title, console, info.category, info.region, info.serial, info.year, s3_rom, img_hash, rom_ext, rom_md5, rom_bytes.len() as i64, info.languages
        ).execute(&mut *tx).await;

        if db_res.is_ok() && tx.commit().await.is_ok() {
            info!("Successfully imported: {}", title);
            success_count += 1;
        } else {
            error!("Database commit failed for: {}. Rolling back S3.", title);
            let _ = tokio::join!(delete_object(&s3_rom), delete_object(&s3_img));
            fail_count += 1;
        }
    }

    let _ = std::fs::remove_file(temp_path);
    info!(
        "Bulk upload complete. Success: {}, Failed: {}",
        success_count, fail_count
    );
    Ok(V1ApiResponse(format!(
        "Processed {} successfully, {} failed.",
        success_count, fail_count
    )))
}
