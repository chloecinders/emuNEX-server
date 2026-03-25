use crate::SQL;
use crate::utils::s3::{compute_md5, download_object, upload_object};
use image::ImageFormat;
use std::io::Cursor;

pub async fn migrate_covers() {
    println!("Checking if any covers need migration...");

    let mut root_covers = std::collections::HashMap::new();
    let mut root_smalls = std::collections::HashMap::new();
    let mut root_icons = std::collections::HashMap::new();

    let bucket = &*crate::S3;

    for base in ["covers", "covers_small", "icons"] {
        for prefix in [format!("{}/", base), format!("/{}/", base)] {
            if let Ok(results) = bucket.list(prefix.clone(), Some("/".to_string())).await {
                let mut count = 0;

                for res in results {
                    for obj in res.contents {
                        if obj.key.ends_with('/') {
                            continue;
                        }

                        let path = std::path::Path::new(&obj.key);
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let stem = stem.to_string();
                            if base == "covers" {
                                root_covers.insert(stem, obj.key);
                            } else if base == "covers_small" {
                                root_smalls.insert(stem, obj.key);
                            } else if base == "icons" {
                                root_icons.insert(stem, obj.key);
                            }
                            count += 1;
                        }
                    }
                }
            }
        }
    }

    println!(
        "Total images sitting in root folders: {}",
        root_covers.len() + root_smalls.len() + root_icons.len()
    );

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
        let new_small = format!("covers_small/{}/{}/{}.webp", rom.console, rom.id, hash);
        let new_icon = format!("icons/{}/{}/{}.webp", rom.console, rom.id, hash);

        let old_cover_key = root_covers.get(hash);
        if old_cover_key.is_none() {
            if crate::utils::s3::download_object(&new_cover).await.is_ok() {
                continue;
            }
        }

        let mut moved_any = false;
        let mut source_bytes = None;

        if let Some(key) = old_cover_key {
            if let Ok(bytes) = crate::utils::s3::download_object(key).await {
                source_bytes = Some(bytes);
            }
        }

        if source_bytes.is_none() {
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

                    if crate::utils::s3::download_object(&new_small).await.is_err() {
                        let other_new_small = format!(
                            "covers_small/{}/{}/{}.webp",
                            s.console, s.id, rom.image_hash
                        );
                        if let Some(key) = root_smalls.get(hash) {
                            if let Ok(s_bytes) = crate::utils::s3::download_object(key).await {
                                let _ = crate::utils::s3::upload_object(&new_small, &s_bytes).await;
                            }
                        }
                    }
                    if crate::utils::s3::download_object(&new_icon).await.is_err() {
                        let other_new_icon =
                            format!("icons/{}/{}/{}.webp", s.console, s.id, rom.image_hash);
                        if let Some(key) = root_icons.get(hash) {
                            if let Ok(i_bytes) = crate::utils::s3::download_object(key).await {
                                let _ = crate::utils::s3::upload_object(&new_icon, &i_bytes).await;
                            }
                        }
                    }
                    break;
                }
            }
        }

        if let Some(mut bytes) = source_bytes {
            if let Ok(img) = image::load_from_memory(&bytes) {
                let mut buf = std::io::Cursor::new(Vec::new());
                if img.write_to(&mut buf, image::ImageFormat::WebP).is_ok() {
                    bytes = buf.into_inner();
                }
            }

            if crate::utils::s3::upload_object(&new_cover, &bytes)
                .await
                .is_ok()
            {
                moved_any = true;
            }

            if let Some(key) = root_smalls.get(hash) {
                if let Ok(mut s_bytes) = crate::utils::s3::download_object(key).await {
                    if let Ok(img) = image::load_from_memory(&s_bytes) {
                        let mut buf = std::io::Cursor::new(Vec::new());
                        if img.write_to(&mut buf, image::ImageFormat::WebP).is_ok() {
                            s_bytes = buf.into_inner();
                        }
                    }
                    let _ = crate::utils::s3::upload_object(&new_small, &s_bytes).await;
                }
            }
            if let Some(key) = root_icons.get(hash) {
                if let Ok(mut i_bytes) = crate::utils::s3::download_object(key).await {
                    if let Ok(img) = image::load_from_memory(&i_bytes) {
                        let mut buf = std::io::Cursor::new(Vec::new());
                        if img.write_to(&mut buf, image::ImageFormat::WebP).is_ok() {
                            i_bytes = buf.into_inner();
                        }
                    }
                    let _ = crate::utils::s3::upload_object(&new_icon, &i_bytes).await;
                }
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

    for base in ["covers", "covers_small", "icons"] {
        for prefix in [format!("{}/", base), format!("/{}/", base)] {
            let bucket = &*crate::S3;

            if let Ok(results) = bucket.list(prefix, Some("/".to_string())).await {
                for res in results {
                    for obj in res.contents {
                        if !obj.key.ends_with('/') {
                            println!("Deleting unused file: {}", obj.key);
                            let _ = bucket.delete_object(&obj.key).await;
                        }
                    }
                }
            }
        }
    }
}
