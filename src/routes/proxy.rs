use rocket::http::{Header, Status};
use rocket::response::Responder;
use rocket::{Request, Response, get, options, response};

#[options("/storage/<_file..>")]
pub async fn storage_options<'r>(_file: std::path::PathBuf) -> CorsResponse {
    CorsResponse {}
}

#[get("/storage/<file..>")]
pub async fn storage(file: std::path::PathBuf) -> Result<Vec<u8>, Status> {
    let key = format!("/{}", file.display());

    crate::utils::s3::download_object(&key)
        .await
        .map_err(|_| Status::NotFound)
}

pub struct CorsResponse;

impl<'r> Responder<'r, 'static> for CorsResponse {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .status(Status::NoContent)
            .header(Header::new("Access-Control-Allow-Origin", "*"))
            .ok()
    }
}
