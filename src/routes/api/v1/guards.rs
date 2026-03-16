use serde::Serialize;

use crate::{SQL, routes::api::V1ApiError, utils::id::Id};

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Moderator,
    User,
}

#[derive(Debug, Serialize)]
pub struct AuthenticatedUser {
    pub id: Id,
    pub username: String,
    pub role: UserRole,
}

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for AuthenticatedUser {
    type Error = V1ApiError;

    async fn from_request(
        req: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let token = req
            .cookies()
            .get("token")
            .map(|c| c.value().to_string())
            .or_else(|| {
                req.headers()
                    .get_one("Authorization")
                    .map(|h| h.to_string())
            });

        match token {
            Some(t) => {
                let user = sqlx::query!(
                    r#"
                    SELECT u.id, u.role as "role: UserRole", u.username
                    FROM users u
                    JOIN user_tokens ut ON u.id = ut.user_id
                    WHERE ut.token = $1
                      AND ut.expires_at > NOW()
                    "#,
                    t
                )
                .fetch_optional(&*SQL)
                .await;

                match user {
                    Ok(Some(row)) => rocket::request::Outcome::Success(AuthenticatedUser {
                        id: Id::new(row.id),
                        username: row.username,
                        role: row.role,
                    }),
                    _ => rocket::request::Outcome::Error((
                        rocket::http::Status::Unauthorized,
                        V1ApiError::NotAuthorized,
                    )),
                }
            }
            None => rocket::request::Outcome::Error((
                rocket::http::Status::Unauthorized,
                V1ApiError::NotAuthorized,
            )),
        }
    }
}
