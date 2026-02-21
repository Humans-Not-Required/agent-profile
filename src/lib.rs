pub mod db;
pub mod models;
pub mod routes;

use rusqlite::Connection;
use std::sync::Mutex;

use routes::profiles::{
    health, create_profile, list_profiles, get_profile, update_profile, delete_profile,
    add_address, delete_address, add_link, delete_link, add_skill, delete_skill,
};
use routes::html::profile_page;

pub fn create_rocket(db_path: &str) -> rocket::Rocket<rocket::Build> {
    let conn = Connection::open(db_path).expect("Failed to open database");
    db::init_db(&conn).expect("Failed to initialize database");
    let db_state = Mutex::new(conn);

    rocket::build()
        .manage(db_state)
        .mount("/api/v1", rocket::routes![
            health,
            create_profile,
            list_profiles,
            get_profile,
            update_profile,
            delete_profile,
            add_address,
            delete_address,
            add_link,
            delete_link,
            add_skill,
            delete_skill,
        ])
        .mount("/", rocket::routes![profile_page])
}
