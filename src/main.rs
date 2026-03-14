use rocket::{launch, routes};
use rocket_dyn_templates::Template;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::{fs::File, io::AsyncReadExt};

use crate::utils::{AutoOnceLock, Config};

mod routes;
mod utils;

pub static CONFIG: AutoOnceLock<Config> = AutoOnceLock::new();
pub static SQL: AutoOnceLock<PgPool> = AutoOnceLock::new();

#[launch]
async fn rocket() -> _ {
    let mut config_file = File::open("./Config.toml")
        .await
        .expect("Could not find Config.toml in cwd");
    let mut config_string = String::new();

    config_file
        .read_to_string(&mut config_string)
        .await
        .expect("Could not read Config.toml");

    let config: Config = toml::from_str(&config_string).expect("Could not parse Config.toml");
    CONFIG.set(config).unwrap();

    SQL.set({
        async {
            PgPoolOptions::new()
                .max_connections(5)
                .connect(&*CONFIG.database_url)
                .await
                .expect("Failed to create database pool, make sure the database url in the config is valid.")
        }
        .await
    })
    .unwrap();

    if let Err(err) = sqlx::migrate!().run(&*SQL).await {
        panic!("Could not run database migrations; err = {err}")
    }

    rocket::build().attach(Template::fairing()).mount(
        "/",
        routes![
            routes::index::index,
            routes::dev::dev,
            routes::dev::update_server,
            routes::auth::auth_login,
            routes::auth::auth_register,
            routes::roms::rom_upload,
            routes::emulators::emulators_upload,
            routes::consoles::consoles_upload,
            routes::api::v1::auth::login,
            routes::api::v1::auth::register,
            routes::api::v1::auth::client_start,
            routes::api::v1::auth::me,
            routes::api::v1::roms::get_rom_list,
            routes::api::v1::roms::get_rom_single,
            routes::api::v1::roms::search_roms,
            routes::api::v1::roms::upload_rom,
            routes::api::v1::roms::get_categories,
            routes::api::v1::roms::start_game,
            routes::api::v1::roms::get_user_library,
            routes::api::v1::consoles::get_consoles,
            routes::api::v1::consoles::upload_console,
            routes::api::v1::emulators::get_emulators_for_platform,
            routes::api::v1::emulators::emulator_upload,
            routes::proxy::storage,
            routes::proxy::storage_options,
        ],
    )
}
