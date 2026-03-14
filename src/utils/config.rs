use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub database_url: String,
    pub seaweedfs_url: String,
    pub repository: Option<String>,
    pub github_token: Option<String>,
}
