use std::path::Path;

use rocket::{
    form::{Form, FromForm},
    fs::TempFile,
    get, post, put, delete,
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
    utils::s3::upload_object,
};

#[derive(Serialize, sqlx::FromRow)]
pub struct V1EmulatorResponse {
    pub id: i32,
    pub name: String,
    pub console: String,
    pub platform: String,
    pub run_command: String,
    pub binary_path: String,
    pub md5_hash: Option<String>,
    pub config_files: Vec<String>,
    pub zipped: bool,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl V1ApiResponseTrait for Vec<V1EmulatorResponse> {}

#[get("/api/v1/emulators/<console>/<platform>")]
pub async fn get_emulators_for_platform(
    console: String,
    platform: String,
    _user: AuthenticatedUser,
) -> V1ApiResponseType<Vec<V1EmulatorResponse>> {
    let emulators = sqlx::query_as!(
        V1EmulatorResponse,
        r#"
        SELECT
            id,
            name,
            console,
            platform,
            run_command,
            binary_path,
            md5_hash,
            config_files as "config_files!",
            zipped as "zipped!",
            created_at
        FROM emulators
        WHERE console = $1 AND platform = $2
        ORDER BY name ASC
        "#,
        console,
        platform
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| {
        error!(
            "Database error fetching emulators for console '{}', platform '{}': {:?}",
            console, platform, e
        );
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(emulators))
}

#[get("/api/v1/emulators/all")]
pub async fn get_all_emulators(
    _user: AuthenticatedUser,
) -> V1ApiResponseType<Vec<V1EmulatorResponse>> {
    let emulators = sqlx::query_as!(
        V1EmulatorResponse,
        r#"
        SELECT
            id,
            name,
            console,
            platform,
            run_command,
            binary_path,
            md5_hash,
            config_files as "config_files!",
            zipped as "zipped!",
            created_at
        FROM emulators
        ORDER BY console ASC, name ASC
        "#
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| {
        error!("Database error fetching all emulators: {:?}", e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(emulators))
}

#[derive(FromForm)]
pub struct V1EmulatorUploadRequest<'r> {
    pub name: String,
    pub console: String,
    pub platform: String,
    pub run_command: String,
    pub binary_file: TempFile<'r>,
    pub config_files: Vec<String>,
    pub zipped: bool,
}

#[post(
    "/api/v1/emulators/upload",
    format = "multipart/form-data",
    data = "<data>"
)]
pub async fn emulator_upload(
    data: Form<V1EmulatorUploadRequest<'_>>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<i32> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::NotAuthorized);
    }

    let bin_filename = data
        .binary_file
        .raw_name()
        .map(|n| n.dangerous_unsafe_unsanitized_raw())
        .unwrap_or("emulator.bin".into());

    let bin_ext = Path::new(bin_filename.as_str())
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("bin");

    let binary_path = format!(
        "/emulators/{}/{}/{}.{}",
        data.platform.to_lowercase(),
        data.console.to_lowercase().replace(" ", "_"),
        data.name.to_lowercase().replace(" ", "_"),
        bin_ext
    );

    let bin_bytes = tokio::fs::read(data.binary_file.path().unwrap())
        .await
        .map_err(|e| {
            error!("Failed to read emulator binary from temp storage: {:?}", e);
            V1ApiError::InternalError
        })?;

    upload_object(&binary_path, &bin_bytes).await.map_err(|e| {
        error!(
            "Failed to upload emulator binary to '{}': {:?}",
            binary_path, e
        );
        V1ApiError::InternalError
    })?;

    let rec = sqlx::query!(
        "INSERT INTO emulators (name, console, platform, run_command, binary_path, config_files, zipped)
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
        data.name,
        data.console,
        data.platform,
        data.run_command,
        binary_path,
        &data.config_files,
        data.zipped
    )
    .fetch_one(&*SQL)
    .await
    .map_err(|e| {
        error!("Database error inserting emulator '{}': {:?}", data.name, e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(rec.id))
}

#[derive(serde::Deserialize)]
pub struct V1EmulatorUpdateRequest {
    pub name: String,
    pub console: String,
    pub platform: String,
    pub run_command: String,
    pub config_files: Vec<String>,
    pub zipped: bool,
}

#[put("/api/v1/emulators/<id>", format = "json", data = "<data>")]
pub async fn update_emulator(
    id: i32,
    data: Json<V1EmulatorUpdateRequest>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<i32> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::NotAuthorized);
    }

    sqlx::query!(
        "UPDATE emulators SET name = $1, console = $2, platform = $3, run_command = $4, config_files = $5, zipped = $6 WHERE id = $7",
        data.name,
        data.console,
        data.platform,
        data.run_command,
        &data.config_files,
        data.zipped,
        id
    )
    .execute(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to update emulator id {}: {:?}", id, e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(id))
}

#[derive(FromForm)]
pub struct V1BinaryUpdate<'r> {
    binary: TempFile<'r>,
}

#[post("/api/v1/emulators/<id>/binary", format = "multipart/form-data", data = "<data>")]
pub async fn update_emulator_binary(
    id: i32,
    data: Form<V1BinaryUpdate<'_>>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<()> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::NotAuthorized);
    }

    let emu = sqlx::query!("SELECT binary_path FROM emulators WHERE id = $1", id)
        .fetch_optional(&*SQL)
        .await
        .map_err(|e| { error!("{:?}", e); V1ApiError::InternalError })?
        .ok_or(V1ApiError::NotFound)?;

    let bin_bytes = tokio::fs::read(data.binary.path().unwrap())
        .await
        .map_err(|e| { error!("{:?}", e); V1ApiError::InternalError })?;

    upload_object(&emu.binary_path, &bin_bytes).await.map_err(|e| {
        error!("Failed to upload binary: {:?}", e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(()))
}

#[delete("/api/v1/emulators/<id>")]
pub async fn delete_emulator(id: i32, user: AuthenticatedUser) -> V1ApiResponseType<()> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::NotAuthorized);
    }

    let emu = sqlx::query!("SELECT binary_path FROM emulators WHERE id = $1", id)
        .fetch_optional(&*SQL)
        .await
        .map_err(|e| {
            error!("Failed to fetch emulator for deletion: {:?}", e);
            V1ApiError::InternalError
        })?
        .ok_or(V1ApiError::NotFound)?;

    let _ = crate::utils::s3::delete_object(&emu.binary_path).await;

    sqlx::query!("DELETE FROM emulators WHERE id = $1", id)
        .execute(&*SQL)
        .await
        .map_err(|e| {
            error!("Failed to delete emulator id {}: {:?}", id, e);
            V1ApiError::InternalError
        })?;

    Ok(V1ApiResponse(()))
}
