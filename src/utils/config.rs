use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct S3Config {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub server_domain: String,
    pub database_url: String,
    pub s3: S3Config,
    pub repository: Option<String>,
    pub github_token: Option<String>,
}
