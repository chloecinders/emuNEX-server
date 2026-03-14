use rocket::{
    Response,
    http::Status,
    response::{self, Responder},
    serde::json::Json,
};

pub struct V1ApiResponse<T>(pub T);

pub trait V1ApiResponseTrait {
    fn status() -> Status {
        Status::Ok
    }
}

impl<'r, T> Responder<'r, 'static> for V1ApiResponse<T>
where
    T: V1ApiResponseTrait + serde::Serialize + Send + 'static,
{
    fn respond_to(self, req: &'r rocket::Request<'_>) -> response::Result<'static> {
        let body = serde_json::json!({
            "success": true,
            "data": self.0
        });

        Response::build_from(Json(body).respond_to(req)?)
            .status(T::status())
            .ok()
    }
}

impl V1ApiResponseTrait for i32 {}
impl V1ApiResponseTrait for () {}
