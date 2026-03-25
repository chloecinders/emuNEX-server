use rocket::http::{Header, Status};
use rocket::response::Responder;
use rocket::{Request, Response, get, options, response};

#[options("/storage/<_file..>")]
pub async fn storage_options<'r>(_file: std::path::PathBuf) -> CorsResponse {
    CorsResponse {}
}

#[get("/storage/<file..>")]
pub async fn storage(file: std::path::PathBuf) -> Result<Vec<u8>, Status> {
    let display_path = file.display().to_string().replace("\\", "/");
    let key = format!("/{}", display_path);

    let is_small_cover = key.starts_with("/covers_small/");
    let is_large_cover = key.starts_with("/covers/");
    let is_icon = key.starts_with("/icons/");

    if (is_small_cover || is_large_cover || is_icon)
        && !key.ends_with(".png")
        && !key.ends_with(".webp")
        && !key.ends_with(".jpg")
        && !key.ends_with(".jpeg")
    {
        return Err(Status::BadRequest);
    }

    match crate::utils::s3::download_object(&key).await {
        Ok(data) => Ok(data),
        Err(_) if is_small_cover || is_large_cover || is_icon => {
            let original_key = if is_small_cover {
                key.replacen("/covers_small/", "/covers/", 1)
            } else if is_icon {
                key.replacen("/icons/", "/covers/", 1)
            } else {
                return Err(Status::NotFound);
            };

            if let Ok(original_data) = crate::utils::s3::download_object(&original_key).await {
                if let Ok(img) = image::load_from_memory(&original_data) {
                    let processed = if is_small_cover {
                        img.resize(150, 225, image::imageops::FilterType::Lanczos3)
                    } else if is_icon {
                        img.resize_to_fill(256, 256, image::imageops::FilterType::Lanczos3)
                    } else {
                        img
                    };

                    let mut buf = std::io::Cursor::new(Vec::new());
                    let format = if key.ends_with(".png") {
                        image::ImageFormat::Png
                    } else if key.ends_with(".webp") {
                        image::ImageFormat::WebP
                    } else {
                        image::ImageFormat::Jpeg
                    };

                    if processed.write_to(&mut buf, format).is_ok() {
                        let raw = buf.into_inner();
                        let _ = crate::utils::s3::upload_object(&key, &raw).await;
                        return Ok(raw);
                    }
                }
            }
            Err(Status::NotFound)
        }
        Err(_) => Err(Status::NotFound),
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
