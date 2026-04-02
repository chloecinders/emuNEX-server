use rocket::{get, patch, post, delete, serde::json::Json};
use serde::Serialize;
use tracing::error;

use crate::{
    SQL,
    routes::api::{
        V1ApiError, V1ApiResponse, V1ApiResponseType,
        api_response::V1ApiResponseTrait,
        v1::guards::{AuthenticatedUser, UserRole},
    },
};

#[derive(serde::Deserialize)]
pub struct V1CreateReportRequest {
    pub rom_id: String,
    pub report_type: String,
    pub description: String,
}

#[post("/api/v1/reports", format = "json", data = "<data>")]
pub async fn create_report(
    data: Json<V1CreateReportRequest>,
    user: AuthenticatedUser,
) -> V1ApiResponseType<String> {
    let report_id = crate::utils::snowflake::next_id().to_string();

    sqlx::query!(
        "INSERT INTO rom_reports (id, rom_id, user_id, report_type, description, status)
         VALUES ($1, $2, $3, $4, $5, 'open')",
        report_id,
        data.rom_id,
        user.id.value(),
        data.report_type,
        data.description
    )
    .execute(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to insert rom report: {:?}", e);
        V1ApiError::DatabaseError
    })?;

    Ok(V1ApiResponse(report_id))
}

#[derive(Serialize, sqlx::FromRow)]
pub struct V1ReportListResponse {
    pub id: String,
    pub rom_id: String,
    pub rom_title: String,
    pub user_id: String,
    pub username: String,
    pub report_type: String,
    pub description: String,
    pub status: String,
    pub claimed_by: Option<String>,
    pub claimed_by_username: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
impl V1ApiResponseTrait for Vec<V1ReportListResponse> {}

#[get("/api/v1/reports")]
pub async fn get_reports(user: AuthenticatedUser) -> V1ApiResponseType<Vec<V1ReportListResponse>> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::MissingPermissions);
    }

    let reports = sqlx::query_as!(
        V1ReportListResponse,
        r#"SELECT rep.id, rep.rom_id, r.title as "rom_title!", rep.user_id::TEXT as "user_id!", u.username as "username!", rep.report_type, rep.description, rep.status, rep.claimed_by::TEXT as "claimed_by?", cu.username as "claimed_by_username?", rep.created_at
         FROM rom_reports rep
         INNER JOIN roms r ON rep.rom_id = r.id
         INNER JOIN users u ON rep.user_id = u.id
         LEFT JOIN users cu ON rep.claimed_by = cu.id
         ORDER BY rep.created_at ASC"#
    )
    .fetch_all(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to fetch rom reports: {:?}", e);
        V1ApiError::DatabaseError
    })?;

    Ok(V1ApiResponse(reports))
}

#[patch("/api/v1/reports/<id>/resolve")]
pub async fn resolve_report(id: String, user: AuthenticatedUser) -> V1ApiResponseType<()> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::MissingPermissions);
    }

    sqlx::query!(
        "UPDATE rom_reports SET status = 'resolved' WHERE id = $1",
        id
    )
    .execute(&*SQL)
    .await
    .map_err(|e| {
        error!("Failed to resolve report id {}: {:?}", id, e);
        V1ApiError::DatabaseError
    })?;

    Ok(V1ApiResponse(()))
}

#[patch("/api/v1/reports/<id>/claim")]
pub async fn claim_report(id: String, user: AuthenticatedUser) -> V1ApiResponseType<()> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::MissingPermissions);
    }

    let report_option = sqlx::query!("SELECT claimed_by FROM rom_reports WHERE id = $1 AND status = 'open'", id)
        .fetch_optional(&*SQL)
        .await
        .map_err(|_| V1ApiError::DatabaseError)?;

    if let Some(report) = report_option {
        if report.claimed_by.is_some() && report.claimed_by != Some(user.id.0) {
            return Err(V1ApiError::Conflict);
        }
    } else {
        return Err(V1ApiError::ReportNotFound);
    }

    sqlx::query!(
        "UPDATE rom_reports SET claimed_by = $1 WHERE id = $2",
        user.id.0,
        id
    )
    .execute(&*SQL)
    .await
    .map_err(|_| V1ApiError::DatabaseError)?;

    Ok(V1ApiResponse(()))
}

#[patch("/api/v1/reports/<id>/unclaim")]
pub async fn unclaim_report(id: String, user: AuthenticatedUser) -> V1ApiResponseType<()> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(V1ApiError::MissingPermissions);
    }

    let report = sqlx::query!("SELECT claimed_by FROM rom_reports WHERE id = $1", id)
        .fetch_optional(&*SQL)
        .await
        .map_err(|_| V1ApiError::DatabaseError)?
        .ok_or(V1ApiError::ReportNotFound)?;

    if report.claimed_by != Some(user.id.0) && user.role != UserRole::Admin {
        return Err(V1ApiError::MissingPermissions);
    }

    sqlx::query!("UPDATE rom_reports SET claimed_by = NULL WHERE id = $1", id)
        .execute(&*SQL)
        .await
        .map_err(|_| V1ApiError::DatabaseError)?;

    Ok(V1ApiResponse(()))
}

#[delete("/api/v1/reports/<id>")]
pub async fn delete_report(id: String, user: AuthenticatedUser) -> V1ApiResponseType<()> {
    if user.role != UserRole::Admin {
        return Err(V1ApiError::MissingPermissions);
    }

    let deleted = sqlx::query!("DELETE FROM rom_reports WHERE id = $1", id)
        .execute(&*SQL)
        .await
        .map_err(|_| V1ApiError::DatabaseError)?;

    if deleted.rows_affected() == 0 {
        return Err(V1ApiError::ReportNotFound);
    }

    Ok(V1ApiResponse(()))
}

