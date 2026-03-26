use crate::SQL;
use crate::utils::s3::{compute_md5, download_object, upload_object};
use image::ImageFormat;
use std::io::Cursor;

pub async fn remove_root_covers() {
    println!("Cleaning up unused covers in root directories...");

    for base in ["covers", "covers_small", "icons"] {
        let bucket = &*crate::S3;

        match bucket
            .list(format!("{}/", base), Some("/".to_string()))
            .await
        {
            Ok(results) => {
                dbg!(&results);
                for res in results {
                    for obj in res.contents {
                        if obj.key.ends_with(".webp")
                            || obj.key.ends_with(".png")
                            || obj.key.ends_with(".jpg")
                        {
                            println!("Deleting unused file: {}", obj.key);
                            let _ = bucket.delete_object(&obj.key).await;
                        }
                    }
                }
            }
            Err(e) => {
                println!("Failed to list {}: {:?}", base, e);
            }
        }
    }
}
