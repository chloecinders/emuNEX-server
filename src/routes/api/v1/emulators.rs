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
pub struct EmulatorUpload<'r> {
    pub name: String,
    pub console: String,
    pub platform: String,
    pub run_command: String,
    pub binary_file: TempFile<'r>,
    pub config_files: Vec<String>,
    pub zipped: bool,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Emulator {
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

impl V1ApiResponseTrait for Vec<Emulator> {}

#[get("/api/v1/emulators/<console>/<platform>")]
pub async fn get_emulators_for_platform(
    console: String,
    platform: String,
    _user: AuthenticatedUser,
) -> V1ApiResponseType<Vec<Emulator>> {
    let emulators = sqlx::query_as!(
        Emulator,
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

#[post(
    "/api/v1/emulators/upload",
    format = "multipart/form-data",
    data = "<data>"
)]
pub async fn emulator_upload(
    data: Form<EmulatorUpload<'_>>,
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
