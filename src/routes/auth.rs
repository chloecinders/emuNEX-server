use rocket::get;
use rocket_dyn_templates::{Template, context};

use crate::CONFIG;

#[get("/auth/login")]
pub fn auth_login() -> Template {
    Template::render(
        "login",
        context! { auth_type: "login", domain: CONFIG.server_domain.clone() },
    )
}

#[get("/auth/register")]
pub fn auth_register() -> Template {
    Template::render(
        "login",
        context! { auth_type: "register", domain: CONFIG.server_domain.clone() },
    )
}
