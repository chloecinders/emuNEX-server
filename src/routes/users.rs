use rocket::{get, http::Status};
use rocket_dyn_templates::{Template, context};

use crate::{
    SQL,
    routes::api::v1::guards::{AuthenticatedUser, UserRole},
};

#[derive(serde::Serialize)]
pub struct UserView {
    pub id: i32,
    pub username: String,
    pub role: UserRole,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[get("/users")]
pub async fn manage_users(user: AuthenticatedUser) -> Result<Template, Status> {
    if user.role != UserRole::Admin {
        return Err(Status::Unauthorized);
    }

    let users = sqlx::query_as!(
        UserView,
        r#"SELECT id, username, role AS "role: UserRole", created_at FROM users ORDER BY id ASC"#
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|_| Status::InternalServerError)?;

    Ok(Template::render(
        "users",
        context! {
            users: users,
            current_user: user
        },
    ))
}
