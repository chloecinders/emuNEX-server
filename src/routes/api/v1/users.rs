use rocket::{put, serde::json::Json};

use crate::{
    SQL,
    routes::api::{
        V1ApiError, V1ApiResponse, V1ApiResponseType,
        api_response::V1ApiResponseTrait,
        v1::guards::{AuthenticatedUser, UserRole},
    },
    utils::id::Id,
};

#[derive(serde::Deserialize)]
pub struct V1UpdateUserRequest {
    pub username: String,
    pub role: String,
}

#[derive(serde::Serialize)]
pub struct V1UserResponse {
    pub id: Id,
    pub username: String,
    pub role: UserRole,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl V1ApiResponseTrait for Vec<V1UserResponse> {}

#[rocket::get("/api/v1/users")]
pub async fn get_users(user: AuthenticatedUser) -> V1ApiResponseType<Vec<V1UserResponse>> {
    if user.role != UserRole::Admin {
        return Err(V1ApiError::NotAuthorized);
    }

    let users = sqlx::query_as!(
        V1UserResponse,
        r#"SELECT id AS "id: Id", username, role AS "role: UserRole", created_at FROM users ORDER BY id ASC"#
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|_| V1ApiError::InternalError)?;

    Ok(V1ApiResponse(users))
}


#[put("/api/v1/users/<id>", format = "json", data = "<data>")]
pub async fn update_user(
    id: i64,
    data: Json<V1UpdateUserRequest>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<Id> {
    if user.role != UserRole::Admin {
        return Err(V1ApiError::NotAuthorized);
    }

    sqlx::query!(
        r#"UPDATE users
         SET username = $1, role = $2::text::user_role, updated_at = CURRENT_TIMESTAMP
         WHERE id = $3"#,
        data.username,
        data.role,
        id
    )
    .execute(&*SQL)
    .await
    .map_err(|_| V1ApiError::InternalError)?;

    Ok(V1ApiResponse(Id::new(id)))
}

#[derive(serde::Serialize)]
pub struct V1InviteResponse {
    pub code: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub used_by_username: Option<String>,
    pub used_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl V1ApiResponseTrait for Vec<V1InviteResponse> {}

#[rocket::get("/api/v1/invites")]
pub async fn get_invites(user: AuthenticatedUser) -> V1ApiResponseType<Vec<V1InviteResponse>> {
    if user.role != UserRole::Admin {
        return Err(V1ApiError::NotAuthorized);
    }

    let invites = sqlx::query!(
        r#"
        SELECT i.code, i.created_at, u.username as used_by_username, i.used_at
        FROM invite_codes i
        LEFT JOIN users u ON i.used_by = u.id
        ORDER BY i.created_at DESC
        "#
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|_| V1ApiError::InternalError)?;

    let res = invites
        .into_iter()
        .map(|i| V1InviteResponse {
            code: i.code,
            created_at: i.created_at.expect("created_at should exist"),
            used_by_username: Some(i.used_by_username),
            used_at: i.used_at,
        })
        .collect();

    Ok(V1ApiResponse(res))
}

#[rocket::post("/api/v1/invites")]
pub async fn create_invite(user: AuthenticatedUser) -> V1ApiResponseType<String> {
    if user.role != UserRole::Admin {
        return Err(V1ApiError::NotAuthorized);
    }

    let code = uuid::Uuid::new_v4().to_string();

    sqlx::query!(
        "INSERT INTO invite_codes (code, created_by) VALUES ($1, $2)",
        code,
        user.id.0
    )
    .execute(&*SQL)
    .await
    .map_err(|_| V1ApiError::InternalError)?;

    Ok(V1ApiResponse(code))
}

#[rocket::delete("/api/v1/invites/<code>")]
pub async fn delete_invite(code: String, user: AuthenticatedUser) -> V1ApiResponseType<String> {
    if user.role != UserRole::Admin {
        return Err(V1ApiError::NotAuthorized);
    }

    let deleted = sqlx::query!(
        "DELETE FROM invite_codes WHERE code = $1 AND used_by IS NULL",
        code
    )
    .execute(&*SQL)
    .await
    .map_err(|_| V1ApiError::InternalError)?;

    if deleted.rows_affected() == 0 {
        return Err(V1ApiError::NotFound);
    }

    Ok(V1ApiResponse(code))
}
