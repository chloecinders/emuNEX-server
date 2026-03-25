use rocket::{delete, get, post, put, serde::json::Json};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    SQL,
    routes::api::{
        V1ApiError, V1ApiResponse, V1ApiResponseType,
        api_response::V1ApiResponseTrait,
        v1::guards::{AuthenticatedUser, UserRole},
    },
};

#[derive(Serialize, sqlx::FromRow)]
pub struct V1SearchSectionResponse {
    pub id: i64,
    pub title: String,
    pub section_type: String,
    pub smart_filter: Option<String>,
    pub filter_value: Option<String>,
    pub order_index: i32,
    pub roms: Option<Vec<String>>,
}
impl V1ApiResponseTrait for Vec<V1SearchSectionResponse> {}
impl V1ApiResponseTrait for V1SearchSectionResponse {}

#[get("/api/v1/search_sections")]
pub async fn get_search_sections(
    _user: AuthenticatedUser,
) -> V1ApiResponseType<Vec<V1SearchSectionResponse>> {
        let sections = sqlx::query!(
            "SELECT id, title, section_type, smart_filter, filter_value, order_index FROM search_sections ORDER BY order_index ASC"
        )
        .fetch_all(&*SQL)
        .await
        .map_err(|e| {
            error!("Failed to fetch search sections: {:?}", e);
            V1ApiError::InternalError
        })?;

        let mut out = Vec::new();

        for s in sections {
            let mut custom_roms = None;
            if s.section_type == "custom" {
                let rom_ids = sqlx::query!(
                    "SELECT rom_id FROM search_section_roms WHERE section_id = $1 ORDER BY order_index ASC",
                    s.id
                )
                .fetch_all(&*SQL)
                .await
                .map_err(|e| {
                    error!("Failed to fetch custom section roms: {:?}", e);
                    V1ApiError::InternalError
                })?;
                custom_roms = Some(rom_ids.into_iter().map(|r| r.rom_id).collect());
            }

            out.push(V1SearchSectionResponse {
                id: s.id,
                title: s.title,
                section_type: s.section_type,
                smart_filter: s.smart_filter,
                filter_value: s.filter_value,
                order_index: s.order_index,
                roms: custom_roms,
            });
        }

        Ok(V1ApiResponse(out))
}

#[derive(Deserialize)]
pub struct V1SearchSectionCreateRequest {
    pub title: String,
    pub section_type: String,
    pub smart_filter: Option<String>,
    pub filter_value: Option<String>,
    pub order_index: i32,
    pub roms: Option<Vec<String>>,
}

#[post("/api/v1/search_sections", format = "json", data = "<data>")]
pub async fn create_search_section(
    data: Json<V1SearchSectionCreateRequest>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<String> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::NotAuthorized);
    }

    let section_id = crate::utils::snowflake::next_id();

    let mut tx = SQL.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {:?}", e);
        V1ApiError::InternalError
    })?;

    sqlx::query!(
        "INSERT INTO search_sections (id, title, section_type, smart_filter, filter_value, order_index) VALUES ($1, $2, $3, $4, $5, $6)",
        section_id,
        data.title,
        data.section_type,
        data.smart_filter,
        data.filter_value,
        data.order_index
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to insert search section: {:?}", e);
        V1ApiError::InternalError
    })?;

    if data.section_type == "custom" {
        if let Some(roms) = &data.roms {
            for (idx, rom_id) in roms.iter().enumerate() {
                sqlx::query!(
                    "INSERT INTO search_section_roms (section_id, rom_id, order_index) VALUES ($1, $2, $3)",
                    section_id,
                    rom_id,
                    idx as i32
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    error!("Failed to insert section roms: {:?}", e);
                    V1ApiError::InternalError
                })?;
            }
        }
    }

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {:?}", e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(section_id.to_string()))
}

#[derive(Deserialize)]
pub struct V1SearchSectionUpdateRequest {
    pub title: String,
    pub section_type: String,
    pub smart_filter: Option<String>,
    pub filter_value: Option<String>,
    pub order_index: i32,
    pub roms: Option<Vec<String>>,
}

#[put("/api/v1/search_sections/<id>", format = "json", data = "<data>")]
pub async fn update_search_section(
    id: i64,
    data: Json<V1SearchSectionUpdateRequest>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<String> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::NotAuthorized);
    }

    let mut tx = SQL.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {:?}", e);
        V1ApiError::InternalError
    })?;

    sqlx::query!(
        "UPDATE search_sections SET title = $1, section_type = $2, smart_filter = $3, filter_value = $4, order_index = $5 WHERE id = $6",
        data.title,
        data.section_type,
        data.smart_filter,
        data.filter_value,
        data.order_index,
        id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        error!("Failed to update search section: {:?}", e);
        V1ApiError::InternalError
    })?;

    if data.section_type == "custom" {
        sqlx::query!("DELETE FROM search_section_roms WHERE section_id = $1", id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!("Failed to clear old section roms: {:?}", e);
                V1ApiError::InternalError
            })?;

        if let Some(roms) = &data.roms {
            for (idx, rom_id) in roms.iter().enumerate() {
                sqlx::query!(
                    "INSERT INTO search_section_roms (section_id, rom_id, order_index) VALUES ($1, $2, $3)",
                    id,
                    rom_id,
                    idx as i32
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    error!("Failed to insert section roms: {:?}", e);
                    V1ApiError::InternalError
                })?;
            }
        }
    } else {
        sqlx::query!("DELETE FROM search_section_roms WHERE section_id = $1", id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!("Failed to clear section roms (type changed from custom): {:?}", e);
                V1ApiError::InternalError
            })?;
    }

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {:?}", e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(id.to_string()))
}

#[derive(Deserialize)]
pub struct V1SearchSectionOrderUpdateRequest {
    // Array of (id, order_index)
    pub updates: Vec<(i64, i32)>,
}

#[put("/api/v1/search_sections/order", format = "json", data = "<data>")]
pub async fn update_search_sections_order(
    data: Json<V1SearchSectionOrderUpdateRequest>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<()> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::NotAuthorized);
    }
    
    let mut tx = SQL.begin().await.map_err(|e| {
        error!("Failed to begin transaction: {:?}", e);
        V1ApiError::InternalError
    })?;

    for (id, order) in &data.updates {
        sqlx::query!("UPDATE search_sections SET order_index = $1 WHERE id = $2", order, id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!("Failed to update order for {}: {:?}", id, e);
                V1ApiError::InternalError
            })?;
    }

    tx.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {:?}", e);
        V1ApiError::InternalError
    })?;

    Ok(V1ApiResponse(()))
}

#[delete("/api/v1/search_sections/<id>")]
pub async fn delete_search_section(
    id: i64,
    user: AuthenticatedUser,
) -> V1ApiResponseType<()> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::NotAuthorized);
    }

    sqlx::query!("DELETE FROM search_sections WHERE id = $1", id)
        .execute(&*SQL)
        .await
        .map_err(|e| {
            error!("Failed to delete search section {}: {:?}", id, e);
            V1ApiError::InternalError
        })?;

    Ok(V1ApiResponse(()))
}
