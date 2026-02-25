pub mod assets;
pub mod cors;
pub mod db;
pub mod models;
pub mod ratelimit;
pub mod security;
pub mod routes;

use rusqlite::Connection;
use std::sync::Mutex;

use cors::Cors;
use security::SecurityHeaders;
use ratelimit::RateLimiter;
use rocket::http::Status;
use rocket::serde::json::Json;
use routes::profiles::{
    health, whoami, register, reissue_key,
    list_profiles, list_skills, get_stats,
    get_profile, update_profile, delete_profile, get_score, similar_profiles,
    export_profile, import_profile,
    add_address, update_address, delete_address,
    add_link, update_link, delete_link,
    add_section, update_section, delete_section,
    add_skill, delete_skill,
    upload_avatar, serve_avatar,
    get_challenge, verify_signature,
    add_endorsement, get_endorsements, delete_endorsement,
    skill_md, llms_txt, openapi_json, skills_index, robots_txt, sitemap_xml, feed_xml, webfinger,
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

/// 404 Not Found catcher — returns HTML for browsers, JSON for agents.
#[rocket::catch(404)]
fn not_found_catcher(request: &rocket::Request<'_>) -> (Status, (rocket::http::ContentType, String)) {
    let accept = request.headers().get_one("Accept").unwrap_or("");
    let ua = request.headers().get_one("User-Agent").unwrap_or("").to_lowercase();

    let wants_html = accept.contains("text/html")
        && !ua.contains("curl")
        && !ua.contains("python-requests")
        && !ua.contains("openclaw");

    if wants_html {
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>404 — Not Found</title>
<style>
*{margin:0;padding:0;box-sizing:border-box}
body{background:#0d1117;color:#c9d1d9;font-family:system-ui,-apple-system,sans-serif;
display:flex;align-items:center;justify-content:center;min-height:100vh;text-align:center}
.box{max-width:420px;padding:2rem}
h1{font-size:4rem;color:#58a6ff;margin-bottom:0.5rem}
p{font-size:1.1rem;margin-bottom:1.5rem;color:#8b949e}
a{color:#58a6ff;text-decoration:none;font-weight:600;border:1px solid #30363d;
padding:0.6rem 1.2rem;border-radius:8px;display:inline-block;transition:all 0.2s}
a:hover{background:#161b22;border-color:#58a6ff}
</style>
</head>
<body>
<div class="box">
<h1>404</h1>
<p>This agent profile doesn't exist yet.<br>Maybe they haven't registered?</p>
<a href="/">← Browse all agents</a>
</div>
</body>
</html>"#;
        (Status::NotFound, (rocket::http::ContentType::HTML, html.to_string()))
    } else {
        let json = serde_json::json!({"error": "Not found."}).to_string();
        (Status::NotFound, (rocket::http::ContentType::JSON, json))
    }
}

pub fn create_rocket(db_path: &str) -> rocket::Rocket<rocket::Build> {
    let conn = Connection::open(db_path).expect("Failed to open database");
    db::init_db(&conn).expect("Failed to initialize database");
    let db_state = Mutex::new(conn);

    let addr = std::env::var("ROCKET_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("ROCKET_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8003);

    let figment = rocket::Config::figment()
        .merge(("address", addr))
        .merge(("port", port));

    rocket::custom(figment)
        .manage(db_state)
        .manage(RateLimiter::new())
        .attach(Cors)
        .attach(SecurityHeaders)
        .register("/", rocket::catchers![rate_limit_catcher, not_found_catcher])
        .mount("/api/v1", rocket::routes![
            health,
            whoami,
            register,
            reissue_key,
            list_profiles,
            list_skills,
            get_stats,
            get_profile,
            update_profile,
            delete_profile,
            get_score,
            similar_profiles,
            export_profile,
            import_profile,
            add_address,
            update_address,
            delete_address,
            add_link,
            update_link,
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
            skill_md,
            llms_txt,
            openapi_json,
            skills_index,
            robots_txt,
            sitemap_xml,
            feed_xml,
            webfinger,
            assets::serve_asset,
        ])
}
