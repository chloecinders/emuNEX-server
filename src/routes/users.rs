use rocket::{get, http::Status};
use rocket_dyn_templates::{Template, context};

use crate::{
    SQL,
    routes::api::v1::guards::{AuthenticatedUser, UserRole},
    utils::id::Id,
};

#[derive(serde::Serialize)]
pub struct UserView {
    pub id: Id,
    pub username: String,
    pub role: UserRole,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(serde::Serialize)]
pub struct InviteView {
    pub code: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub used_by_username: Option<String>,
    pub used_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[get("/users")]
pub async fn manage_users(user: AuthenticatedUser) -> Result<Template, Status> {
    if user.role != UserRole::Admin {
        return Err(Status::Unauthorized);
    }

    let users = sqlx::query_as!(
        UserView,
        r#"SELECT id AS "id: Id", username, role AS "role: UserRole", created_at FROM users ORDER BY id ASC"#
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| {
        eprintln!("Error fetching users: {:?}", e);
        Status::InternalServerError
    })?;

    let invites = sqlx::query_as!(
        InviteView,
        r#"
        SELECT i.code, i.created_at, u.username as "used_by_username?", i.used_at
        FROM invite_codes i
        LEFT JOIN users u ON i.used_by = u.id
        ORDER BY i.created_at DESC
        "#
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| {
        eprintln!("Error fetching invites: {:?}", e);
        Status::InternalServerError
    })?;

    Ok(Template::render(
        "users",
        context! {
            users: users,
            invites: invites,
            current_user: user
        },
    ))
}
