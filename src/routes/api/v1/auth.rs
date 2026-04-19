use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use rocket::{get, post, put, serde::json::Json};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use crate::{
    SQL,
    routes::api::{
        V1ApiResponse, V1ApiResponseType,
        api_error::V1ApiError,
        api_response::V1ApiResponseTrait,
        v1::guards::{AuthToken, AuthenticatedUser, UserRole},
    },
    utils::{id::Id, snowflake::next_id},
};

#[derive(Serialize)]
pub struct V1ClientStartResponse {
    login_url: String,
    storage_path: String,
}

impl V1ApiResponseTrait for V1ClientStartResponse {}

#[get("/api/v1/start")]
pub async fn client_start() -> V1ApiResponseType<V1ClientStartResponse> {
    return Ok(V1ApiResponse(V1ClientStartResponse {
        login_url: "/auth/login".into(),
        storage_path: "/storage".into(),
    }));
}

#[derive(Deserialize)]
pub struct V1AuthRegisterRequest {
    pub username: String,
    pub password: String,
    pub invite_code: String,
}

#[derive(Serialize)]
pub struct V1AuthResponse {
    token: String,
}

impl V1ApiResponseTrait for V1AuthResponse {}

#[post("/api/v1/register", format = "json", data = "<request_data>")]
pub async fn register(
    request_data: Json<V1AuthRegisterRequest>,
) -> V1ApiResponseType<V1AuthResponse> {
    let data = request_data.into_inner();

    let invite = sqlx::query!(
        "SELECT id FROM invite_codes WHERE code = $1 AND used_by IS NULL",
        data.invite_code
    )
    .fetch_optional(&*SQL)
    .await
    .map_err(|_| V1ApiError::InternalError)?
    .ok_or(V1ApiError::InvalidInviteCode)?;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(data.password.as_bytes(), &salt)
        .map_err(|_| V1ApiError::InternalError)?
        .to_string();

    let user_id = next_id();

    let mut tx = SQL.begin().await.map_err(|_| V1ApiError::InternalError)?;

    sqlx::query!(
        "INSERT INTO users (id, username, password) VALUES ($1, $2, $3)",
        user_id,
        data.username,
        password_hash
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        if let Some(db_err) = e.as_database_error() {
            if db_err.code() == Some(std::borrow::Cow::Borrowed("23505")) {
                return V1ApiError::UsernameTaken;
            }
        }
        V1ApiError::InternalError
    })?;

    sqlx::query!(
        "UPDATE invite_codes SET used_by = $1, used_at = NOW() WHERE id = $2",
        user_id,
        invite.id
    )
    .execute(&mut *tx)
    .await
    .map_err(|_| V1ApiError::InternalError)?;

    tx.commit().await.map_err(|_| V1ApiError::InternalError)?;

    let token = Uuid::new_v4().to_string();
    let token_id = next_id();

    sqlx::query!(
        "INSERT INTO user_tokens (id, user_id, token, expires_at) VALUES ($1, $2, $3, NOW() + INTERVAL '30 days')",
        token_id,
        user_id,
        token
    )
    .execute(&*SQL)
    .await
    .map_err(|_| V1ApiError::InternalError)?;

    Ok(V1ApiResponse(V1AuthResponse { token }))
}

#[derive(Deserialize)]
pub struct V1AuthLoginRequest {
    pub username: String,
    pub password: String,
}

#[post("/api/v1/login", format = "json", data = "<request_data>")]
pub async fn login(request_data: Json<V1AuthLoginRequest>) -> V1ApiResponseType<V1AuthResponse> {
    let data = request_data.into_inner();

    let user = sqlx::query!(
        "SELECT id, password FROM users WHERE username = $1",
        data.username
    )
    .fetch_optional(&*SQL)
    .await
    .map_err(|_| V1ApiError::DatabaseError)?
    .ok_or(V1ApiError::InvalidCredentials)?;

    let password_str = user
        .password
        .as_deref()
        .ok_or(V1ApiError::InvalidCredentials)?;
    let parsed_hash = PasswordHash::new(password_str).map_err(|_| V1ApiError::InternalError)?;

    Argon2::default()
        .verify_password(data.password.as_bytes(), &parsed_hash)
        .map_err(|_| V1ApiError::InvalidCredentials)?;

    let token = Uuid::new_v4().to_string();
    let token_id = next_id();

    sqlx::query!(
        "INSERT INTO user_tokens (id, user_id, token, expires_at) VALUES ($1, $2, $3, NOW() + INTERVAL '30 days')",
        token_id,
        user.id,
        token
    )
    .execute(&*SQL)
    .await
    .map_err(|_| V1ApiError::DatabaseError)?;

    Ok(V1ApiResponse(V1AuthResponse { token }))
}

#[derive(Serialize)]
pub struct V1AuthMeResponse {
    id: Id,
    username: String,
    role: UserRole,
    theme: String,
    avatar_path: Option<String>,
    profile_color: String,
    discord_id: Option<String>,
    has_password: bool,
    has_migrated: bool,
}

impl V1ApiResponseTrait for V1AuthMeResponse {}

#[get("/api/v1/users/@me")]
pub async fn me(user: AuthenticatedUser) -> V1ApiResponseType<V1AuthMeResponse> {
    let row = sqlx::query(
        r#"SELECT u.theme, u.avatar_hash, u.profile_color, dc.discord_id, u.password IS NOT NULL as has_password, u.has_migrated
         FROM users u
         LEFT JOIN discord_connections dc ON u.id = dc.user_id
         WHERE u.id = $1"#
    )
    .bind(user.id.0)
    .fetch_one(&*SQL)
    .await
    .map_err(|_| V1ApiError::UserNotFound)?;

    use sqlx::Row;
    let theme: String = row.get("theme");
    let avatar_hash: Option<String> = row.try_get("avatar_hash").unwrap_or(None);
    let profile_color: String = row.get("profile_color");
    let discord_id: Option<String> = row.try_get("discord_id").unwrap_or(None);
    let has_password: bool = row.get("has_password");
    let has_migrated: bool = row.get("has_migrated");

    Ok(V1ApiResponse(V1AuthMeResponse {
        id: user.id,
        username: user.username,
        role: user.role,
        theme,
        avatar_path: avatar_hash.map(|h| format!("/avatars/{}/{}.webp", user.id.0, h)),
        profile_color,
        discord_id,
        has_password,
        has_migrated,
    }))
}

#[derive(Serialize)]
pub struct V1ThemeResponse {
    theme: String,
}

impl V1ApiResponseTrait for V1ThemeResponse {}

#[get("/api/v1/users/@me/preferences")]
pub async fn get_preferences(user: AuthenticatedUser) -> V1ApiResponseType<V1ThemeResponse> {
    let row = sqlx::query!("SELECT theme FROM users WHERE id = $1", user.id.0)
        .fetch_one(&*SQL)
        .await
        .map_err(|_| V1ApiError::UserNotFound)?;

    Ok(V1ApiResponse(V1ThemeResponse { theme: row.theme }))
}

#[derive(Deserialize)]
pub struct V1UpdatePreferencesRequest {
    pub theme: String,
}

#[rocket::put(
    "/api/v1/users/@me/preferences",
    format = "json",
    data = "<request_data>"
)]
pub async fn update_preferences(
    user: AuthenticatedUser,
    request_data: Json<V1UpdatePreferencesRequest>,
) -> V1ApiResponseType<String> {
    let data = request_data.into_inner();
    let theme = if data.theme == "dark" {
        "dark"
    } else {
        "light"
    };

    sqlx::query!(
        "UPDATE users SET theme = $1 WHERE id = $2",
        theme,
        user.id.0
    )
    .execute(&*SQL)
    .await
    .map_err(|_| V1ApiError::InternalError)?;

    Ok(V1ApiResponse("updated".into()))
}

#[post("/api/v1/logout")]
pub async fn logout(token: AuthToken) -> V1ApiResponseType<String> {
    sqlx::query!("DELETE FROM user_tokens WHERE token = $1", token.0)
        .execute(&*SQL)
        .await
        .map_err(|_| V1ApiError::InternalError)?;
    Ok(V1ApiResponse("logged out".into()))
}

#[derive(Deserialize)]
pub struct V1AuthUpdateUsernameRequest {
    pub username: String,
}

#[rocket::put("/api/v1/users/@me/username", format = "json", data = "<request_data>")]
pub async fn update_username(
    user: AuthenticatedUser,
    request_data: Json<V1AuthUpdateUsernameRequest>,
) -> V1ApiResponseType<String> {
    let data = request_data.into_inner();
    let trimmed = data.username.trim();

    if trimmed.is_empty() {
        return Err(V1ApiError::InvalidUsername);
    }

    sqlx::query!(
        "UPDATE users SET username = $1 WHERE id = $2",
        trimmed,
        user.id.0
    )
    .execute(&*SQL)
    .await
    .map_err(|e| {
        if let Some(db_err) = e.as_database_error() {
            if db_err.code() == Some(std::borrow::Cow::Borrowed("23505")) {
                return V1ApiError::UsernameTaken;
            }
        }
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse("updated".into()))
}

#[derive(Deserialize)]
pub struct V1AuthUpdatePasswordRequest {
    pub current_password: Option<String>,
    pub new_password: String,
}

#[rocket::put("/api/v1/users/@me/password", format = "json", data = "<request_data>")]
pub async fn update_password(
    user: AuthenticatedUser,
    request_data: Json<V1AuthUpdatePasswordRequest>,
) -> V1ApiResponseType<String> {
    let data = request_data.into_inner();

    let user_row = sqlx::query!("SELECT password FROM users WHERE id = $1", user.id.0)
        .fetch_one(&*SQL)
        .await
        .map_err(|_| V1ApiError::UserNotFound)?;

    if let Some(password_str) = user_row.password.as_deref() {
        let current = data
            .current_password
            .as_deref()
            .ok_or(V1ApiError::InvalidCredentials)?;
        let parsed_hash = PasswordHash::new(password_str).map_err(|_| V1ApiError::InternalError)?;

        Argon2::default()
            .verify_password(current.as_bytes(), &parsed_hash)
            .map_err(|_| V1ApiError::InvalidCredentials)?;
    }

    let salt = SaltString::generate(&mut OsRng);
    let new_hash = Argon2::default()
        .hash_password(data.new_password.as_bytes(), &salt)
        .map_err(|_| V1ApiError::InternalError)?
        .to_string();

    let mut tx = SQL.begin().await.map_err(|_| V1ApiError::InternalError)?;

    sqlx::query!(
        "UPDATE users SET password = $1 WHERE id = $2",
        new_hash,
        user.id.0
    )
    .execute(&mut *tx)
    .await
    .map_err(|_| V1ApiError::InternalError)?;

    sqlx::query!("DELETE FROM user_tokens WHERE user_id = $1", user.id.0)
        .execute(&mut *tx)
        .await
        .map_err(|_| V1ApiError::InternalError)?;

    tx.commit().await.map_err(|_| V1ApiError::InternalError)?;

    Ok(V1ApiResponse("updated".into()))
}

#[derive(Deserialize)]
pub struct V1UploadAvatarRequest {
    pub content: String,
}

#[put("/api/v1/users/@me/avatar", format = "json", data = "<request_data>")]
pub async fn upload_avatar(
    user: AuthenticatedUser,
    request_data: Json<V1UploadAvatarRequest>,
) -> V1ApiResponseType<String> {
    use base64::{Engine as _, engine::general_purpose};

    let data = request_data.into_inner();

    let bytes = general_purpose::STANDARD
        .decode(&data.content)
        .map_err(|_| V1ApiError::InvalidFile)?;

    if bytes.len() > 10 * 1024 * 1024 {
        return Err(V1ApiError::InvalidFile);
    }

    let img = image::load_from_memory(&bytes).map_err(|_| V1ApiError::InvalidFile)?;
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::WebP)
        .map_err(|_| V1ApiError::InternalError)?;
    let webp_bytes = buf.into_inner();

    let hash = crate::utils::s3::compute_md5(&webp_bytes);
    let avatar_path = format!("/avatars/{}/{}.webp", user.id.value(), hash);

    crate::utils::s3::upload_object(&avatar_path, &webp_bytes)
        .await
        .map_err(|e| {
            error!(
                "Failed to upload avatar for user {}: {:?}",
                user.id.value(),
                e
            );
            V1ApiError::InternalError
        })?;

    sqlx::query("UPDATE users SET avatar_hash = $1 WHERE id = $2")
        .bind(hash)
        .bind(user.id.0)
        .execute(&*SQL)
        .await
        .map_err(|e| {
            error!(
                "Failed to save avatar_path for user {}: {:?}",
                user.id.value(),
                e
            );
            V1ApiError::InternalError
        })?;

    Ok(V1ApiResponse(avatar_path))
}

#[derive(Deserialize)]
pub struct V1UpdateProfileColorRequest {
    pub color: String,
}

#[rocket::put(
    "/api/v1/users/@me/profile-color",
    format = "json",
    data = "<request_data>"
)]
pub async fn update_profile_color(
    user: AuthenticatedUser,
    request_data: Json<V1UpdateProfileColorRequest>,
) -> V1ApiResponseType<String> {
    let color = request_data.into_inner().color;

    if !color.starts_with('#') || color.len() != 7 {
        return Err(V1ApiError::BadRequest);
    }

    sqlx::query!(
        "UPDATE users SET profile_color = $1 WHERE id = $2",
        color,
        user.id.0
    )
    .execute(&*SQL)
    .await
    .map_err(|_| V1ApiError::InternalError)?;

    Ok(V1ApiResponse(color))
}
