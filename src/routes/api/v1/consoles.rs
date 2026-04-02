use rocket::{
    delete,
    form::{Form, FromForm},
    get, post, put,
    serde::json::Json,
};
use serde::Serialize;

use crate::{
    SQL,
    routes::api::{
        V1ApiError, V1ApiResponse, V1ApiResponseType,
        api_response::V1ApiResponseTrait,
        v1::guards::{AuthenticatedUser, UserRole},
    },
    utils::{id::Id, snowflake::next_id},
};

#[derive(Serialize, sqlx::FromRow)]
pub struct V1ConsoleResponse {
    pub id: Id,
    pub name: String,
    pub card_color: Option<String>,
}
impl V1ApiResponseTrait for Vec<V1ConsoleResponse> {}

#[get("/api/v1/consoles")]
pub async fn get_consoles(_user: AuthenticatedUser) -> V1ApiResponseType<Vec<V1ConsoleResponse>> {
    let consoles = sqlx::query_as!(
        V1ConsoleResponse,
        "SELECT id, name, card_color FROM consoles ORDER BY name ASC"
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| {
        eprintln!("Database error: {:?}", e);
        V1ApiError::DatabaseError
    })?;

    Ok(V1ApiResponse(consoles))
}

#[derive(serde::Deserialize, FromForm)]
pub struct V1ConsoleInsert {
    pub name: String,
    pub card_color: Option<String>,
}

#[post("/api/v1/consoles", data = "<data>")]
pub async fn upload_console(
    data: Form<V1ConsoleInsert>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<Id> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::MissingPermissions);
    }

    let id = next_id();

    sqlx::query!(
        "INSERT INTO consoles (id, name, card_color)
         VALUES ($1, $2, $3)
         ON CONFLICT (name) DO UPDATE SET
            card_color = EXCLUDED.card_color",
        id,
        data.name,
        data.card_color
    )
    .execute(&*SQL)
    .await
    .map_err(|_| V1ApiError::DatabaseError)?;

    Ok(V1ApiResponse(Id::new(id)))
}

#[derive(serde::Deserialize)]
pub struct V1ConsoleUpdateRequest {
    pub card_color: Option<String>,
}

#[put("/api/v1/consoles/<name>", format = "json", data = "<data>")]
pub async fn update_console_metadata(
    name: String,
    data: Json<V1ConsoleUpdateRequest>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<String> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::MissingPermissions);
    }

    sqlx::query!(
        "UPDATE consoles SET card_color = $1 WHERE name = $2",
        data.card_color,
        name
    )
    .execute(&*SQL)
    .await
    .map_err(|_| V1ApiError::DatabaseError)?;

    Ok(V1ApiResponse(name))
}

#[delete("/api/v1/consoles/<name>")]
pub async fn delete_console(name: String, user: AuthenticatedUser) -> V1ApiResponseType<()> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::MissingPermissions);
    }

    sqlx::query!("DELETE FROM consoles WHERE name = $1", name)
        .execute(&*SQL)
        .await
        .map_err(|e| {
            eprintln!("Failed to delete console {}: {:?}", name, e);
            V1ApiError::DatabaseError
        })?;

    Ok(V1ApiResponse(()))
}
