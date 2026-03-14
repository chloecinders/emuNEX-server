use rocket::{
    form::{Form, FromForm},
    get, post,
};
use serde::Serialize;

use crate::{
    SQL,
    routes::api::{
        V1ApiError, V1ApiResponse, V1ApiResponseType,
        api_response::V1ApiResponseTrait,
        v1::guards::{AuthenticatedUser, UserRole},
    },
};

#[derive(Serialize, sqlx::FromRow)]
pub struct V1ConsoleResponse {
    pub name: String,
    pub card_color: Option<String>,
}
impl V1ApiResponseTrait for Vec<V1ConsoleResponse> {}

#[derive(serde::Deserialize, FromForm)]
pub struct ConsoleInsert {
    pub name: String,
    pub card_color: Option<String>,
}

#[get("/api/v1/roms/consoles")]
pub async fn get_consoles(_user: AuthenticatedUser) -> V1ApiResponseType<Vec<V1ConsoleResponse>> {
    let consoles = sqlx::query_as!(
        V1ConsoleResponse,
        "SELECT name, card_color FROM consoles ORDER BY name ASC"
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(consoles))
}

#[post("/api/v1/roms/consoles", data = "<data>")]
pub async fn upload_console(
    data: Form<ConsoleInsert>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<i32> {
    if user.role != UserRole::Admin {
        return Err(V1ApiError::NotAuthorized);
    }

    let rec = sqlx::query!(
        "INSERT INTO consoles (name, card_color)
         VALUES ($1, $2)
         ON CONFLICT (name) DO UPDATE SET
            card_color = EXCLUDED.card_color
         RETURNING id",
        data.name,
        data.card_color
    )
    .fetch_one(&*SQL)
    .await
    .map_err(|_| V1ApiError::InternalError)?;

    Ok(V1ApiResponse(rec.id))
}
