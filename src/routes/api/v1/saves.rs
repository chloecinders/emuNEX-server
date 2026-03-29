use base64::{Engine as _, engine::general_purpose};
use rocket::{get, post, serde::json::Json};
use serde::Serialize;
use tracing::error;

use crate::{
    SQL,
    routes::api::{
        V1ApiError, V1ApiResponse, V1ApiResponseType, api_response::V1ApiResponseTrait,
        v1::guards::AuthenticatedUser,
    },
    utils::s3::{download_object, upload_object},
};

#[derive(serde::Deserialize)]
pub struct V1SaveUploadFile {
    pub hash: String,
    pub path: String,
    pub content: String,
}

#[derive(serde::Deserialize)]
pub struct V1SaveUploadRequest {
    pub files: Vec<V1SaveUploadFile>,
}

#[post("/api/v1/roms/<id>/save/<version_id>", data = "<data>")]
pub async fn upload_save(
    id: String,
    version_id: i64,
    data: Json<V1SaveUploadRequest>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<()> {
    for file in data.files.iter() {
        let buffer = general_purpose::STANDARD
            .decode(&file.content)
            .map_err(|e| {
                error!("Failed to decode base64 for file '{}': {:?}", file.path, e);
                V1ApiError::BadRequest
            })?;

        let save_path = format!(
            "/saves/{}/{}/{}/{}",
            user.id.value(),
            id,
            version_id,
            file.hash
        );

        upload_object(&save_path, &buffer).await.map_err(|e| {
            error!(
                "Failed to upload save file '{}' to '{}': {:?}",
                file.path, save_path, e
            );
            V1ApiError::InternalError
        })?;

        let save_id = crate::utils::snowflake::next_id();

        sqlx::query!(
            "INSERT INTO user_save_files (id, user_id, rom_id, version_id, file_name, save_path)
             VALUES ($1, $2, $3, $4, $5, $6)",
            save_id,
            user.id.value(),
            id,
            version_id,
            file.path,
            save_path
        )
        .execute(&*SQL)
        .await
        .map_err(|e| {
            error!(
                "Database error inserting save file '{}': {:?}",
                file.path, e
            );
            V1ApiError::InternalError
        })?;
    }

    Ok(V1ApiResponse(()))
}

#[derive(serde::Deserialize)]
pub struct V1SaveDownloadRequest {
    pub path: String,
}

#[post("/api/v1/roms/<id>/save/<version_id>/download", data = "<data>")]
pub async fn download_save_file(
    id: String,
    version_id: i64,
    data: Json<V1SaveDownloadRequest>,
    user: AuthenticatedUser,
) -> Result<Vec<u8>, V1ApiError> {
    let file_name_str = &data.path;

    let record = sqlx::query!(
        "SELECT save_path FROM user_save_files
         WHERE user_id = $1 AND rom_id = $2 AND version_id = $3 AND file_name = $4",
        user.id.value(),
        id,
        version_id,
        file_name_str
    )
    .fetch_optional(&*SQL)
    .await
    .map_err(|e| {
        error!(
            "Database error fetching save file for id {}, version {}: {:?}",
            id, version_id, e
        );
        V1ApiError::InternalError
    })?
    .ok_or(V1ApiError::NotFound)?;

    let bytes = download_object(&record.save_path).await.map_err(|e| {
        error!(
            "Failed to download save file from '{}': {:?}",
            record.save_path, e
        );
        V1ApiError::InternalError
    })?;

    Ok(bytes)
}

#[derive(Serialize)]
pub struct V1SaveFileMetadataResponse {
    pub file_name: String,
    pub version_id: i64,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
impl V1ApiResponseTrait for Vec<V1SaveFileMetadataResponse> {}

#[get("/api/v1/roms/<id>/save/latest")]
pub async fn get_latest_save(
    id: String,
    user: AuthenticatedUser,
) -> V1ApiResponseType<Vec<V1SaveFileMetadataResponse>> {
    let latest_version = sqlx::query!(
        "SELECT version_id FROM user_save_files
         WHERE user_id = $1 AND rom_id = $2
         ORDER BY version_id DESC LIMIT 1",
        user.id.value(),
        id
    )
    .fetch_optional(&*SQL)
    .await
    .map_err(|e| {
        error!(
            "Database error fetching latest version for rom {}: {:?}",
            id, e
        );
        V1ApiError::InternalError
    })?
    .ok_or(V1ApiError::NotFound)?;

    let files = sqlx::query_as!(
        V1SaveFileMetadataResponse,
        "SELECT file_name, version_id, created_at FROM user_save_files
         WHERE user_id = $1 AND rom_id = $2 AND version_id = $3",
        user.id.value(),
        id,
        latest_version.version_id
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| {
        error!(
            "Database error fetching save metadata for rom {}, version {}: {:?}",
            id, latest_version.version_id, e
        );
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(files))
}
