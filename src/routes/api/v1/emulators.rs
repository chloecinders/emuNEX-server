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
        s3::{compute_md5, upload_object},
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
        "INSERT INTO emulators (id, name, consoles, platform, run_command, binary_name, save_path, save_extensions, binary_path, md5_hash, input_config_file, input_mapper, zipped, file_size)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
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
        file_size
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

    sqlx::query!(
        "UPDATE emulators SET name = $1, consoles = $2, platform = $3, run_command = $4, binary_name = $5, save_path = $6, save_extensions = $7, input_config_file = $8, input_mapper = $9, zipped = $10 WHERE id = $11",
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
