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

    if key.starts_with("/saves/") {
        return Err(Status::Forbidden);
    }

    let is_small_cover = key.starts_with("/covers_small/");
    let is_large_cover = key.starts_with("/covers/");
    let is_icon = key.starts_with("/icons/");

    let extensions = [".png", ".webp", ".jpg", ".jpeg"];
    let mut current_extension = None;
    for ext in extensions {
        if key.ends_with(ext) {
            current_extension = Some(ext);
            break;
        }
    }

    if (is_small_cover || is_large_cover || is_icon) && current_extension.is_none() {
        return Err(Status::BadRequest);
    }

    if let Ok(data) = crate::utils::s3::download_object(&key).await {
        return Ok(data);
    }

    if is_small_cover || is_large_cover || is_icon {
        let base_key = if is_small_cover {
            key.replacen("/covers_small/", "/covers/", 1)
        } else if is_icon {
            key.replacen("/icons/", "/covers/", 1)
        } else {
            key.clone()
        };

        let base_key_no_ext = match current_extension {
            Some(ext) => base_key.strip_suffix(ext).unwrap_or(&base_key).to_string(),
            None => base_key,
        };

        for ext in extensions {
            let original_key = format!("{}{}", base_key_no_ext, ext);

            if original_key == key {
                continue;
            }

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
        }
    }

    Err(Status::NotFound)
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
