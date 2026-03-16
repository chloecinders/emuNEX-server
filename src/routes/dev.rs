use rocket::{get, http::Status, post};
use rocket_dyn_templates::{Template, context};
use serde::Deserialize;

use crate::{
    CONFIG,
    routes::api::v1::guards::{AuthenticatedUser, UserRole},
};

#[get("/dev")]
pub fn dev(user: AuthenticatedUser) -> Result<Template, Status> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(Status::Unauthorized);
    }

    Ok(Template::render("dev", context! { role: user.role }))
}

use std::{fs, io::Cursor, process, time::Duration};
#[cfg(not(target_os = "windows"))]
use std::{fs::Permissions, os::unix::fs::PermissionsExt};

#[post("/admin/update")]
pub async fn update_server(user: AuthenticatedUser) -> Result<Status, String> {
    if user.role != UserRole::Admin {
        return Err("Not Authorized".into());
    }

    let repo = CONFIG.repository.clone().unwrap_or_default();
    let client = reqwest::Client::new();

    let mut req_builder = client
        .get(format!(
            "https://api.github.com/repos/{repo}/actions/runs?per_page=1"
        ))
        .header("User-Agent", "Rocket-Server-Updater");

    if let Some(token) = &CONFIG.github_token {
        req_builder = req_builder.header("Authorization", format!("Bearer {token}"));
    }

    let res = req_builder.send().await.map_err(|e| e.to_string())?;
    let json: WorkflowRunsResponse = res.json().await.map_err(|e| e.to_string())?;

    if let Some(run) = json.workflow_runs.first() {
        if run.status != "completed" || run.conclusion.as_deref() != Some("success") {
            return Err("Latest CI run failed".into());
        }

        let mut art_req = client
            .get(&run.artifacts_url)
            .header("User-Agent", "Rocket-Server-Updater");

        if let Some(token) = &CONFIG.github_token {
            art_req = art_req.header("Authorization", format!("Bearer {token}"));
        }

        let art_res = art_req.send().await.map_err(|e| e.to_string())?;
        let art_json: ArtifactsResponse = art_res.json().await.map_err(|e| e.to_string())?;

        let artifact = art_json
            .artifacts
            .iter()
            .find(|a| !a.name.ends_with(".exe"))
            .ok_or("No valid artifact found")?;

        let mut dl_req = client
            .get(&artifact.archive_download_url)
            .header("User-Agent", "Rocket-Server-Updater");

        if let Some(token) = &CONFIG.github_token {
            dl_req = dl_req.header("Authorization", format!("Bearer {token}"));
        }

        let bytes = dl_req
            .send()
            .await
            .map_err(|e| e.to_string())?
            .bytes()
            .await
            .map_err(|e| e.to_string())?;

        tokio::task::spawn_blocking(move || {
            let reader = Cursor::new(bytes);
            let mut zip = zip::ZipArchive::new(reader).map_err(|e| e.to_string())?;
            let mut binary_found = false;

            for i in 0..zip.len() {
                let mut file = zip.by_index(i).map_err(|e| e.to_string())?;
                let name = file.name().to_string();

                if name == "target/release/emunex-server" {
                    let mut outfile =
                        fs::File::create("./emunex-server.new").map_err(|e| e.to_string())?;
                    std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
                    binary_found = true;
                } else if name.starts_with("templates") {
                    let rel_path = &name["templates".len()..];
                    if rel_path.is_empty() {
                        continue;
                    }

                    let target_path = std::path::Path::new("./templates").join(rel_path);

                    if name.ends_with('/') {
                        fs::create_dir_all(&target_path).map_err(|e| e.to_string())?;
                    } else {
                        if let Some(p) = target_path.parent() {
                            if !p.exists() {
                                fs::create_dir_all(p).map_err(|e| e.to_string())?;
                            }
                        }

                        let mut outfile =
                            fs::File::create(&target_path).map_err(|e| e.to_string())?;

                        std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
                    }
                } else if name.starts_with("public") {
                    let rel_path = &name["public".len()..];
                    if rel_path.is_empty() {
                        continue;
                    }

                    let target_path = std::path::Path::new("./public").join(rel_path);

                    if name.ends_with('/') {
                        fs::create_dir_all(&target_path).map_err(|e| e.to_string())?;
                    } else {
                        if let Some(p) = target_path.parent() {
                            if !p.exists() {
                                fs::create_dir_all(p).map_err(|e| e.to_string())?;
                            }
                        }

                        let mut outfile =
                            fs::File::create(&target_path).map_err(|e| e.to_string())?;

                        std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
                    }
                }
            }

            if !binary_found {
                return Err("Binary not found in artifact".into());
            }

            Ok::<(), String>(())
        })
        .await
        .map_err(|e| e.to_string())??;

        let target = "./emunex-server";
        let temp_bin = "./emunex-server.new";

        fs::remove_file(target).ok();
        fs::copy(temp_bin, target).map_err(|e| e.to_string())?;
        #[cfg(not(target_os = "windows"))]
        fs::set_permissions(target, Permissions::from_mode(0o755)).map_err(|e| e.to_string())?;
        fs::remove_file(temp_bin).ok();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(250)).await;
            process::exit(0);
        });
    }

    Ok(Status::Ok)
}

#[derive(Debug, Deserialize)]
pub struct WorkflowRunsResponse {
    pub workflow_runs: Vec<WorkflowRun>,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowRun {
    pub status: String,
    pub conclusion: Option<String>,
    pub artifacts_url: String,
}

#[derive(Debug, Deserialize)]
pub struct ArtifactsResponse {
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub archive_download_url: String,
}
