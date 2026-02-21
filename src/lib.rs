pub mod cors;
pub mod db;
pub mod models;
pub mod routes;

use rusqlite::Connection;
use std::sync::Mutex;

use cors::Cors;
use routes::profiles::{
    health, register, reissue_key,
    list_profiles, get_profile, update_profile, delete_profile, get_score,
    add_address, delete_address,
    add_link, delete_link,
    add_section, update_section, delete_section,
    add_skill, delete_skill,
    upload_avatar, serve_avatar,
    get_challenge, verify_signature,
    llms_txt, openapi_json, skills_index,
};
use routes::html::profile_page;

pub fn create_rocket(db_path: &str) -> rocket::Rocket<rocket::Build> {
    let conn = Connection::open(db_path).expect("Failed to open database");
    db::init_db(&conn).expect("Failed to initialize database");
    let db_state = Mutex::new(conn);

    rocket::build()
        .manage(db_state)
        .attach(Cors)
        .mount("/api/v1", rocket::routes![
            health,
            register,
            reissue_key,
            list_profiles,
            get_profile,
            update_profile,
            delete_profile,
            get_score,
            add_address,
            delete_address,
            add_link,
            delete_link,
            add_section,
            update_section,
            delete_section,
            add_skill,
            delete_skill,
            upload_avatar,
            get_challenge,
            verify_signature,
        ])
        .mount("/", rocket::routes![
            profile_page,
            serve_avatar,
            llms_txt,
            openapi_json,
            skills_index,
        ])
}
