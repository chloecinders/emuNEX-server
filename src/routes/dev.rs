use rocket::{get, http::Status, post};
use rocket_dyn_templates::{Template, context};

use crate::routes::api::v1::guards::{AuthenticatedUser, UserRole};

#[get("/dev")]
pub fn dev(user: AuthenticatedUser) -> Result<Template, Status> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(Status::Unauthorized);
    }

    Ok(Template::render("dev", context! { role: user.role }))
}

use std::{process, time::Duration};

const LAUNCHER_UPDATE_SIGNAL: i32 = 2;

#[post("/admin/update")]
pub async fn update_server(user: AuthenticatedUser) -> Result<Status, String> {
    if user.role != UserRole::Admin {
        return Err("Not Authorized".into());
    }

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(250)).await;
        process::exit(LAUNCHER_UPDATE_SIGNAL);
    });

    Ok(Status::Ok)
}
