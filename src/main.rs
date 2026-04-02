use rocket::{fs::FileServer, launch, routes};
use rocket_dyn_templates::Template;
use s3::Bucket;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::{fs::File, io::AsyncReadExt};

use crate::utils::{AutoOnceLock, Config, rate_limit::RateLimitFairing};

mod routes;
mod utils;

pub static CONFIG: AutoOnceLock<Config> = AutoOnceLock::new();
pub static SQL: AutoOnceLock<PgPool> = AutoOnceLock::new();
pub static S3: AutoOnceLock<Box<Bucket>> = AutoOnceLock::new();

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

    S3.set(utils::s3::create_bucket()).unwrap();

    if let Err(err) = sqlx::migrate!().run(&*SQL).await {
        panic!("Could not run database migrations; err = {err}")
    }

    rocket::build()
        .attach(Template::fairing())
        .attach(RateLimitFairing::new())
        .mount("/public", FileServer::from("./public"))
        .mount(
            "/",
            routes![
                routes::index::index,
                routes::dev::dev,
                routes::dev::update_server,
                routes::auth::auth_login,
                routes::auth::auth_register,
                routes::roms::rom_upload,
                routes::roms::rom_bulk_upload,
                routes::roms::rom_manage,
                routes::emulators::emulators_upload,
                routes::emulators::emulators_manage,
                routes::consoles::consoles_upload,
                routes::consoles::consoles_manage,
                routes::users::manage_users,
                routes::reports::manage_reports,
                routes::search_sections::search_sections_manage,
                routes::profile::profile_page,
                routes::profile::settings_page,
                routes::saves::saves_manage,
                routes::api::v1::auth::login,
                routes::api::v1::auth::register,
                routes::api::v1::auth::client_start,
                routes::api::v1::auth::me,
                routes::api::v1::auth::logout,
                routes::api::v1::auth::update_username,
                routes::api::v1::auth::update_password,
                routes::api::v1::auth::get_preferences,
                routes::api::v1::auth::update_preferences,
                routes::api::v1::auth::upload_avatar,
                routes::api::v1::auth::update_profile_color,
                routes::api::v1::roms::get_rom_list,
                routes::api::v1::roms::get_rom_single,
                routes::api::v1::roms::get_rom_versions,
                routes::api::v1::roms::search_roms,
                routes::api::v1::roms::upload_rom,
                routes::api::v1::roms::bulk_upload_roms,
                routes::api::v1::roms::update_rom,
                routes::api::v1::roms::update_rom_file,
                routes::api::v1::roms::update_rom_image,
                routes::api::v1::roms::delete_rom,
                routes::api::v1::roms::get_categories,
                routes::api::v1::roms::start_game,
                routes::api::v1::roms::get_user_library,
                routes::api::v1::roms::get_search_overview,
                routes::api::v1::library::get_shelves,
                routes::api::v1::library::create_shelf,
                routes::api::v1::library::update_shelf,
                routes::api::v1::library::delete_shelf,
                routes::api::v1::library::add_rom_to_shelf,
                routes::api::v1::library::remove_rom_from_shelf,
                routes::api::v1::library::update_rom_order,
                routes::api::v1::search_sections::get_search_sections,
                routes::api::v1::search_sections::create_search_section,
                routes::api::v1::search_sections::update_search_section,
                routes::api::v1::search_sections::update_search_sections_order,
                routes::api::v1::search_sections::delete_search_section,
                routes::api::v1::consoles::get_consoles,
                routes::api::v1::consoles::upload_console,
                routes::api::v1::consoles::update_console_metadata,
                routes::api::v1::consoles::delete_console,
                routes::api::v1::emulators::get_emulators_for_platform,
                routes::api::v1::emulators::emulator_upload,
                routes::api::v1::emulators::get_all_emulators,
                routes::api::v1::emulators::update_emulator,
                routes::api::v1::emulators::update_emulator_binary,
                routes::api::v1::emulators::delete_emulator,
                routes::api::v1::saves::upload_save,
                routes::api::v1::saves::get_all_saves,
                routes::api::v1::saves::get_latest_save,
                routes::api::v1::saves::download_save_file,
                routes::api::v1::users::update_user,
                routes::api::v1::users::get_users,
                routes::api::v1::users::get_invites,
                routes::api::v1::users::create_invite,
                routes::api::v1::users::delete_invite,
                routes::api::v1::nointro::import_nointro,
                routes::api::v1::nointro::get_nointro_dats,
                routes::api::v1::nointro::update_dat_notes,
                routes::api::v1::nointro::search_nointro,
                routes::api::v1::reports::create_report,
                routes::api::v1::reports::get_reports,
                routes::api::v1::reports::resolve_report,
                routes::api::v1::reports::claim_report,
                routes::api::v1::reports::unclaim_report,
                routes::api::v1::reports::delete_report,
                routes::proxy::storage,
                routes::proxy::storage_options,
            ],
        )
}
