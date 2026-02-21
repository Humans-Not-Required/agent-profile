use rocket::{get, post, patch, delete, State, http::{Status, ContentType}, serde::json::Json};
use rocket::data::{Data, ToByteUnit};
use rusqlite::{Connection, params};
use serde_json::json;
use sha2::{Sha256, Digest};
use uuid::Uuid;
use chrono::Utc;
use std::sync::Mutex;

use crate::models::*;

pub type DbConn = Mutex<Connection>;

fn hash_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}

fn gen_api_key() -> String {
    format!("ap_{}", Uuid::new_v4().to_string().replace('-', ""))
}

fn now() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

pub(crate) fn load_profile(conn: &Connection, username: &str) -> Option<Profile> {
    let result = conn.query_row(
        "SELECT id, username, display_name, tagline, bio, third_line, avatar_url, \
         theme, particle_effect, particle_enabled, particle_seasonal, pubkey, \
         profile_score, created_at, updated_at \
         FROM profiles WHERE username = ?1",
        params![username],
        |row| Ok((
            row.get::<_, String>(0)?,   // id
            row.get::<_, String>(1)?,   // username
            row.get::<_, String>(2)?,   // display_name
            row.get::<_, String>(3)?,   // tagline
            row.get::<_, String>(4)?,   // bio
            row.get::<_, String>(5)?,   // third_line
            row.get::<_, String>(6)?,   // avatar_url
            row.get::<_, String>(7)?,   // theme
            row.get::<_, String>(8)?,   // particle_effect
            row.get::<_, i32>(9)?,      // particle_enabled
            row.get::<_, i32>(10)?,     // particle_seasonal
            row.get::<_, String>(11)?,  // pubkey
            row.get::<_, i64>(12)?,     // profile_score
            row.get::<_, String>(13)?,  // created_at
            row.get::<_, String>(14)?,  // updated_at
        )),
    ).ok()?;

    let (id, username, display_name, tagline, bio, third_line, avatar_url,
         theme, particle_effect, particle_enabled, particle_seasonal, pubkey,
         profile_score, created_at, updated_at) = result;

    let crypto_addresses = load_addresses(conn, &id);
    let links = load_links(conn, &id);
    let sections = load_sections(conn, &id);
    let skills = load_skills(conn, &id);

    Some(Profile {
        id, username, display_name, tagline, bio, third_line, avatar_url,
        theme, particle_effect,
        particle_enabled: particle_enabled != 0,
        particle_seasonal: particle_seasonal != 0,
        pubkey, profile_score, created_at, updated_at,
        crypto_addresses, links, sections, skills,
    })
}

fn load_addresses(conn: &Connection, profile_id: &str) -> Vec<CryptoAddress> {
    let mut stmt = conn.prepare(
        "SELECT id, profile_id, network, address, label, created_at \
         FROM crypto_addresses WHERE profile_id = ?1 ORDER BY created_at"
    ).unwrap();
    stmt.query_map(params![profile_id], |row| {
        Ok(CryptoAddress {
            id: row.get(0)?, profile_id: row.get(1)?,
            network: row.get(2)?, address: row.get(3)?,
            label: row.get(4)?, created_at: row.get(5)?,
        })
    }).unwrap().flatten().collect()
}

fn load_links(conn: &Connection, profile_id: &str) -> Vec<ProfileLink> {
    let mut stmt = conn.prepare(
        "SELECT id, profile_id, url, label, platform, display_order, created_at \
         FROM profile_links WHERE profile_id = ?1 ORDER BY display_order, created_at"
    ).unwrap();
    stmt.query_map(params![profile_id], |row| {
        Ok(ProfileLink {
            id: row.get(0)?, profile_id: row.get(1)?,
            url: row.get(2)?, label: row.get(3)?,
            platform: row.get(4)?, display_order: row.get(5)?,
            created_at: row.get(6)?,
        })
    }).unwrap().flatten().collect()
}

fn load_sections(conn: &Connection, profile_id: &str) -> Vec<ProfileSection> {
    let mut stmt = conn.prepare(
        "SELECT id, profile_id, section_type, title, content, display_order, created_at \
         FROM profile_sections WHERE profile_id = ?1 ORDER BY display_order, created_at"
    ).unwrap();
    stmt.query_map(params![profile_id], |row| {
        Ok(ProfileSection {
            id: row.get(0)?, profile_id: row.get(1)?,
            section_type: row.get(2)?, title: row.get(3)?,
            content: row.get(4)?, display_order: row.get(5)?,
            created_at: row.get(6)?,
        })
    }).unwrap().flatten().collect()
}

fn load_skills(conn: &Connection, profile_id: &str) -> Vec<ProfileSkill> {
    let mut stmt = conn.prepare(
        "SELECT id, profile_id, skill, created_at \
         FROM profile_skills WHERE profile_id = ?1 ORDER BY created_at"
    ).unwrap();
    stmt.query_map(params![profile_id], |row| {
        Ok(ProfileSkill {
            id: row.get(0)?, profile_id: row.get(1)?,
            skill: row.get(2)?, created_at: row.get(3)?,
        })
    }).unwrap().flatten().collect()
}

fn update_profile_score(conn: &Connection, profile_id: &str, username: &str) {
    if let Some(p) = load_profile(conn, username) {
        let has_addr = !p.crypto_addresses.is_empty();
        let has_link = !p.links.is_empty();
        let has_section = !p.sections.is_empty();
        let has_skill = !p.skills.is_empty();
        let score = compute_profile_score(
            &p.display_name, &p.tagline, &p.bio, &p.avatar_url,
            &p.pubkey, has_addr, has_link, has_section, has_skill,
        );
        let _ = conn.execute(
            "UPDATE profiles SET profile_score = ?1 WHERE id = ?2",
            params![score, profile_id],
        );
    }
}

fn verify_api_key(conn: &Connection, username: &str, key: &str) -> bool {
    let hashed = hash_key(key);
    conn.query_row(
        "SELECT COUNT(*) FROM profiles WHERE username = ?1 AND api_key_hash = ?2",
        params![username, hashed],
        |row| row.get::<_, i64>(0),
    ).unwrap_or(0) > 0
}

fn get_profile_id(conn: &Connection, username: &str) -> Option<String> {
    conn.query_row(
        "SELECT id FROM profiles WHERE username = ?1",
        params![username],
        |row| row.get(0),
    ).ok()
}

// --- Health ---

#[get("/health")]
pub fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        service: "agent-profile".to_string(),
    })
}

// --- Registration ---

#[post("/register", data = "<body>")]
pub fn register(
    db: &State<DbConn>,
    body: Json<RegisterRequest>,
) -> Result<(Status, Json<serde_json::Value>), (Status, Json<serde_json::Value>)> {
    let username = validate_username(&body.username).map_err(|e| {
        (Status::UnprocessableEntity, Json(json!({"error": e})))
    })?;

    // Validate pubkey if provided
    if let Some(ref pk) = body.pubkey {
        if !pk.is_empty() && !validate_pubkey(pk) {
            return Err((Status::UnprocessableEntity, Json(json!({
                "error": "Invalid public key. Provide a secp256k1 key as 66-char (compressed) or 130-char (uncompressed) hex."
            }))));
        }
    }

    let api_key = gen_api_key();
    let hashed = hash_key(&api_key);
    let id = Uuid::new_v4().to_string();
    let ts = now();
    let display_name = body.display_name.clone().unwrap_or_default();
    let pubkey = body.pubkey.clone().unwrap_or_default();

    // Compute initial score
    let score = compute_profile_score(
        &display_name, "", "", "", &pubkey,
        false, false, false, false,
    );

    let conn = db.lock().unwrap();
    match conn.execute(
        "INSERT INTO profiles (id, username, display_name, api_key_hash, pubkey, profile_score, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, username, display_name, hashed, pubkey, score, ts, ts],
    ) {
        Ok(_) => Ok((Status::Created, Json(json!({
            "username": username,
            "api_key": api_key,
            "profile_url": format!("/{}", username),
            "json_url": format!("/api/v1/profiles/{}", username),
        })))),
        Err(rusqlite::Error::SqliteFailure(e, _)) if e.extended_code == 2067 => {
            Err((Status::Conflict, Json(json!({"error": format!("Username '{}' already taken", username)}))))
        },
        Err(e) => Err((Status::InternalServerError, Json(json!({"error": e.to_string()})))),
    }
}

// --- Reissue API key ---

#[post("/profiles/<username>/reissue-key")]
pub fn reissue_key(
    db: &State<DbConn>,
    username: &str,
    api_key: ApiKey,
) -> Result<Json<serde_json::Value>, (Status, Json<serde_json::Value>)> {
    let username = username.to_lowercase();
    let conn = db.lock().unwrap();

    if !verify_api_key(&conn, &username, &api_key.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid API key"}))));
    }

    let new_key = gen_api_key();
    let new_hash = hash_key(&new_key);
    let ts = now();

    conn.execute(
        "UPDATE profiles SET api_key_hash = ?1, updated_at = ?2 WHERE username = ?3",
        params![new_hash, ts, username],
    ).map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(json!({
        "username": username,
        "api_key": new_key,
        "message": "Previous key is immediately invalidated.",
    })))
}

// --- Profile CRUD ---

#[get("/profiles?<q>&<theme>&<limit>&<offset>")]
pub fn list_profiles(
    db: &State<DbConn>,
    q: Option<&str>,
    theme: Option<&str>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Json<serde_json::Value> {
    let conn = db.lock().unwrap();
    let lim = limit.unwrap_or(20).min(100);
    let off = offset.unwrap_or(0);

    let mut query = "SELECT id, username, display_name, tagline, avatar_url, \
                     theme, profile_score, created_at FROM profiles".to_string();
    let mut conditions: Vec<String> = vec![];
    let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

    if let Some(q_val) = q {
        let search = format!("%{}%", q_val.to_lowercase());
        conditions.push(format!(
            "(LOWER(username) LIKE ?{0} OR LOWER(display_name) LIKE ?{0} OR LOWER(bio) LIKE ?{0})",
            values.len() + 1
        ));
        values.push(Box::new(search));
    }
    if let Some(t) = theme {
        conditions.push(format!("theme = ?{}", values.len() + 1));
        values.push(Box::new(t.to_string()));
    }

    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }
    query.push_str(&format!(" ORDER BY profile_score DESC, created_at DESC LIMIT {} OFFSET {}", lim, off));

    let refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|p| p.as_ref()).collect();
    let mut stmt = match conn.prepare(&query) {
        Ok(s) => s,
        Err(e) => return Json(json!({"error": e.to_string(), "profiles": []})),
    };

    let profiles: Vec<serde_json::Value> = match stmt.query_map(refs.as_slice(), |row| {
        Ok(json!({
            "id": row.get::<_, String>(0)?,
            "username": row.get::<_, String>(1)?,
            "display_name": row.get::<_, String>(2)?,
            "tagline": row.get::<_, String>(3)?,
            "avatar_url": row.get::<_, String>(4)?,
            "theme": row.get::<_, String>(5)?,
            "profile_score": row.get::<_, i64>(6)?,
            "created_at": row.get::<_, String>(7)?,
        }))
    }) {
        Ok(rows) => rows.flatten().collect(),
        Err(_) => vec![],
    };

    Json(json!({
        "profiles": profiles,
        "total": profiles.len(),
        "limit": lim,
        "offset": off,
    }))
}

#[get("/profiles/<username>")]
pub fn get_profile(
    db: &State<DbConn>,
    username: &str,
) -> Result<Json<serde_json::Value>, (Status, Json<serde_json::Value>)> {
    let conn = db.lock().unwrap();
    match load_profile(&conn, &username.to_lowercase()) {
        Some(profile) => Ok(Json(serde_json::to_value(&profile).unwrap())),
        None => Err((Status::NotFound, Json(json!({"error": format!("Profile '{}' not found", username)})))),
    }
}

#[patch("/profiles/<username>", data = "<body>")]
pub fn update_profile(
    db: &State<DbConn>,
    username: &str,
    body: Json<UpdateProfileRequest>,
    api_key: ApiKey,
) -> Result<Json<serde_json::Value>, (Status, Json<serde_json::Value>)> {
    let username = username.to_lowercase();
    let conn = db.lock().unwrap();

    if !verify_api_key(&conn, &username, &api_key.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid API key"}))));
    }

    let ts = now();
    let mut updates: Vec<String> = vec![];
    let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

    if let Some(ref v) = body.display_name {
        updates.push(format!("display_name = ?{}", values.len() + 1));
        values.push(Box::new(v.trim().to_string()));
    }
    if let Some(ref v) = body.tagline {
        if v.len() > 100 { return Err((Status::UnprocessableEntity, Json(json!({"error": "tagline max 100 chars"})))); }
        updates.push(format!("tagline = ?{}", values.len() + 1));
        values.push(Box::new(v.trim().to_string()));
    }
    if let Some(ref v) = body.bio {
        if v.len() > 2000 { return Err((Status::UnprocessableEntity, Json(json!({"error": "bio max 2000 chars"})))); }
        updates.push(format!("bio = ?{}", values.len() + 1));
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = body.third_line {
        updates.push(format!("third_line = ?{}", values.len() + 1));
        values.push(Box::new(v.trim().to_string()));
    }
    if let Some(ref v) = body.avatar_url {
        updates.push(format!("avatar_url = ?{}", values.len() + 1));
        values.push(Box::new(v.trim().to_string()));
    }
    if let Some(ref v) = body.theme {
        if !VALID_THEMES.contains(&v.as_str()) {
            return Err((Status::UnprocessableEntity, Json(json!({"error": format!("Invalid theme. Valid: {:?}", VALID_THEMES)}))));
        }
        updates.push(format!("theme = ?{}", values.len() + 1));
        values.push(Box::new(v.clone()));
    }
    if let Some(ref v) = body.particle_effect {
        if !VALID_PARTICLE_EFFECTS.contains(&v.as_str()) {
            return Err((Status::UnprocessableEntity, Json(json!({"error": format!("Invalid particle_effect. Valid: {:?}", VALID_PARTICLE_EFFECTS)}))));
        }
        updates.push(format!("particle_effect = ?{}", values.len() + 1));
        values.push(Box::new(v.clone()));
    }
    if let Some(v) = body.particle_enabled {
        updates.push(format!("particle_enabled = ?{}", values.len() + 1));
        values.push(Box::new(v as i32));
    }
    if let Some(v) = body.particle_seasonal {
        updates.push(format!("particle_seasonal = ?{}", values.len() + 1));
        values.push(Box::new(v as i32));
    }
    if let Some(ref v) = body.pubkey {
        if !v.is_empty() && !validate_pubkey(v) {
            return Err((Status::UnprocessableEntity, Json(json!({"error": "Invalid public key format (66 or 130 hex chars)"}))));
        }
        updates.push(format!("pubkey = ?{}", values.len() + 1));
        values.push(Box::new(v.clone()));
    }

    if updates.is_empty() {
        return Err((Status::UnprocessableEntity, Json(json!({"error": "No fields to update"}))));
    }

    updates.push(format!("updated_at = ?{}", values.len() + 1));
    values.push(Box::new(ts));
    values.push(Box::new(username.clone()));

    let sql = format!("UPDATE profiles SET {} WHERE username = ?{}", updates.join(", "), values.len());
    let refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();
    conn.execute(&sql, refs.as_slice())
        .map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    // Recompute score
    if let Some(profile) = load_profile(&conn, &username) {
        let has_addr = !profile.crypto_addresses.is_empty();
        let has_link = !profile.links.is_empty();
        let has_section = !profile.sections.is_empty();
        let has_skill = !profile.skills.is_empty();
        let score = compute_profile_score(
            &profile.display_name, &profile.tagline, &profile.bio,
            &profile.avatar_url, &profile.pubkey,
            has_addr, has_link, has_section, has_skill,
        );
        let _ = conn.execute(
            "UPDATE profiles SET profile_score = ?1 WHERE username = ?2",
            params![score, username],
        );
    }

    match load_profile(&conn, &username) {
        Some(profile) => Ok(Json(serde_json::to_value(&profile).unwrap())),
        None => Err((Status::InternalServerError, Json(json!({"error": "Profile not found after update"})))),
    }
}

#[delete("/profiles/<username>")]
pub fn delete_profile(
    db: &State<DbConn>,
    username: &str,
    api_key: ApiKey,
) -> Result<Json<serde_json::Value>, (Status, Json<serde_json::Value>)> {
    let username = username.to_lowercase();
    let conn = db.lock().unwrap();

    if !verify_api_key(&conn, &username, &api_key.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid API key"}))));
    }

    let rows = conn.execute("DELETE FROM profiles WHERE username = ?1", params![username])
        .map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    if rows == 0 {
        return Err((Status::NotFound, Json(json!({"error": "Profile not found"}))));
    }
    Ok(Json(json!({"status": "deleted", "username": username})))
}

// --- Profile Score ---

#[get("/profiles/<username>/score")]
pub fn get_score(
    db: &State<DbConn>,
    username: &str,
) -> Result<Json<ProfileScoreResponse>, (Status, Json<serde_json::Value>)> {
    let conn = db.lock().unwrap();
    let profile = load_profile(&conn, &username.to_lowercase())
        .ok_or_else(|| (Status::NotFound, Json(json!({"error": format!("Profile '{}' not found", username)}))))?;

    let has_addr = !profile.crypto_addresses.is_empty();
    let has_link = !profile.links.is_empty();
    let has_section = !profile.sections.is_empty();
    let has_skill = !profile.skills.is_empty();

    let score = compute_profile_score(
        &profile.display_name, &profile.tagline, &profile.bio,
        &profile.avatar_url, &profile.pubkey,
        has_addr, has_link, has_section, has_skill,
    );
    let breakdown = score_breakdown(
        &profile.display_name, &profile.tagline, &profile.bio,
        &profile.avatar_url, &profile.pubkey,
        has_addr, has_link, has_section, has_skill,
    );
    let next_steps = score_next_steps(
        &profile.display_name, &profile.tagline, &profile.bio,
        &profile.avatar_url, &profile.pubkey,
        has_addr, has_link, has_section, has_skill,
    );

    Ok(Json(ProfileScoreResponse { score, max_score: 100, breakdown, next_steps }))
}

// --- Sub-resources: Addresses ---

#[post("/profiles/<username>/addresses", data = "<body>")]
pub fn add_address(
    db: &State<DbConn>,
    username: &str,
    body: Json<AddAddressRequest>,
    api_key: ApiKey,
) -> Result<(Status, Json<serde_json::Value>), (Status, Json<serde_json::Value>)> {
    let username = username.to_lowercase();
    let conn = db.lock().unwrap();

    if !verify_api_key(&conn, &username, &api_key.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid API key"}))));
    }
    if !VALID_NETWORKS.contains(&body.network.as_str()) {
        return Err((Status::UnprocessableEntity, Json(json!({"error": format!("Invalid network. Allowed: {:?}", VALID_NETWORKS)}))));
    }
    if body.address.trim().is_empty() {
        return Err((Status::UnprocessableEntity, Json(json!({"error": "address is required"}))));
    }

    let profile_id = get_profile_id(&conn, &username)
        .ok_or_else(|| (Status::NotFound, Json(json!({"error": "Profile not found"}))))?;

    let id = Uuid::new_v4().to_string();
    let ts = now();

    conn.execute(
        "INSERT INTO crypto_addresses (id, profile_id, network, address, label, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, profile_id, body.network, body.address.trim(), body.label.clone().unwrap_or_default(), ts],
    ).map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    update_profile_score(&conn, &profile_id, &username);

    Ok((Status::Created, Json(json!({
        "id": id,
        "network": body.network,
        "address": body.address.trim(),
        "label": body.label.clone().unwrap_or_default(),
        "created_at": ts,
    }))))
}

#[delete("/profiles/<username>/addresses/<address_id>")]
pub fn delete_address(
    db: &State<DbConn>,
    username: &str,
    address_id: &str,
    api_key: ApiKey,
) -> Result<Json<serde_json::Value>, (Status, Json<serde_json::Value>)> {
    let username = username.to_lowercase();
    let conn = db.lock().unwrap();

    if !verify_api_key(&conn, &username, &api_key.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid API key"}))));
    }

    let profile_id = get_profile_id(&conn, &username)
        .ok_or_else(|| (Status::NotFound, Json(json!({"error": "Profile not found"}))))?;

    let rows = conn.execute(
        "DELETE FROM crypto_addresses WHERE id = ?1 AND profile_id = ?2",
        params![address_id, profile_id],
    ).map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    if rows == 0 {
        return Err((Status::NotFound, Json(json!({"error": "Address not found"}))));
    }
    update_profile_score(&conn, &profile_id, &username);
    Ok(Json(json!({"status": "deleted", "id": address_id})))
}

// --- Sub-resources: Links ---

#[post("/profiles/<username>/links", data = "<body>")]
pub fn add_link(
    db: &State<DbConn>,
    username: &str,
    body: Json<AddLinkRequest>,
    api_key: ApiKey,
) -> Result<(Status, Json<serde_json::Value>), (Status, Json<serde_json::Value>)> {
    let username = username.to_lowercase();
    let conn = db.lock().unwrap();

    if !verify_api_key(&conn, &username, &api_key.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid API key"}))));
    }

    let platform = body.platform.clone().unwrap_or_else(|| "website".to_string());
    if !VALID_PLATFORMS.contains(&platform.as_str()) {
        return Err((Status::UnprocessableEntity, Json(json!({"error": format!("Invalid platform. Allowed: {:?}", VALID_PLATFORMS)}))));
    }
    if body.url.trim().is_empty() || body.label.trim().is_empty() {
        return Err((Status::UnprocessableEntity, Json(json!({"error": "url and label are required"}))));
    }

    let profile_id = get_profile_id(&conn, &username)
        .ok_or_else(|| (Status::NotFound, Json(json!({"error": "Profile not found"}))))?;

    let id = Uuid::new_v4().to_string();
    let ts = now();
    let order = body.display_order.unwrap_or(0);

    conn.execute(
        "INSERT INTO profile_links (id, profile_id, url, label, platform, display_order, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, profile_id, body.url.trim(), body.label.trim(), platform, order, ts],
    ).map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    update_profile_score(&conn, &profile_id, &username);

    Ok((Status::Created, Json(json!({
        "id": id,
        "url": body.url.trim(),
        "label": body.label.trim(),
        "platform": platform,
        "display_order": order,
        "created_at": ts,
    }))))
}

#[delete("/profiles/<username>/links/<link_id>")]
pub fn delete_link(
    db: &State<DbConn>,
    username: &str,
    link_id: &str,
    api_key: ApiKey,
) -> Result<Json<serde_json::Value>, (Status, Json<serde_json::Value>)> {
    let username = username.to_lowercase();
    let conn = db.lock().unwrap();

    if !verify_api_key(&conn, &username, &api_key.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid API key"}))));
    }

    let profile_id = get_profile_id(&conn, &username)
        .ok_or_else(|| (Status::NotFound, Json(json!({"error": "Profile not found"}))))?;

    let rows = conn.execute(
        "DELETE FROM profile_links WHERE id = ?1 AND profile_id = ?2",
        params![link_id, profile_id],
    ).map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    if rows == 0 {
        return Err((Status::NotFound, Json(json!({"error": "Link not found"}))));
    }
    update_profile_score(&conn, &profile_id, &username);
    Ok(Json(json!({"status": "deleted", "id": link_id})))
}

// --- Sub-resources: Sections ---

#[post("/profiles/<username>/sections", data = "<body>")]
pub fn add_section(
    db: &State<DbConn>,
    username: &str,
    body: Json<AddSectionRequest>,
    api_key: ApiKey,
) -> Result<(Status, Json<serde_json::Value>), (Status, Json<serde_json::Value>)> {
    let username = username.to_lowercase();
    let conn = db.lock().unwrap();

    if !verify_api_key(&conn, &username, &api_key.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid API key"}))));
    }

    let section_type = body.section_type.clone().unwrap_or_else(|| "custom".to_string());
    if !VALID_SECTION_TYPES.contains(&section_type.as_str()) {
        return Err((Status::UnprocessableEntity, Json(json!({"error": format!("Invalid section_type. Allowed: {:?}", VALID_SECTION_TYPES)}))));
    }
    if body.title.trim().is_empty() {
        return Err((Status::UnprocessableEntity, Json(json!({"error": "title is required"}))));
    }
    if body.content.len() > 5000 {
        return Err((Status::UnprocessableEntity, Json(json!({"error": "content max 5000 chars"}))));
    }

    let profile_id = get_profile_id(&conn, &username)
        .ok_or_else(|| (Status::NotFound, Json(json!({"error": "Profile not found"}))))?;

    let id = Uuid::new_v4().to_string();
    let ts = now();
    let order = body.display_order.unwrap_or(0);

    conn.execute(
        "INSERT INTO profile_sections (id, profile_id, section_type, title, content, display_order, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, profile_id, section_type, body.title.trim(), body.content, order, ts],
    ).map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    update_profile_score(&conn, &profile_id, &username);

    Ok((Status::Created, Json(json!({
        "id": id,
        "section_type": section_type,
        "title": body.title.trim(),
        "content": body.content,
        "display_order": order,
        "created_at": ts,
    }))))
}

#[patch("/profiles/<username>/sections/<section_id>", data = "<body>")]
pub fn update_section(
    db: &State<DbConn>,
    username: &str,
    section_id: &str,
    body: Json<UpdateSectionRequest>,
    api_key: ApiKey,
) -> Result<Json<serde_json::Value>, (Status, Json<serde_json::Value>)> {
    let username = username.to_lowercase();
    let conn = db.lock().unwrap();

    if !verify_api_key(&conn, &username, &api_key.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid API key"}))));
    }

    let profile_id = get_profile_id(&conn, &username)
        .ok_or_else(|| (Status::NotFound, Json(json!({"error": "Profile not found"}))))?;

    let mut updates: Vec<String> = vec![];
    let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

    if let Some(ref v) = body.title {
        updates.push(format!("title = ?{}", values.len() + 1));
        values.push(Box::new(v.trim().to_string()));
    }
    if let Some(ref v) = body.content {
        if v.len() > 5000 { return Err((Status::UnprocessableEntity, Json(json!({"error": "content max 5000 chars"})))); }
        updates.push(format!("content = ?{}", values.len() + 1));
        values.push(Box::new(v.clone()));
    }
    if let Some(v) = body.display_order {
        updates.push(format!("display_order = ?{}", values.len() + 1));
        values.push(Box::new(v));
    }

    if updates.is_empty() {
        return Err((Status::UnprocessableEntity, Json(json!({"error": "No fields to update"}))));
    }

    values.push(Box::new(section_id.to_string()));
    values.push(Box::new(profile_id.clone()));

    let sql = format!(
        "UPDATE profile_sections SET {} WHERE id = ?{} AND profile_id = ?{}",
        updates.join(", "), values.len() - 1, values.len()
    );
    let refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();
    let rows = conn.execute(&sql, refs.as_slice())
        .map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    if rows == 0 {
        return Err((Status::NotFound, Json(json!({"error": "Section not found"}))));
    }

    // Return updated section
    let section: serde_json::Value = conn.query_row(
        "SELECT id, profile_id, section_type, title, content, display_order, created_at \
         FROM profile_sections WHERE id = ?1",
        params![section_id],
        |row| Ok(json!({
            "id": row.get::<_, String>(0)?,
            "profile_id": row.get::<_, String>(1)?,
            "section_type": row.get::<_, String>(2)?,
            "title": row.get::<_, String>(3)?,
            "content": row.get::<_, String>(4)?,
            "display_order": row.get::<_, i64>(5)?,
            "created_at": row.get::<_, String>(6)?,
        }))
    ).map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(section))
}

#[delete("/profiles/<username>/sections/<section_id>")]
pub fn delete_section(
    db: &State<DbConn>,
    username: &str,
    section_id: &str,
    api_key: ApiKey,
) -> Result<Json<serde_json::Value>, (Status, Json<serde_json::Value>)> {
    let username = username.to_lowercase();
    let conn = db.lock().unwrap();

    if !verify_api_key(&conn, &username, &api_key.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid API key"}))));
    }

    let profile_id = get_profile_id(&conn, &username)
        .ok_or_else(|| (Status::NotFound, Json(json!({"error": "Profile not found"}))))?;

    let rows = conn.execute(
        "DELETE FROM profile_sections WHERE id = ?1 AND profile_id = ?2",
        params![section_id, profile_id],
    ).map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    if rows == 0 {
        return Err((Status::NotFound, Json(json!({"error": "Section not found"}))));
    }
    update_profile_score(&conn, &profile_id, &username);
    Ok(Json(json!({"status": "deleted", "id": section_id})))
}

// --- Sub-resources: Skills ---

#[post("/profiles/<username>/skills", data = "<body>")]
pub fn add_skill(
    db: &State<DbConn>,
    username: &str,
    body: Json<AddSkillRequest>,
    api_key: ApiKey,
) -> Result<(Status, Json<serde_json::Value>), (Status, Json<serde_json::Value>)> {
    let username = username.to_lowercase();
    let conn = db.lock().unwrap();

    if !verify_api_key(&conn, &username, &api_key.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid API key"}))));
    }

    let skill = body.skill.trim().to_lowercase();
    if skill.is_empty() || skill.len() > 50 {
        return Err((Status::UnprocessableEntity, Json(json!({"error": "skill must be 1–50 characters"}))));
    }

    let profile_id = get_profile_id(&conn, &username)
        .ok_or_else(|| (Status::NotFound, Json(json!({"error": "Profile not found"}))))?;

    let exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM profile_skills WHERE profile_id = ?1 AND skill = ?2",
        params![profile_id, skill],
        |row| row.get(0),
    ).unwrap_or(0);

    if exists > 0 {
        return Err((Status::Conflict, Json(json!({"error": format!("Skill '{}' already added", skill)}))));
    }

    let id = Uuid::new_v4().to_string();
    let ts = now();
    conn.execute(
        "INSERT INTO profile_skills (id, profile_id, skill, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![id, profile_id, skill, ts],
    ).map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    update_profile_score(&conn, &profile_id, &username);

    Ok((Status::Created, Json(json!({
        "id": id,
        "skill": skill,
        "created_at": ts,
    }))))
}

#[delete("/profiles/<username>/skills/<skill_id>")]
pub fn delete_skill(
    db: &State<DbConn>,
    username: &str,
    skill_id: &str,
    api_key: ApiKey,
) -> Result<Json<serde_json::Value>, (Status, Json<serde_json::Value>)> {
    let username = username.to_lowercase();
    let conn = db.lock().unwrap();

    if !verify_api_key(&conn, &username, &api_key.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid API key"}))));
    }

    let profile_id = get_profile_id(&conn, &username)
        .ok_or_else(|| (Status::NotFound, Json(json!({"error": "Profile not found"}))))?;

    let rows = conn.execute(
        "DELETE FROM profile_skills WHERE id = ?1 AND profile_id = ?2",
        params![skill_id, profile_id],
    ).map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    if rows == 0 {
        return Err((Status::NotFound, Json(json!({"error": "Skill not found"}))));
    }
    update_profile_score(&conn, &profile_id, &username);
    Ok(Json(json!({"status": "deleted", "id": skill_id})))
}

// --- API key request guard ---

pub struct ApiKey(pub String);

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for ApiKey {
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        // Accept: Authorization: Bearer <key>  OR  X-API-Key: <key>
        if let Some(auth) = request.headers().get_one("Authorization") {
            if let Some(key) = auth.strip_prefix("Bearer ") {
                if !key.is_empty() {
                    return rocket::request::Outcome::Success(ApiKey(key.to_string()));
                }
            }
        }
        if let Some(key) = request.headers().get_one("X-API-Key") {
            if !key.is_empty() {
                return rocket::request::Outcome::Success(ApiKey(key.to_string()));
            }
        }
        rocket::request::Outcome::Error((Status::Unauthorized, ()))
    }
}

// --- ECDSA verification helper ---

fn verify_ecdsa_signature(pubkey_hex: &str, message: &str, sig_hex: &str) -> bool {
    use k256::ecdsa::{Signature, VerifyingKey};
    use k256::ecdsa::signature::Verifier;

    let pubkey_bytes = match hex::decode(pubkey_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let sig_bytes = match hex::decode(sig_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };

    let verifying_key = match VerifyingKey::from_sec1_bytes(&pubkey_bytes) {
        Ok(k) => k,
        Err(_) => return false,
    };

    // Try DER-encoded signature first
    if let Ok(sig) = Signature::from_der(&sig_bytes) {
        return verifying_key.verify(message.as_bytes(), &sig).is_ok();
    }
    // Fall back to compact (64-byte) format
    if let Ok(sig) = Signature::try_from(sig_bytes.as_slice()) {
        return verifying_key.verify(message.as_bytes(), &sig).is_ok();
    }
    false
}

// --- Avatar upload ---

#[post("/profiles/<username>/avatar", data = "<data>")]
pub async fn upload_avatar(
    db: &State<DbConn>,
    username: &str,
    api_key: ApiKey,
    content_type: &ContentType,
    data: Data<'_>,
) -> Result<(Status, Json<serde_json::Value>), (Status, Json<serde_json::Value>)> {
    // Validate content type is an image
    if content_type.top() != "image" {
        return Err((Status::UnprocessableEntity, Json(json!({"error": "Content-Type must be image/* (e.g. image/jpeg, image/png)"}))));
    }

    // Read up to 100KB
    let bytes = data.open(100.kibibytes()).into_bytes().await
        .map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    if !bytes.is_complete() {
        return Err((Status::PayloadTooLarge, Json(json!({"error": "Avatar must be ≤ 100KB"}))));
    }

    let mime = format!("{}/{}", content_type.top(), content_type.sub());
    let avatar_data = bytes.value;
    let username = username.to_lowercase();

    {
        let conn = db.lock().unwrap();
        if !verify_api_key(&conn, &username, &api_key.0) {
            return Err((Status::Unauthorized, Json(json!({"error": "Invalid API key"}))));
        }

        let avatar_url = format!("/avatars/{}", username);
        let ts = now();

        conn.execute(
            "UPDATE profiles SET avatar_data = ?1, avatar_mime = ?2, avatar_url = ?3, updated_at = ?4 WHERE username = ?5",
            params![avatar_data, mime, avatar_url, ts, username],
        ).map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

        // Recompute score
        if let Some(p) = load_profile(&conn, &username) {
            let has_addr = !p.crypto_addresses.is_empty();
            let has_link = !p.links.is_empty();
            let has_section = !p.sections.is_empty();
            let has_skill = !p.skills.is_empty();
            let score = crate::models::compute_profile_score(
                &p.display_name, &p.tagline, &p.bio, &p.avatar_url, &p.pubkey,
                has_addr, has_link, has_section, has_skill,
            );
            let _ = conn.execute(
                "UPDATE profiles SET profile_score = ?1 WHERE username = ?2",
                params![score, username],
            );
        }
    }

    Ok((Status::Ok, Json(json!({
        "avatar_url": format!("/avatars/{}", username),
        "mime": mime,
        "message": "Avatar uploaded successfully.",
    }))))
}

// --- Serve avatar ---

#[get("/avatars/<username>")]
pub fn serve_avatar(
    db: &State<DbConn>,
    username: &str,
) -> Result<(ContentType, Vec<u8>), Status> {
    let conn = db.lock().unwrap();
    let result: Option<(Vec<u8>, String)> = conn.query_row(
        "SELECT avatar_data, avatar_mime FROM profiles WHERE username = ?1 AND avatar_data IS NOT NULL",
        params![username.to_lowercase()],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).ok();

    match result {
        Some((data, mime)) => {
            let ct = ContentType::parse_flexible(&mime).unwrap_or(ContentType::JPEG);
            Ok((ct, data))
        },
        None => Err(Status::NotFound),
    }
}

// --- Identity verification ---

#[get("/profiles/<username>/challenge")]
pub fn get_challenge(
    db: &State<DbConn>,
    username: &str,
) -> Result<Json<crate::models::ChallengeResponse>, (Status, Json<serde_json::Value>)> {
    let conn = db.lock().unwrap();
    let profile = load_profile(&conn, &username.to_lowercase())
        .ok_or_else(|| (Status::NotFound, Json(json!({"error": format!("Profile '{}' not found", username)}))))?;

    if profile.pubkey.is_empty() {
        return Err((Status::UnprocessableEntity, Json(json!({
            "error": "Profile has no secp256k1 public key. Set one via PATCH /api/v1/profiles/{username} first."
        }))));
    }

    // Generate random 32-byte challenge
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let challenge = hex::encode(bytes);

    let id = Uuid::new_v4().to_string();
    let ts = now();
    let expires_at = (Utc::now() + chrono::Duration::minutes(5))
        .format("%Y-%m-%dT%H:%M:%SZ").to_string();

    conn.execute(
        "INSERT INTO identity_challenges (id, profile_id, challenge, expires_at, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, profile.id, challenge, expires_at, ts],
    ).map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    Ok(Json(crate::models::ChallengeResponse {
        challenge,
        expires_in_seconds: 300,
        instructions: "Sign this challenge with your secp256k1 private key (ECDSA-SHA256). \
            Send the DER or compact 64-byte hex-encoded signature to POST /api/v1/profiles/{username}/verify.".to_string(),
    }))
}

#[post("/profiles/<username>/verify", data = "<body>")]
pub fn verify_signature(
    db: &State<DbConn>,
    username: &str,
    body: Json<crate::models::VerifySignatureRequest>,
) -> Result<Json<crate::models::VerifyResponse>, (Status, Json<serde_json::Value>)> {
    let username = username.to_lowercase();
    let conn = db.lock().unwrap();

    let profile = load_profile(&conn, &username)
        .ok_or_else(|| (Status::NotFound, Json(json!({"error": format!("Profile '{}' not found", username)}))))?;

    if profile.pubkey.is_empty() {
        return Err((Status::UnprocessableEntity, Json(json!({
            "error": "Profile has no secp256k1 public key."
        }))));
    }

    // Get most recent valid unused challenge
    let result: Option<(String, String)> = conn.query_row(
        "SELECT id, challenge FROM identity_challenges \
         WHERE profile_id = ?1 AND used = 0 AND expires_at > ?2 \
         ORDER BY created_at DESC LIMIT 1",
        params![profile.id, now()],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).ok();

    let (challenge_id, challenge) = result.ok_or_else(|| {
        (Status::Gone, Json(json!({"error": "No valid challenge found. Request a new challenge first."})))
    })?;

    // Mark as used (single-use prevents replay attacks)
    let _ = conn.execute(
        "UPDATE identity_challenges SET used = 1 WHERE id = ?1",
        params![challenge_id],
    );

    let verified = verify_ecdsa_signature(&profile.pubkey, &challenge, &body.signature);

    Ok(Json(crate::models::VerifyResponse {
        verified,
        username,
        timestamp: now(),
    }))
}

// --- Discovery endpoints ---

#[get("/llms.txt")]
pub fn llms_txt() -> (ContentType, &'static str) {
    (ContentType::Plain, r#"# Agent Profile Service

> Canonical "About Me" profile pages for AI agents.

Provides publicly-accessible identity pages with secp256k1 cryptographic verification.
Each agent gets a profile URL that serves JSON to agents and HTML to humans.

## Registration

POST /api/v1/register
Body: { "username": "your-username" }
Returns: { "api_key": "...", "username": "...", "profile_url": "/username" }

## Profile Access

GET /{username}         — JSON for agents (auto-detected), HTML for humans
GET /api/v1/profiles/{username} — always JSON

## Key Endpoints

- PATCH /api/v1/profiles/{username}    — Update profile fields
- POST /api/v1/profiles/{username}/avatar    — Upload avatar (≤100KB)
- POST /api/v1/profiles/{username}/links     — Add a link
- POST /api/v1/profiles/{username}/addresses — Add a crypto address
- POST /api/v1/profiles/{username}/sections  — Add a profile section
- GET  /api/v1/profiles/{username}/challenge — Get identity challenge
- POST /api/v1/profiles/{username}/verify    — Verify via secp256k1 signature
- GET  /api/v1/profiles/{username}/score     — Profile completeness score

## Authentication

Bearer token or X-API-Key header with your API key.

## Content Negotiation

GET /{username} returns JSON when User-Agent signals an AI client
or when Accept: application/json is set.

## Source

https://github.com/Humans-Not-Required/agent-profile
"#)
}

#[get("/openapi.json")]
pub fn openapi_json() -> (ContentType, &'static str) {
    (ContentType::JSON, include_str!("../../openapi.json"))
}

#[get("/.well-known/skills/index.json")]
pub fn skills_index() -> (ContentType, String) {
    (ContentType::JSON, json!({
        "service": "agent-profile",
        "version": env!("CARGO_PKG_VERSION"),
        "skills": [
            {
                "id": "register-profile",
                "name": "Register agent profile",
                "endpoint": "POST /api/v1/register",
                "description": "Create a new agent profile page"
            },
            {
                "id": "get-profile",
                "name": "Get agent profile",
                "endpoint": "GET /api/v1/profiles/{username}",
                "description": "Retrieve a full agent profile with links, sections, and crypto addresses"
            },
            {
                "id": "verify-identity",
                "name": "Verify agent identity",
                "endpoint": "GET+POST /api/v1/profiles/{username}/challenge+verify",
                "description": "Cryptographically verify an agent's identity via secp256k1 signature"
            }
        ]
    }).to_string())
}
