use rocket::{get, http::Status};
use rocket_dyn_templates::{Template, context};

use crate::routes::api::v1::guards::{AuthenticatedUser, UserRole};

#[get("/emulators/upload")]
pub fn emulators_upload(user: AuthenticatedUser) -> Result<Template, Status> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(Status::Unauthorized);
    }

    Ok(Template::render("emulator", context! {}))
}
