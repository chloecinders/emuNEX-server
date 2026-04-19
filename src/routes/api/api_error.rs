use rocket::{
    Response,
    http::Status,
    response::{self, Responder},
    serde::json::Json,
};
use serde::Serialize;

#[allow(dead_code)]
#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum V1ApiError {
    NotAuthorized,
    InvalidToken,
    NotFound,
    InternalError,
    Conflict,
    BadRequest,

    InvalidCredentials,
    InvalidInviteCode,
    UsernameTaken,
    InvalidUsername,
    MissingPermissions,

    RomNotFound,
    SaveNotFound,
    EmulatorNotFound,
    ConsoleNotFound,
    NoIntroEntryNotFound,
    ReportNotFound,
    LibraryItemNotFound,
    SearchSectionNotFound,
    UserNotFound,

    FileNotFound,
    InvalidFile,
    FileTooLarge,

    DatabaseError,
}

impl V1ApiError {
    fn response(&self) -> (Status, &'static str) {
        match self {
            V1ApiError::NotAuthorized => (
                Status::Forbidden,
                "You are not authorized to use this endpoint",
            ),
            V1ApiError::InvalidToken => (
                Status::Unauthorized,
                "Your session has expired or your token is invalid. Please log in again.",
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

            V1ApiError::InvalidCredentials => {
                (Status::Unauthorized, "Incorrect username or password")
            }
            V1ApiError::InvalidInviteCode => (
                Status::BadRequest,
                "The provided invite code does not exist or has already been used",
            ),
            V1ApiError::UsernameTaken => (Status::Conflict, "That username is already taken"),
            V1ApiError::InvalidUsername => (Status::BadRequest, "The provided username is invalid"),
            V1ApiError::MissingPermissions => (
                Status::Forbidden,
                "You do not have permission to perform this action",
            ),

            V1ApiError::RomNotFound => (Status::NotFound, "The requested ROM could not be found"),
            V1ApiError::SaveNotFound => (
                Status::NotFound,
                "The requested save data could not be found",
            ),
            V1ApiError::EmulatorNotFound => (
                Status::NotFound,
                "The requested emulator could not be found",
            ),
            V1ApiError::ConsoleNotFound => {
                (Status::NotFound, "The requested console could not be found")
            }
            V1ApiError::NoIntroEntryNotFound => (
                Status::NotFound,
                "The requested No-Intro entry could not be found",
            ),
            V1ApiError::ReportNotFound => {
                (Status::NotFound, "The requested report could not be found")
            }
            V1ApiError::LibraryItemNotFound => (
                Status::NotFound,
                "The requested library item could not be found",
            ),
            V1ApiError::SearchSectionNotFound => (
                Status::NotFound,
                "The requested search section could not be found",
            ),
            V1ApiError::UserNotFound => (Status::NotFound, "The requested user could not be found"),

            V1ApiError::FileNotFound => (
                Status::NotFound,
                "The requested file could not be found on disk",
            ),
            V1ApiError::InvalidFile => (
                Status::BadRequest,
                "The uploaded file is invalid or corrupted",
            ),
            V1ApiError::FileTooLarge => (
                Status::PayloadTooLarge,
                "The uploaded file exceeds the maximum allowed size",
            ),

            V1ApiError::DatabaseError => (Status::InternalServerError, "A database error occurred"),
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
