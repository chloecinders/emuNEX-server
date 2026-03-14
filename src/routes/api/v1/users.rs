use rocket::{put, serde::json::Json};

use crate::{
    SQL,
    routes::api::{
        V1ApiError, V1ApiResponse, V1ApiResponseType,
        v1::guards::{AuthenticatedUser, UserRole},
    },
};

#[derive(serde::Deserialize)]
pub struct UpdateUserRequest {
    pub username: String,
    pub role: String,
}

#[put("/api/v1/users/<id>", format = "json", data = "<data>")]
pub async fn update_user(
    id: i32,
    data: Json<UpdateUserRequest>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<i32> {
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

    Ok(V1ApiResponse(id))
}
