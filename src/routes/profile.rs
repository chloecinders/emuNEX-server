use rocket::{get, http::Status};
use rocket_dyn_templates::{Template, context};
use crate::routes::api::v1::guards::AuthenticatedUser;

#[get("/profile")]
pub async fn profile_page(user: AuthenticatedUser) -> Result<Template, Status> {
    Ok(Template::render(
        "profile",
        context! {
            current_user: user,
            domain: crate::CONFIG.server_domain.clone()
        },
    ))
}

#[get("/settings")]
pub async fn settings_page(user: AuthenticatedUser) -> Result<Template, Status> {
    Ok(Template::render(
        "settings",
        context! {
            current_user: user,
            domain: crate::CONFIG.server_domain.clone()
        },
    ))
}
