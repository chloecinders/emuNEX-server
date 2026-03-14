use rocket::get;
use rocket_dyn_templates::{Template, context};

#[get("/auth/login")]
pub fn auth_login() -> Template {
    Template::render("login", context! { auth_type: "login" })
}

#[get("/auth/register")]
pub fn auth_register() -> Template {
    Template::render("login", context! { auth_type: "register" })
}
