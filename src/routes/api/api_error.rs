use rocket::{
    Response,
    http::Status,
    response::{self, Responder},
    serde::json::Json,
};
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum V1ApiError {
    NotAuthorized,
    NotFound,
    InternalError,
    Conflict,
    BadRequest,
}

impl V1ApiError {
    fn response(&self) -> (Status, &'static str) {
        match self {
            V1ApiError::NotAuthorized => (
                Status::Unauthorized,
                "User is not authorized to use this endpoint",
            ),
            V1ApiError::NotFound => (Status::NotFound, "The requested resource was not found"),
            V1ApiError::InternalError => {
                (Status::InternalServerError, "An unexpected error occurred")
            }
            V1ApiError::Conflict => (
                Status::Conflict,
                "The request could not be completed due to a conflict with the current state of the resource.",
            ),
            V1ApiError::BadRequest => (
                Status::BadRequest,
                "The request was invalid or contained malformed data",
            ),
        }
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for V1ApiError {
    fn respond_to(self, req: &'r rocket::Request<'_>) -> response::Result<'static> {
        let (status, message) = self.response();

        let body = serde_json::json!({
            "code": format!("{:?}", self),
            "message": message,
            "success": false,
        });

        Response::build_from(Json(body).respond_to(req)?)
            .status(status)
            .ok()
    }
}
