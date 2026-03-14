use std::{collections::HashMap, path::PathBuf};

use base64::{Engine as _, engine::general_purpose};
use rocket::{get, post, serde::json::Json};
use serde::Serialize;

use crate::{
    CONFIG, SQL,
    routes::api::{
        V1ApiError, V1ApiResponse, V1ApiResponseType, api_response::V1ApiResponseTrait,
        v1::guards::AuthenticatedUser,
    },
};

#[derive(serde::Deserialize)]
pub struct JsonMultiFileSave {
    pub files: HashMap<String, String>,
}

#[post("/api/v1/roms/<id>/save/<version_id>", data = "<data>")]
pub async fn upload_save(
    id: i32,
    version_id: i32,
    data: Json<JsonMultiFileSave>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<()> {
    let client = reqwest::Client::new();

    for (file_name, b64_content) in data.files.iter() {
        let buffer = general_purpose::STANDARD
            .decode(b64_content)
            .map_err(|_| V1ApiError::BadRequest)?;

        let save_path = format!("/saves/{}/{}/{}/{}", user.id, id, version_id, file_name);

        client
            .put(format!("{}{}", CONFIG.seaweedfs_url, save_path))
            .body(buffer)
            .send()
            .await
            .map_err(|_| V1ApiError::InternalError)?;

        sqlx::query!(
            "INSERT INTO user_save_files (user_id, rom_id, version_id, file_name, save_path)
             VALUES ($1, $2, $3, $4, $5)",
            user.id,
            id,
            version_id,
            file_name,
            save_path
        )
        .execute(&*SQL)
        .await
        .map_err(|_| V1ApiError::InternalError)?;
    }

    Ok(V1ApiResponse(()))
}

#[get("/api/v1/roms/<id>/save/<version_id>/<file_name>", rank = 1)]
pub async fn download_save_file(
    id: i32,
    version_id: i32,
    file_name: PathBuf,
    user: AuthenticatedUser,
) -> Result<Vec<u8>, V1ApiError> {
    let file_name_str = file_name.to_str().ok_or(V1ApiError::BadRequest)?;

    let record = sqlx::query!(
        "SELECT save_path FROM user_save_files
         WHERE user_id = $1 AND rom_id = $2 AND version_id = $3 AND file_name = $4",
        user.id,
        id,
        version_id,
        file_name_str
    )
    .fetch_optional(&*SQL)
    .await
    .map_err(|_| V1ApiError::InternalError)?
    .ok_or(V1ApiError::NotFound)?;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}{}", CONFIG.seaweedfs_url, record.save_path))
        .send()
        .await
        .map_err(|_| V1ApiError::InternalError)?;

    let bytes = resp.bytes().await.map_err(|_| V1ApiError::InternalError)?;
    Ok(bytes.to_vec())
}

#[derive(Serialize)]
pub struct SaveFileMetadata {
    pub file_name: String,
    pub version_id: i32,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
impl V1ApiResponseTrait for Vec<SaveFileMetadata> {}

#[get("/api/v1/roms/<id>/save/latest")]
pub async fn get_latest_save(
    id: i32,
    user: AuthenticatedUser,
) -> V1ApiResponseType<Vec<SaveFileMetadata>> {
    let latest_version = sqlx::query!(
        "SELECT version_id FROM user_save_files
         WHERE user_id = $1 AND rom_id = $2
         ORDER BY version_id DESC LIMIT 1",
        user.id,
        id
    )
    .fetch_optional(&*SQL)
    .await
    .map_err(|_| V1ApiError::InternalError)?
    .ok_or(V1ApiError::NotFound)?;

    let files = sqlx::query_as!(
        SaveFileMetadata,
        "SELECT file_name, version_id, created_at FROM user_save_files
         WHERE user_id = $1 AND rom_id = $2 AND version_id = $3",
        user.id,
        id,
        latest_version.version_id
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|_| V1ApiError::InternalError)?;

    Ok(V1ApiResponse(files))
}
