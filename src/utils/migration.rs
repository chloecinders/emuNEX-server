use crate::SQL;
use crate::utils::s3::{compute_md5, download_object, upload_object};
use image::ImageFormat;
use std::io::Cursor;

pub async fn migrate_covers() {
    println!("Checking if any covers need migration...");

    let roms = match sqlx::query!("SELECT id, console, image_hash FROM roms")
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
        let hash = &rom.image_hash;

        let new_cover = format!("covers/{}/{}/{}.webp", rom.console, rom.id, hash);
        let old_cover = format!("covers/{}.webp", hash);

        let new_small = format!("covers_small/{}/{}/{}.webp", rom.console, rom.id, hash);
        let old_small = format!("covers_small/{}.webp", hash);

        let new_icon = format!("icons/{}/{}/{}.webp", rom.console, rom.id, hash);
        let old_icon = format!("icons/{}.webp", hash);

        let mut moved_any = false;
        let mut source_bytes = None;

        if let Ok(bytes) = crate::utils::s3::download_object(&old_cover).await {
            source_bytes = Some(bytes);
        } else {
            let shared_roms = sqlx::query!(
                "SELECT id, console FROM roms WHERE image_hash = $1 AND id != $2",
                rom.image_hash,
                rom.id
            )
            .fetch_all(&*SQL)
            .await
            .unwrap_or_default();

            for s in shared_roms {
                let other_new_path =
                    format!("covers/{}/{}/{}.webp", s.console, s.id, rom.image_hash);
                if let Ok(bytes) = crate::utils::s3::download_object(&other_new_path).await {
                    source_bytes = Some(bytes);

                    // Also try to recover small and icon from the same source ROM if needed
                    if crate::utils::s3::download_object(&old_small).await.is_err() {
                        let other_new_small = format!(
                            "covers_small/{}/{}/{}.webp",
                            s.console, s.id, rom.image_hash
                        );
                        if let Ok(s_bytes) =
                            crate::utils::s3::download_object(&other_new_small).await
                        {
                            let _ = crate::utils::s3::upload_object(&new_small, &s_bytes).await;
                        }
                    }
                    if crate::utils::s3::download_object(&old_icon).await.is_err() {
                        let other_new_icon =
                            format!("icons/{}/{}/{}.webp", s.console, s.id, rom.image_hash);
                        if let Ok(i_bytes) =
                            crate::utils::s3::download_object(&other_new_icon).await
                        {
                            let _ = crate::utils::s3::upload_object(&new_icon, &i_bytes).await;
                        }
                    }
                    break;
                }
            }
        }

        if let Some(bytes) = source_bytes {
            if crate::utils::s3::upload_object(&new_cover, &bytes)
                .await
                .is_ok()
            {
                moved_any = true;
            }

            if let Ok(s_bytes) = crate::utils::s3::download_object(&old_small).await {
                let _ = crate::utils::s3::upload_object(&new_small, &s_bytes).await;
            }
            if let Ok(i_bytes) = crate::utils::s3::download_object(&old_icon).await {
                let _ = crate::utils::s3::upload_object(&new_icon, &i_bytes).await;
            }
        }

        if moved_any {
            println!("Migrated covers for ROM {}", rom.id);
            migrated_count += 1;
        }
    }

    if migrated_count > 0 {
        println!("Successfully migrated covers for {} ROMs.", migrated_count);
    } else {
        println!("No covers needed migration.");
    }

    println!("Cleaning up unused covers in root directories...");
    for prefix in ["covers/", "covers_small/", "icons/"] {
        if let Ok(results) = crate::utils::s3::list_objects_shallow(prefix).await {
            for key in results {
                if key.ends_with(".webp") {
                    println!("Deleting unused file: {}", key);
                    let _ = crate::utils::s3::delete_object(&key).await;
                }
            }
        }
    }
}
