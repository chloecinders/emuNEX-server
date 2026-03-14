use rocket::{get, http::Status, response::Redirect};
use rocket_dyn_templates::{Template, context};

use crate::routes::api::v1::guards::{AuthenticatedUser, UserRole};

#[get("/")]
pub fn index() -> Redirect {
    Redirect::permanent("/auth/login")
}

#[get("/dev")]
pub fn dev(user: AuthenticatedUser) -> Result<Template, Status> {
    if user.role != UserRole::Admin {
        return Err(Status::Unauthorized);
    }

    Ok(Template::render("dev", context! {}))
}
