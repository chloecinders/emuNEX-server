use rocket::{get, http::Status};
use rocket_dyn_templates::{Template, context};

use crate::routes::api::v1::guards::{AuthenticatedUser, UserRole};

#[get("/roms")]
pub fn rom_manage(user: AuthenticatedUser) -> Result<Template, Status> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(Status::Unauthorized);
    }

    Ok(Template::render("roms_manage", context! {}))
}

#[get("/roms/upload")]
pub fn rom_upload(user: AuthenticatedUser) -> Result<Template, Status> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(Status::Unauthorized);
    }

    Ok(Template::render("rom", context! {}))
}

#[get("/roms/bulk_upload")]
pub fn rom_bulk_upload(user: AuthenticatedUser) -> Result<Template, Status> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(Status::Unauthorized);
    }

    Ok(Template::render("bulk_import", context! {}))
}
