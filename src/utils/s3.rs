use s3::Bucket;
use s3::Region;
use s3::creds::Credentials;
use tracing::error;

use crate::CONFIG;
use crate::S3;

pub fn compute_md5(bytes: &[u8]) -> String {
    format!("{:x}", md5::compute(bytes))
}

pub fn create_bucket() -> Box<Bucket> {
    let region = Region::Custom {
        region: CONFIG.s3.region.clone().into(),
        endpoint: CONFIG.s3.endpoint.clone().into(),
    };

    let credentials = Credentials::new(
        Some(&CONFIG.s3.access_key),
        Some(&CONFIG.s3.secret_key),
        None,
        None,
        None,
    )
    .expect("Failed to create S3 credentials");

    Bucket::new(&CONFIG.s3.bucket, region, credentials)
        .expect("Failed to create S3 bucket handle")
        .with_path_style()
}

pub async fn upload_object(key: &str, body: &[u8]) -> Result<(), String> {
    let bucket = &*S3;
    bucket.put_object(key, body).await.map_err(|e| {
        error!("S3 upload error for key {}: {:?}", key, e);
        format!("S3 upload error: {e}")
    })?;
    Ok(())
}

pub async fn download_object(key: &str) -> Result<Vec<u8>, String> {
    let bucket = &*S3;
    let response = bucket.get_object(key).await.map_err(|e| {
        error!("S3 download error for key {}: {:?}", key, e);
        format!("S3 download error: {e}")
    })?;
    Ok(response.to_vec())
}

pub async fn delete_object(key: &str) -> Result<(), String> {
    let bucket = &*S3;
    bucket.delete_object(key).await.map_err(|e| {
        error!("S3 delete error for key {}: {:?}", key, e);
        format!("S3 delete error: {e}")
    })?;
    Ok(())
}
