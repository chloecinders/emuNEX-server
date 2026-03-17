use rocket::{delete, get, post, put, serde::json::Json};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    SQL,
    routes::api::{
        V1ApiError, V1ApiResponse, V1ApiResponseType, api_response::V1ApiResponseTrait,
        v1::guards::AuthenticatedUser, v1::roms::V1RomListResponse,
    },
    utils::{id::Id, snowflake::next_id},
};

#[derive(Serialize)]
pub struct V1ShelfResponse {
    pub id: Id,
    pub name: String,
    pub sort_order: i32,
    pub games: Vec<V1RomListResponse>,
}

impl V1ApiResponseTrait for Vec<V1ShelfResponse> {}

#[get("/api/v1/library/shelves")]
pub async fn get_shelves(user: AuthenticatedUser) -> V1ApiResponseType<Vec<V1ShelfResponse>> {
    let shelves_records = sqlx::query!(
        "SELECT id, name, sort_order FROM library_shelves WHERE user_id = $1 ORDER BY sort_order ASC, name ASC",
        user.id.value()
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| { error!("{:?}", e); V1ApiError::InternalError })?;

    let mut response = Vec::new();

    for shelf in shelves_records {
        let games = sqlx::query_as!(
            V1RomListResponse,
            "SELECT r.id, r.title, r.image_path, r.console, r.category, r.release_year, r.region, r.serial
             FROM roms r
             JOIN shelf_roms sr ON r.id = sr.rom_id
             WHERE sr.shelf_id = $1
             ORDER BY sr.sort_order ASC, r.title ASC",
            shelf.id
        )
        .fetch_all(&*SQL)
        .await
        .map_err(|e| {
            error!("{:?}", e);
            V1ApiError::InternalError
        })?;

        response.push(V1ShelfResponse {
            id: Id::new(shelf.id),
            name: shelf.name,
            sort_order: shelf.sort_order,
            games,
        });
    }

    Ok(V1ApiResponse(response))
}

#[derive(Deserialize)]
pub struct V1ShelfCreate {
    pub name: String,
}

#[post("/api/v1/library/shelves", data = "<data>")]
pub async fn create_shelf(
    data: Json<V1ShelfCreate>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<Id> {
    let id = next_id();

    sqlx::query!(
        "INSERT INTO library_shelves (id, user_id, name, sort_order)
         VALUES ($1, $2, $3, (SELECT COALESCE(MAX(sort_order), 0) + 1 FROM library_shelves WHERE user_id = $2))",
        id,
        user.id.value(),
        data.name
    )
    .execute(&*SQL)
    .await
    .map_err(|e| { error!("{:?}", e); V1ApiError::InternalError })?;

    Ok(V1ApiResponse(Id::new(id)))
}

#[derive(Deserialize)]
pub struct V1ShelfUpdate {
    pub name: Option<String>,
    pub sort_order: Option<i32>,
}

#[put("/api/v1/library/shelves/<id>", data = "<data>")]
pub async fn update_shelf(
    id: i64,
    data: Json<V1ShelfUpdate>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<()> {
    let _shelf = sqlx::query!(
        "SELECT id FROM library_shelves WHERE id = $1 AND user_id = $2",
        id,
        user.id.value()
    )
    .fetch_optional(&*SQL)
    .await
    .map_err(|e| {
        error!("{:?}", e);
        V1ApiError::InternalError
    })?
    .ok_or(V1ApiError::NotFound)?;

    if let Some(name) = &data.name {
        sqlx::query!(
            "UPDATE library_shelves SET name = $1 WHERE id = $2",
            name,
            id
        )
        .execute(&*SQL)
        .await
        .map_err(|e| {
            error!("{:?}", e);
            V1ApiError::InternalError
        })?;
    }

    if let Some(sort_order) = data.sort_order {
        sqlx::query!(
            "UPDATE library_shelves SET sort_order = $1 WHERE id = $2",
            sort_order,
            id
        )
        .execute(&*SQL)
        .await
        .map_err(|e| {
            error!("{:?}", e);
            V1ApiError::InternalError
        })?;
    }

    Ok(V1ApiResponse(()))
}

#[delete("/api/v1/library/shelves/<id>")]
pub async fn delete_shelf(id: i64, user: AuthenticatedUser) -> V1ApiResponseType<()> {
    let result = sqlx::query!(
        "DELETE FROM library_shelves WHERE id = $1 AND user_id = $2",
        id,
        user.id.value()
    )
    .execute(&*SQL)
    .await
    .map_err(|e| {
        error!("{:?}", e);
        V1ApiError::InternalError
    })?;

    if result.rows_affected() == 0 {
        return Err(V1ApiError::NotFound);
    }

    Ok(V1ApiResponse(()))
}

#[post("/api/v1/library/shelves/<id>/roms/<rom_id>")]
pub async fn add_rom_to_shelf(
    id: i64,
    rom_id: &str,
    user: AuthenticatedUser,
) -> V1ApiResponseType<()> {
    sqlx::query!(
        "SELECT id FROM library_shelves WHERE id = $1 AND user_id = $2",
        id,
        user.id.value()
    )
    .fetch_optional(&*SQL)
    .await
    .map_err(|e| {
        error!("{:?}", e);
        V1ApiError::InternalError
    })?
    .ok_or(V1ApiError::NotFound)?;

    sqlx::query!(
        "INSERT INTO shelf_roms (shelf_id, rom_id, sort_order)
         VALUES ($1, $2, (SELECT COALESCE(MAX(sort_order), 0) + 1 FROM shelf_roms WHERE shelf_id = $1))
         ON CONFLICT DO NOTHING",
        id,
        rom_id
    )
    .execute(&*SQL)
    .await
    .map_err(|e| { error!("{:?}", e); V1ApiError::InternalError })?;

    Ok(V1ApiResponse(()))
}

#[delete("/api/v1/library/shelves/<id>/roms/<rom_id>")]
pub async fn remove_rom_from_shelf(
    id: i64,
    rom_id: &str,
    user: AuthenticatedUser,
) -> V1ApiResponseType<()> {
    sqlx::query!(
        "SELECT id FROM library_shelves WHERE id = $1 AND user_id = $2",
        id,
        user.id.value()
    )
    .fetch_optional(&*SQL)
    .await
    .map_err(|e| {
        error!("{:?}", e);
        V1ApiError::InternalError
    })?
    .ok_or(V1ApiError::NotFound)?;

    sqlx::query!(
        "DELETE FROM shelf_roms WHERE shelf_id = $1 AND rom_id = $2",
        id,
        rom_id
    )
    .execute(&*SQL)
    .await
    .map_err(|e| {
        error!("{:?}", e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(()))
}

#[derive(Deserialize)]
pub struct V1ShelfRomOrderUpdate {
    pub rom_ids: Vec<String>,
}

#[put("/api/v1/library/shelves/<id>/roms/order", data = "<data>")]
pub async fn update_rom_order(
    id: i64,
    data: Json<V1ShelfRomOrderUpdate>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<()> {
    sqlx::query!(
        "SELECT id FROM library_shelves WHERE id = $1 AND user_id = $2",
        id,
        user.id.value()
    )
    .fetch_optional(&*SQL)
    .await
    .map_err(|e| {
        error!("{:?}", e);
        V1ApiError::InternalError
    })?
    .ok_or(V1ApiError::NotFound)?;

    let mut tx = SQL.begin().await.map_err(|e| {
        error!("{:?}", e);
        V1ApiError::InternalError
    })?;

    for (index, rom_id) in data.rom_ids.iter().enumerate() {
        sqlx::query!(
            "UPDATE shelf_roms SET sort_order = $1 WHERE shelf_id = $2 AND rom_id = $3",
            index as i32,
            id,
            rom_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            error!("{:?}", e);
            V1ApiError::InternalError
        })?;
    }

    tx.commit().await.map_err(|e| {
        error!("{:?}", e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(()))
}
