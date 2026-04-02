use rocket::{get, http::Status};
use rocket_dyn_templates::{Template, context};

use crate::routes::api::v1::guards::AuthenticatedUser;

#[get("/saves")]
pub async fn saves_manage(_user: AuthenticatedUser) -> Result<Template, Status> {
    Ok(Template::render("saves_manage", context! {}))
}
