use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use rocket::{get, post, serde::json::Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    SQL,
    routes::api::{
        V1ApiResponse, V1ApiResponseType,
        api_error::V1ApiError,
        api_response::V1ApiResponseTrait,
        v1::guards::{AuthenticatedUser, UserRole},
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
    .ok_or(V1ApiError::NotFound)?;

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
                return V1ApiError::Conflict;
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
    .map_err(|_| V1ApiError::InternalError)?
    .ok_or(V1ApiError::NotFound)?;

    let parsed_hash = PasswordHash::new(&user.password).map_err(|_| V1ApiError::InternalError)?;

    Argon2::default()
        .verify_password(data.password.as_bytes(), &parsed_hash)
        .map_err(|_| V1ApiError::NotAuthorized)?;

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
    .map_err(|_| V1ApiError::NotAuthorized)?;

    Ok(V1ApiResponse(V1AuthResponse { token }))
}

#[derive(Serialize)]
pub struct V1AuthMeResponse {
    id: Id,
    username: String,
    role: UserRole,
}

impl V1ApiResponseTrait for V1AuthMeResponse {}

#[get("/api/v1/users/@me")]
pub async fn me(user: AuthenticatedUser) -> V1ApiResponseType<V1AuthMeResponse> {
    Ok(V1ApiResponse(V1AuthMeResponse {
        id: user.id,
        username: user.username,
        role: user.role,
    }))
}
