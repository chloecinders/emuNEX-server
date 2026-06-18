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
    utils::{
        id::Id,
        s3::{compute_md5, presign_put_url, upload_object},
        snowflake::next_id,
    },
};

#[derive(Serialize, sqlx::FromRow)]
pub struct V1EmulatorResponse {
    pub id: Id,
    pub name: String,
    pub consoles: Vec<String>,
    pub platform: String,
    pub run_command: String,
    pub binary_path: String,
    pub binary_name: Option<String>,
    pub save_path: Option<String>,
    pub save_extensions: Vec<String>,
    pub md5_hash: Option<String>,
    pub input_config_file: Option<String>,
    pub input_mapper: Option<String>,
    pub zipped: bool,
    pub file_size: i64,
    pub version: Option<String>,
    pub extra_files: serde_json::Value,
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
            consoles as "consoles!",
            platform,
            run_command,
            binary_path,
            binary_name,
            save_path,
            save_extensions as "save_extensions!",
            md5_hash,
            input_config_file,
            input_mapper,
            zipped as "zipped!",
            file_size as "file_size!",
            version,
            extra_files as "extra_files!",
            created_at
        FROM emulators
        WHERE $1 ILIKE ANY(consoles) AND platform = $2
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
        V1ApiError::DatabaseError
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
            consoles as "consoles!",
            platform,
            run_command,
            binary_path,
            binary_name,
            save_path,
            save_extensions as "save_extensions!",
            md5_hash,
            input_config_file,
            input_mapper,
            zipped as "zipped!",
            file_size as "file_size!",
            version,
            extra_files as "extra_files!",
            created_at
        FROM emulators
        ORDER BY consoles ASC, name ASC
        "#
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| {
        error!("Database error fetching all emulators: {:?}", e);
        V1ApiError::DatabaseError
    })?;

    Ok(V1ApiResponse(emulators))
}

#[derive(FromForm)]
pub struct V1EmulatorUploadRequest<'r> {
    pub name: String,
    pub consoles: Vec<String>,
    pub platform: String,
    pub run_command: String,
    pub binary_name: Option<String>,
    pub save_path: String,
    pub save_extensions: Vec<String>,
    pub binary_file: TempFile<'r>,
    pub input_config_file: Option<String>,
    pub input_mapper: Option<String>,
    pub zipped: bool,
    pub version: Option<String>,
}

#[post(
    "/api/v1/emulators/upload",
    format = "multipart/form-data",
    data = "<data>"
)]
pub async fn emulator_upload(
    data: Form<V1EmulatorUploadRequest<'_>>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<Id> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::MissingPermissions);
    }

    let bin_ext = data
        .binary_file
        .raw_name()
        .and_then(|n| {
            let raw = n.dangerous_unsafe_unsanitized_raw();
            let filename = raw
                .to_string()
                .replace('\\', "/")
                .split('/')
                .last()
                .unwrap_or("")
                .to_string();
            filename.rsplit_once('.').map(|(_, ext)| ext.to_lowercase())
        })
        .unwrap_or_else(|| "bin".to_string());

    let bin_bytes = tokio::fs::read(data.binary_file.path().unwrap())
        .await
        .map_err(|e| {
            error!("Failed to read emulator binary from temp storage: {:?}", e);
            V1ApiError::DatabaseError
        })?;

    let file_size = bin_bytes.len() as i64;
    let md5 = compute_md5(&bin_bytes);

    let binary_path = format!(
        "/emulators/{}/{}/{}.{}",
        data.platform.to_lowercase(),
        data.consoles.join("_").to_lowercase().replace(" ", "_"),
        data.name.to_lowercase().replace(" ", "_"),
        bin_ext
    );

    upload_object(&binary_path, &bin_bytes).await.map_err(|e| {
        error!(
            "Failed to upload emulator binary to '{}': {:?}",
            binary_path, e
        );
        V1ApiError::DatabaseError
    })?;

    let id = next_id();

    sqlx::query!(
        "INSERT INTO emulators (id, name, consoles, platform, run_command, binary_name, save_path, save_extensions, binary_path, md5_hash, input_config_file, input_mapper, zipped, file_size, version, extra_files)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)",
        id,
        data.name,
        &data.consoles,
        data.platform,
        data.run_command,
        data.binary_name,
        data.save_path,
        &data.save_extensions,
        binary_path,
        md5,
        data.input_config_file,
        data.input_mapper,
        data.zipped,
        file_size,
        data.version,
        serde_json::json!([])
    )
    .execute(&*SQL)
    .await
    .map_err(|e| {
        error!("Database error inserting emulator '{}': {:?}", data.name, e);
        V1ApiError::DatabaseError
    })?;

    Ok(V1ApiResponse(Id::new(id)))
}

#[derive(serde::Deserialize)]
pub struct V1EmulatorUpdateRequest {
    pub name: String,
    pub consoles: Vec<String>,
    pub platform: String,
    pub run_command: String,
    pub binary_name: Option<String>,
    pub save_path: String,
    pub save_extensions: Vec<String>,
    pub input_config_file: Option<String>,
    pub input_mapper: Option<String>,
    pub zipped: bool,
    pub version: Option<String>,
    pub extra_files: Option<serde_json::Value>,
}

#[put("/api/v1/emulators/<id>", format = "json", data = "<data>")]
pub async fn update_emulator(
    id: i64,
    data: Json<V1EmulatorUpdateRequest>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<Id> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::MissingPermissions);
    }

    let data = data.into_inner();
    let extra_files_json = data.extra_files.unwrap_or_else(|| serde_json::json!([]));

    sqlx::query!(
        "UPDATE emulators SET name = $1, consoles = $2, platform = $3, run_command = $4, binary_name = $5, save_path = $6, save_extensions = $7, input_config_file = $8, input_mapper = $9, zipped = $10, version = $11, extra_files = $12 WHERE id = $13",
        data.name,
        &data.consoles,
        data.platform,
        data.run_command,
        data.binary_name,
        data.save_path,
        &data.save_extensions,
        data.input_config_file,
        data.input_mapper,
        data.zipped,
        data.version,
        extra_files_json,
        id
    )
    .execute(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to update emulator id {}: {:?}", id, e);
        V1ApiError::DatabaseError
    })?;

    Ok(V1ApiResponse(Id::new(id)))
}

#[derive(serde::Deserialize)]
pub struct V1ExtraFileSignRequest {
    pub filename: String,
    pub windows_path: String,
    pub linux_path: String,
    pub macos_path: String,
}

#[derive(serde::Serialize)]
pub struct V1ExtraFileSignResponse {
    pub upload_url: String,
    pub s3_path: String,
    pub windows_path: String,
    pub linux_path: String,
    pub macos_path: String,
}

impl V1ApiResponseTrait for V1ExtraFileSignResponse {}
impl V1ApiResponseTrait for serde_json::Value {}

#[post(
    "/api/v1/emulators/<id>/extra-file/sign",
    format = "json",
    data = "<data>"
)]
pub async fn sign_extra_file_upload(
    id: i64,
    data: Json<V1ExtraFileSignRequest>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<V1ExtraFileSignResponse> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::MissingPermissions);
    }

    let emu = sqlx::query!("SELECT name, platform FROM emulators WHERE id = $1", id)
        .fetch_optional(&*SQL)
        .await
        .map_err(|e| {
            error!("{:?}", e);
            V1ApiError::DatabaseError
        })?
        .ok_or(V1ApiError::EmulatorNotFound)?;

    let safe_name = emu.name.to_lowercase().replace(" ", "_");
    let safe_filename = data
        .filename
        .replace('\\', "/")
        .split('/')
        .last()
        .unwrap_or("file")
        .to_string();

    let s3_path = format!(
        "/emulators/{}/{}/extra/{}",
        emu.platform.to_lowercase(),
        safe_name,
        safe_filename
    );

    let upload_url = presign_put_url(&s3_path, 900).await.map_err(|e| {
        error!("Failed to generate presigned URL for '{}': {}", s3_path, e);
        V1ApiError::DatabaseError
    })?;

    Ok(V1ApiResponse(V1ExtraFileSignResponse {
        upload_url,
        s3_path,
        windows_path: data.windows_path.clone(),
        linux_path: data.linux_path.clone(),
        macos_path: data.macos_path.clone(),
    }))
}

#[derive(serde::Deserialize)]
pub struct V1ExtraFileConfirmRequest {
    pub s3_path: String,
    pub windows_path: String,
    pub linux_path: String,
    pub macos_path: String,
}

#[post(
    "/api/v1/emulators/<id>/extra-file/confirm",
    format = "json",
    data = "<data>"
)]
pub async fn confirm_extra_file(
    id: i64,
    data: Json<V1ExtraFileConfirmRequest>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<serde_json::Value> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::MissingPermissions);
    }

    let emu = sqlx::query!("SELECT extra_files FROM emulators WHERE id = $1", id)
        .fetch_optional(&*SQL)
        .await
        .map_err(|e| {
            error!("{:?}", e);
            V1ApiError::DatabaseError
        })?
        .ok_or(V1ApiError::EmulatorNotFound)?;

    let new_entry = serde_json::json!({
        "s3_path": data.s3_path,
        "windows_path": data.windows_path,
        "linux_path": data.linux_path,
        "macos_path": data.macos_path,
    });

    let mut extra_files: Vec<serde_json::Value> =
        serde_json::from_value(emu.extra_files).unwrap_or_default();
    extra_files.push(new_entry.clone());

    sqlx::query!(
        "UPDATE emulators SET extra_files = $1 WHERE id = $2",
        serde_json::Value::Array(extra_files),
        id
    )
    .execute(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to confirm extra file for emulator {}: {:?}", id, e);
        V1ApiError::DatabaseError
    })?;

    Ok(V1ApiResponse(new_entry))
}

#[derive(FromForm)]
pub struct V1BinaryUpdate<'r> {
    binary: TempFile<'r>,
}

#[post(
    "/api/v1/emulators/<id>/binary",
    format = "multipart/form-data",
    data = "<data>"
)]
pub async fn update_emulator_binary(
    id: i64,
    data: Form<V1BinaryUpdate<'_>>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<()> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::MissingPermissions);
    }

    let emu = sqlx::query!("SELECT binary_path FROM emulators WHERE id = $1", id)
        .fetch_optional(&*SQL)
        .await
        .map_err(|e| {
            error!("{:?}", e);
            V1ApiError::DatabaseError
        })?
        .ok_or(V1ApiError::EmulatorNotFound)?;

    let bin_bytes = tokio::fs::read(data.binary.path().unwrap())
        .await
        .map_err(|e| {
            error!("{:?}", e);
            V1ApiError::DatabaseError
        })?;

    let file_size = bin_bytes.len() as i64;
    let new_md5 = compute_md5(&bin_bytes);

    upload_object(&emu.binary_path, &bin_bytes)
        .await
        .map_err(|e| {
            error!("Failed to upload binary: {:?}", e);
            V1ApiError::DatabaseError
        })?;

    sqlx::query!(
        "UPDATE emulators SET md5_hash = $1, file_size = $2 WHERE id = $3",
        new_md5,
        file_size,
        id
    )
    .execute(&*SQL)
    .await
    .map_err(|e| {
        error!("{:?}", e);
        V1ApiError::DatabaseError
    })?;

    Ok(V1ApiResponse(()))
}

#[delete("/api/v1/emulators/<id>")]
pub async fn delete_emulator(id: i64, user: AuthenticatedUser) -> V1ApiResponseType<()> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::MissingPermissions);
    }

    let emu = sqlx::query!("SELECT binary_path FROM emulators WHERE id = $1", id)
        .fetch_optional(&*SQL)
        .await
        .map_err(|e| {
            error!("Failed to fetch emulator for deletion: {:?}", e);
            V1ApiError::DatabaseError
        })?
        .ok_or(V1ApiError::EmulatorNotFound)?;

    let _ = crate::utils::s3::delete_object(&emu.binary_path).await;

    sqlx::query!("DELETE FROM emulators WHERE id = $1", id)
        .execute(&*SQL)
        .await
        .map_err(|e| {
            error!("Failed to delete emulator id {}: {:?}", id, e);
            V1ApiError::DatabaseError
        })?;

    Ok(V1ApiResponse(()))
}
