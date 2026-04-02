use rocket::{get, http::Status};
use rocket_dyn_templates::{Template, context};

use crate::routes::api::v1::guards::{AuthenticatedUser, UserRole};

#[get("/reports")]
pub async fn manage_reports(user: AuthenticatedUser) -> Result<Template, Status> {
    if user.role != UserRole::Admin && user.role != UserRole::Moderator {
        return Err(Status::Unauthorized);
    }

    Ok(Template::render("reports_manage", context! { role: user.role, user_id: user.id.0 }))
}
