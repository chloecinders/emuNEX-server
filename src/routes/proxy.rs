use rocket::futures::StreamExt;
use rocket::http::{Header, Status};
use rocket::response::Responder;
use rocket::response::stream::ByteStream;
use rocket::{Request, Response, get, options, response};

use crate::CONFIG;

#[options("/storage/<_file..>")]
pub async fn storage_options<'r>(_file: std::path::PathBuf) -> CorsResponse {
    CorsResponse {}
}

#[get("/storage/<file..>")]
pub async fn storage(file: std::path::PathBuf) -> ByteStream![Vec<u8>] {
    let url = format!("{}/{}", CONFIG.seaweedfs_url, file.display());

    ByteStream! {
        let res = reqwest::get(url).await;

        if let Ok(response) = res {
            let mut stream = response.bytes_stream();

            while let Some(Ok(chunk)) = stream.next().await {
                yield chunk.to_vec();
            }
        }
    }
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
