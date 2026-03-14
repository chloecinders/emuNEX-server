use rocket::{get, response::Redirect};

#[get("/")]
pub fn index() -> Redirect {
    Redirect::permanent("/auth/login")
}
