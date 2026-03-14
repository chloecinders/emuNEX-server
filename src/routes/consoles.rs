use rocket::{get, http::Status};
use rocket_dyn_templates::{Template, context};

use crate::routes::api::v1::guards::{AuthenticatedUser, UserRole};

#[get("/consoles/upload")]
pub fn consoles_upload(user: AuthenticatedUser) -> Result<Template, Status> {
    if user.role != UserRole::Admin {
        return Err(Status::Unauthorized);
    }

    Ok(Template::render("consoles", context! {}))
}
