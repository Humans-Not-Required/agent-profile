pub mod assets;
pub mod cors;
pub mod db;
pub mod models;
pub mod ratelimit;
pub mod routes;

use rusqlite::Connection;
use std::sync::Mutex;

use cors::Cors;
use ratelimit::RateLimiter;
use rocket::http::Status;
use rocket::serde::json::Json;
use routes::profiles::{
    health, register, reissue_key,
    list_profiles, list_skills, get_stats,
    get_profile, update_profile, delete_profile, get_score, badge_svg,
    add_address, delete_address,
    add_link, delete_link,
    add_section, update_section, delete_section,
    add_skill, delete_skill,
    upload_avatar, serve_avatar,
    get_challenge, verify_signature,
    add_endorsement, get_endorsements, delete_endorsement,
    llms_txt, openapi_json, skills_index, robots_txt, sitemap_xml, webfinger,
};
use routes::html::{landing_page, profile_page};

/// 429 Too Many Requests catcher — returns JSON error body.
#[rocket::catch(429)]
fn rate_limit_catcher() -> (Status, Json<serde_json::Value>) {
    (Status::TooManyRequests, Json(serde_json::json!({
        "error": "Too many requests. Please slow down and try again later.",
        "retry_after_seconds": 60,
    })))
}

/// 404 Not Found catcher — returns JSON error body.
#[rocket::catch(404)]
fn not_found_catcher() -> (Status, Json<serde_json::Value>) {
    (Status::NotFound, Json(serde_json::json!({
        "error": "Not found.",
    })))
}

pub fn create_rocket(db_path: &str) -> rocket::Rocket<rocket::Build> {
    let conn = Connection::open(db_path).expect("Failed to open database");
    db::init_db(&conn).expect("Failed to initialize database");
    let db_state = Mutex::new(conn);

    rocket::build()
        .manage(db_state)
        .manage(RateLimiter::new())
        .attach(Cors)
        .register("/", rocket::catchers![rate_limit_catcher, not_found_catcher])
        .mount("/api/v1", rocket::routes![
            health,
            register,
            reissue_key,
            list_profiles,
            list_skills,
            get_stats,
            get_profile,
            update_profile,
            delete_profile,
            get_score,
            badge_svg,
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
            add_endorsement,
            get_endorsements,
            delete_endorsement,
        ])
        .mount("/", rocket::routes![
            landing_page,
            profile_page,
            serve_avatar,
            llms_txt,
            openapi_json,
            skills_index,
            robots_txt,
            sitemap_xml,
            webfinger,
            assets::serve_asset,
        ])
}
