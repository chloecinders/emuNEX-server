use crate::SQL;
use crate::utils::s3::{compute_md5, download_object, upload_object};
use image::ImageFormat;
use std::io::Cursor;

pub async fn migrate_covers() {
    println!("Checking if any covers need migration...");

    let roms = match sqlx::query!("SELECT id, image_hash FROM roms")
        .fetch_all(&*SQL)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            println!("Failed to fetch roms for migration: {:?}", e);
            return;
        }
    };

    let mut migrated_count = 0;

    for rom in roms {
        let path = rom.image_hash;

        if path.len() == 32 && path.chars().all(|c| c.is_ascii_hexdigit()) {
            continue;
        }

        println!("Migrating cover for ROM {}: {}", rom.id, path);

        let s3_path = if path.starts_with('/') {
            path.clone()
        } else {
            format!("/{}", path)
        };

        match download_object(&s3_path).await {
            Ok(bytes) => {
                let is_webp = s3_path.to_lowercase().ends_with(".webp");

                let (hash, final_bytes) = if is_webp {
                    let hash = compute_md5(&bytes);
                    (hash, bytes)
                } else {
                    let img = match image::load_from_memory(&bytes) {
                        Ok(i) => i,
                        Err(e) => {
                            println!("Failed to load cover for ROM {}: {}", rom.id, e);
                            continue;
                        }
                    };

                    let mut buf = Cursor::new(Vec::new());
                    if let Err(e) = img.write_to(&mut buf, ImageFormat::WebP) {
                        println!("Failed to encode WebP for ROM {}: {}", rom.id, e);
                        continue;
                    }

                    let webp_bytes = buf.into_inner();
                    let hash = compute_md5(&webp_bytes);
                    (hash, webp_bytes)
                };

                let new_s3_path = format!("/covers/{}.webp", hash);

                if let Err(e) = upload_object(&new_s3_path, &final_bytes).await {
                    println!("Failed to upload migrated cover for ROM {}: {}", rom.id, e);
                    continue;
                }

                if let Err(e) = sqlx::query!(
                    "UPDATE roms SET image_hash = $1 WHERE id = $2",
                    hash,
                    rom.id
                )
                .execute(&*SQL)
                .await
                {
                    println!("Failed to update database for ROM {}: {:?}", rom.id, e);
                } else {
                    migrated_count += 1;
                }
            }
            Err(e) => {
                println!("Failed to download cover for ROM {}: {}", rom.id, e);
            }
        }
    }

    if migrated_count > 0 {
        println!("Successfully migrated {} covers.", migrated_count);
    } else {
        println!("No covers needed migration.");
    }
}
