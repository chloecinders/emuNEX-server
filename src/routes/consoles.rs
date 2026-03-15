use rocket::{get, http::Status};
use rocket_dyn_templates::{Template, context};

use crate::routes::api::v1::guards::{AuthenticatedUser, UserRole};

#[get("/consoles")]
pub fn consoles_manage(user: AuthenticatedUser) -> Result<Template, Status> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(Status::Unauthorized);
    }

    Ok(Template::render("consoles_manage", context! {}))
}

#[get("/consoles/upload")]
pub fn consoles_upload(user: AuthenticatedUser) -> Result<Template, Status> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(Status::Unauthorized);
    }

    Ok(Template::render("consoles", context! {}))
}
