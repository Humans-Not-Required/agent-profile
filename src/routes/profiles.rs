use rocket::{get, post, patch, delete, State, http::Status, serde::json::Json};
use rusqlite::{Connection, params};
use serde_json::json;
use sha2::{Sha256, Digest};
use uuid::Uuid;
use chrono::Utc;
use std::sync::Mutex;

use crate::models::*;

type DbConn = Mutex<Connection>;

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

fn gen_token() -> String {
    Uuid::new_v4().to_string().replace('-', "")
}

fn now() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn load_profile(conn: &Connection, slug: &str) -> Option<Profile> {
    let result = conn.query_row(
        "SELECT id, slug, display_name, bio, avatar_url, created_at, updated_at FROM profiles WHERE slug = ?1",
        params![slug],
        |row| Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
        )),
    ).ok()?;

    let (id, slug, display_name, bio, avatar_url, created_at, updated_at) = result;

    // Load crypto addresses
    let mut stmt = conn.prepare(
        "SELECT id, profile_id, network, address, label, verified, created_at FROM crypto_addresses WHERE profile_id = ?1 ORDER BY created_at"
    ).ok()?;
    let crypto_addresses: Vec<CryptoAddress> = stmt.query_map(params![&id], |row| {
        Ok(CryptoAddress {
            id: row.get(0)?,
            profile_id: row.get(1)?,
            network: row.get(2)?,
            address: row.get(3)?,
            label: row.get(4)?,
            verified: row.get::<_, i32>(5)? != 0,
            created_at: row.get(6)?,
        })
    }).ok()?.flatten().collect();

    // Load links
    let mut stmt = conn.prepare(
        "SELECT id, profile_id, link_type, label, value, created_at FROM profile_links WHERE profile_id = ?1 ORDER BY created_at"
    ).ok()?;
    let links: Vec<ProfileLink> = stmt.query_map(params![&id], |row| {
        Ok(ProfileLink {
            id: row.get(0)?,
            profile_id: row.get(1)?,
            link_type: row.get(2)?,
            label: row.get(3)?,
            value: row.get(4)?,
            created_at: row.get(5)?,
        })
    }).ok()?.flatten().collect();

    // Load skills
    let mut stmt = conn.prepare(
        "SELECT id, profile_id, skill, created_at FROM profile_skills WHERE profile_id = ?1 ORDER BY created_at"
    ).ok()?;
    let skills: Vec<ProfileSkill> = stmt.query_map(params![&id], |row| {
        Ok(ProfileSkill {
            id: row.get(0)?,
            profile_id: row.get(1)?,
            skill: row.get(2)?,
            created_at: row.get(3)?,
        })
    }).ok()?.flatten().collect();

    Some(Profile {
        id,
        slug,
        display_name,
        bio,
        avatar_url,
        created_at,
        updated_at,
        crypto_addresses,
        links,
        skills,
    })
}

fn verify_manage_token(conn: &Connection, slug: &str, token: &str) -> bool {
    let hashed = hash_token(token);
    conn.query_row(
        "SELECT COUNT(*) FROM profiles WHERE slug = ?1 AND manage_token = ?2",
        params![slug, hashed],
        |row| row.get::<_, i64>(0),
    ).unwrap_or(0) > 0
}

// --- Endpoints ---

#[get("/health")]
pub fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        service: "agent-profile".to_string(),
    })
}

#[post("/profiles", data = "<body>")]
pub fn create_profile(
    db: &State<DbConn>,
    body: Json<CreateProfileRequest>,
) -> Result<(Status, Json<serde_json::Value>), (Status, Json<serde_json::Value>)> {
    let slug = validate_slug(&body.slug).map_err(|e| {
        (Status::UnprocessableEntity, Json(json!({"error": e})))
    })?;

    if body.display_name.trim().is_empty() {
        return Err((Status::UnprocessableEntity, Json(json!({"error": "display_name is required"}))));
    }
    if body.display_name.len() > 100 {
        return Err((Status::UnprocessableEntity, Json(json!({"error": "display_name max 100 chars"}))));
    }
    let bio = body.bio.clone().unwrap_or_default();
    if bio.len() > 2000 {
        return Err((Status::UnprocessableEntity, Json(json!({"error": "bio max 2000 chars"}))));
    }

    let token = gen_token();
    let hashed = hash_token(&token);
    let id = Uuid::new_v4().to_string();
    let ts = now();

    let conn = db.lock().unwrap();
    match conn.execute(
        "INSERT INTO profiles (id, slug, display_name, bio, avatar_url, manage_token, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id, slug, body.display_name.trim(), bio,
            body.avatar_url.clone().unwrap_or_default(),
            hashed, ts, ts
        ],
    ) {
        Ok(_) => Ok((Status::Created, Json(json!({
            "slug": slug,
            "manage_token": token,
            "profile_url": format!("/agents/{}", slug),
            "json_url": format!("/api/v1/profiles/{}", slug),
        })))),
        Err(rusqlite::Error::SqliteFailure(e, _)) if e.extended_code == 2067 => {
            Err((Status::Conflict, Json(json!({"error": format!("Slug '{}' already taken", slug)}))))
        },
        Err(e) => Err((Status::InternalServerError, Json(json!({"error": e.to_string()})))),
    }
}

#[get("/profiles?<q>&<skill>&<network>&<limit>&<offset>")]
pub fn list_profiles(
    db: &State<DbConn>,
    q: Option<&str>,
    skill: Option<&str>,
    network: Option<&str>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Json<serde_json::Value> {
    let conn = db.lock().unwrap();
    let lim = limit.unwrap_or(20).min(100);
    let off = offset.unwrap_or(0);

    // Build query dynamically
    let mut query = "SELECT DISTINCT p.id, p.slug, p.display_name, p.bio, p.avatar_url, p.created_at, p.updated_at FROM profiles p".to_string();
    let mut conditions: Vec<String> = vec![];
    let mut params_list: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

    if skill.is_some() {
        query.push_str(" LEFT JOIN profile_skills ps ON ps.profile_id = p.id");
        conditions.push(format!("LOWER(ps.skill) = LOWER(?{})", params_list.len() + 1));
        params_list.push(Box::new(skill.unwrap().to_string()));
    }
    if network.is_some() {
        query.push_str(" LEFT JOIN crypto_addresses ca ON ca.profile_id = p.id");
        conditions.push(format!("LOWER(ca.network) = LOWER(?{})", params_list.len() + 1));
        params_list.push(Box::new(network.unwrap().to_string()));
    }
    if let Some(q_val) = q {
        let search = format!("%{}%", q_val.to_lowercase());
        conditions.push(format!("(LOWER(p.slug) LIKE ?{0} OR LOWER(p.display_name) LIKE ?{0} OR LOWER(p.bio) LIKE ?{0})", params_list.len() + 1));
        params_list.push(Box::new(search));
    }

    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }
    query.push_str(&format!(" ORDER BY p.created_at DESC LIMIT {} OFFSET {}", lim, off));

    let refs: Vec<&dyn rusqlite::types::ToSql> = params_list.iter().map(|p| p.as_ref()).collect();

    let mut stmt = match conn.prepare(&query) {
        Ok(s) => s,
        Err(e) => return Json(json!({"error": e.to_string(), "profiles": []})),
    };

    let profiles: Vec<serde_json::Value> = match stmt.query_map(refs.as_slice(), |row| {
        Ok(json!({
            "id": row.get::<_, String>(0)?,
            "slug": row.get::<_, String>(1)?,
            "display_name": row.get::<_, String>(2)?,
            "bio": row.get::<_, String>(3)?,
            "avatar_url": row.get::<_, String>(4)?,
            "created_at": row.get::<_, String>(5)?,
            "updated_at": row.get::<_, String>(6)?,
        }))
    }) {
        Ok(rows) => rows.flatten().collect(),
        Err(_) => vec![],
    };

    let total = profiles.len();
    Json(json!({
        "profiles": profiles,
        "total": total,
        "limit": lim,
        "offset": off,
    }))
}

#[get("/profiles/<slug>")]
pub fn get_profile(
    db: &State<DbConn>,
    slug: &str,
) -> Result<Json<serde_json::Value>, (Status, Json<serde_json::Value>)> {
    let conn = db.lock().unwrap();
    match load_profile(&conn, &slug.to_lowercase()) {
        Some(profile) => Ok(Json(serde_json::to_value(&profile).unwrap())),
        None => Err((Status::NotFound, Json(json!({"error": format!("Profile '{}' not found", slug)})))),
    }
}

#[patch("/profiles/<slug>", data = "<body>")]
pub fn update_profile(
    db: &State<DbConn>,
    slug: &str,
    body: Json<UpdateProfileRequest>,
    manage_token: ManageToken,
) -> Result<Json<serde_json::Value>, (Status, Json<serde_json::Value>)> {
    let slug = slug.to_lowercase();
    let conn = db.lock().unwrap();

    if !verify_manage_token(&conn, &slug, &manage_token.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid manage token"}))));
    }

    let ts = now();
    let mut updates: Vec<String> = vec![];
    let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

    if let Some(ref name) = body.display_name {
        if name.trim().is_empty() {
            return Err((Status::UnprocessableEntity, Json(json!({"error": "display_name cannot be empty"}))));
        }
        updates.push(format!("display_name = ?{}", values.len() + 1));
        values.push(Box::new(name.trim().to_string()));
    }
    if let Some(ref bio) = body.bio {
        if bio.len() > 2000 {
            return Err((Status::UnprocessableEntity, Json(json!({"error": "bio max 2000 chars"}))));
        }
        updates.push(format!("bio = ?{}", values.len() + 1));
        values.push(Box::new(bio.clone()));
    }
    if let Some(ref url) = body.avatar_url {
        updates.push(format!("avatar_url = ?{}", values.len() + 1));
        values.push(Box::new(url.clone()));
    }

    if updates.is_empty() {
        return Err((Status::UnprocessableEntity, Json(json!({"error": "No fields to update"}))));
    }

    updates.push(format!("updated_at = ?{}", values.len() + 1));
    values.push(Box::new(ts));
    values.push(Box::new(slug.clone()));

    let sql = format!("UPDATE profiles SET {} WHERE slug = ?{}", updates.join(", "), values.len());
    let refs: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();

    conn.execute(&sql, refs.as_slice()).map_err(|e| {
        (Status::InternalServerError, Json(json!({"error": e.to_string()})))
    })?;

    match load_profile(&conn, &slug) {
        Some(profile) => Ok(Json(serde_json::to_value(&profile).unwrap())),
        None => Err((Status::InternalServerError, Json(json!({"error": "Profile not found after update"})))),
    }
}

#[delete("/profiles/<slug>")]
pub fn delete_profile(
    db: &State<DbConn>,
    slug: &str,
    manage_token: ManageToken,
) -> Result<Json<serde_json::Value>, (Status, Json<serde_json::Value>)> {
    let slug = slug.to_lowercase();
    let conn = db.lock().unwrap();

    if !verify_manage_token(&conn, &slug, &manage_token.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid manage token"}))));
    }

    let rows = conn.execute("DELETE FROM profiles WHERE slug = ?1", params![slug])
        .map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    if rows == 0 {
        return Err((Status::NotFound, Json(json!({"error": "Profile not found"}))));
    }

    Ok(Json(json!({"status": "deleted", "slug": slug})))
}

// --- Sub-resources ---

#[post("/profiles/<slug>/addresses", data = "<body>")]
pub fn add_address(
    db: &State<DbConn>,
    slug: &str,
    body: Json<AddAddressRequest>,
    manage_token: ManageToken,
) -> Result<(Status, Json<serde_json::Value>), (Status, Json<serde_json::Value>)> {
    let slug = slug.to_lowercase();
    let conn = db.lock().unwrap();

    if !verify_manage_token(&conn, &slug, &manage_token.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid manage token"}))));
    }
    if !validate_network(&body.network) {
        return Err((Status::UnprocessableEntity, Json(json!({"error": "Invalid network. Allowed: bitcoin, lightning, ethereum, solana, nostr, other"}))));
    }
    if body.address.trim().is_empty() {
        return Err((Status::UnprocessableEntity, Json(json!({"error": "address is required"}))));
    }

    let profile_id: String = conn.query_row(
        "SELECT id FROM profiles WHERE slug = ?1", params![slug], |row| row.get(0)
    ).map_err(|_| (Status::NotFound, Json(json!({"error": "Profile not found"}))))?;

    let id = Uuid::new_v4().to_string();
    let ts = now();

    conn.execute(
        "INSERT INTO crypto_addresses (id, profile_id, network, address, label, verified, created_at) VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)",
        params![id, profile_id, body.network, body.address.trim(), body.label.clone().unwrap_or_default(), ts],
    ).map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    Ok((Status::Created, Json(json!({
        "id": id,
        "network": body.network,
        "address": body.address.trim(),
        "label": body.label.clone().unwrap_or_default(),
        "verified": false,
        "created_at": ts,
    }))))
}

#[delete("/profiles/<slug>/addresses/<address_id>")]
pub fn delete_address(
    db: &State<DbConn>,
    slug: &str,
    address_id: &str,
    manage_token: ManageToken,
) -> Result<Json<serde_json::Value>, (Status, Json<serde_json::Value>)> {
    let slug = slug.to_lowercase();
    let conn = db.lock().unwrap();

    if !verify_manage_token(&conn, &slug, &manage_token.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid manage token"}))));
    }

    let profile_id: String = conn.query_row(
        "SELECT id FROM profiles WHERE slug = ?1", params![slug], |row| row.get(0)
    ).map_err(|_| (Status::NotFound, Json(json!({"error": "Profile not found"}))))?;

    let rows = conn.execute(
        "DELETE FROM crypto_addresses WHERE id = ?1 AND profile_id = ?2",
        params![address_id, profile_id],
    ).map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    if rows == 0 {
        return Err((Status::NotFound, Json(json!({"error": "Address not found"}))));
    }

    Ok(Json(json!({"status": "deleted", "id": address_id})))
}

#[post("/profiles/<slug>/links", data = "<body>")]
pub fn add_link(
    db: &State<DbConn>,
    slug: &str,
    body: Json<AddLinkRequest>,
    manage_token: ManageToken,
) -> Result<(Status, Json<serde_json::Value>), (Status, Json<serde_json::Value>)> {
    let slug = slug.to_lowercase();
    let conn = db.lock().unwrap();

    if !verify_manage_token(&conn, &slug, &manage_token.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid manage token"}))));
    }
    if !validate_link_type(&body.link_type) {
        return Err((Status::UnprocessableEntity, Json(json!({"error": "Invalid link_type. Allowed: nostr, moltbook, github, telegram, email, website, twitter, custom"}))));
    }
    if body.label.trim().is_empty() || body.value.trim().is_empty() {
        return Err((Status::UnprocessableEntity, Json(json!({"error": "label and value are required"}))));
    }

    let profile_id: String = conn.query_row(
        "SELECT id FROM profiles WHERE slug = ?1", params![slug], |row| row.get(0)
    ).map_err(|_| (Status::NotFound, Json(json!({"error": "Profile not found"}))))?;

    let id = Uuid::new_v4().to_string();
    let ts = now();

    conn.execute(
        "INSERT INTO profile_links (id, profile_id, link_type, label, value, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, profile_id, body.link_type, body.label.trim(), body.value.trim(), ts],
    ).map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    Ok((Status::Created, Json(json!({
        "id": id,
        "link_type": body.link_type,
        "label": body.label.trim(),
        "value": body.value.trim(),
        "created_at": ts,
    }))))
}

#[delete("/profiles/<slug>/links/<link_id>")]
pub fn delete_link(
    db: &State<DbConn>,
    slug: &str,
    link_id: &str,
    manage_token: ManageToken,
) -> Result<Json<serde_json::Value>, (Status, Json<serde_json::Value>)> {
    let slug = slug.to_lowercase();
    let conn = db.lock().unwrap();

    if !verify_manage_token(&conn, &slug, &manage_token.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid manage token"}))));
    }

    let profile_id: String = conn.query_row(
        "SELECT id FROM profiles WHERE slug = ?1", params![slug], |row| row.get(0)
    ).map_err(|_| (Status::NotFound, Json(json!({"error": "Profile not found"}))))?;

    let rows = conn.execute(
        "DELETE FROM profile_links WHERE id = ?1 AND profile_id = ?2",
        params![link_id, profile_id],
    ).map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    if rows == 0 {
        return Err((Status::NotFound, Json(json!({"error": "Link not found"}))));
    }

    Ok(Json(json!({"status": "deleted", "id": link_id})))
}

#[post("/profiles/<slug>/skills", data = "<body>")]
pub fn add_skill(
    db: &State<DbConn>,
    slug: &str,
    body: Json<AddSkillRequest>,
    manage_token: ManageToken,
) -> Result<(Status, Json<serde_json::Value>), (Status, Json<serde_json::Value>)> {
    let slug = slug.to_lowercase();
    let conn = db.lock().unwrap();

    if !verify_manage_token(&conn, &slug, &manage_token.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid manage token"}))));
    }
    let skill = body.skill.trim().to_lowercase();
    if skill.is_empty() || skill.len() > 50 {
        return Err((Status::UnprocessableEntity, Json(json!({"error": "skill must be 1–50 characters"}))));
    }

    let profile_id: String = conn.query_row(
        "SELECT id FROM profiles WHERE slug = ?1", params![slug], |row| row.get(0)
    ).map_err(|_| (Status::NotFound, Json(json!({"error": "Profile not found"}))))?;

    // Check for duplicate skill on this profile
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

    Ok((Status::Created, Json(json!({
        "id": id,
        "skill": skill,
        "created_at": ts,
    }))))
}

#[delete("/profiles/<slug>/skills/<skill_id>")]
pub fn delete_skill(
    db: &State<DbConn>,
    slug: &str,
    skill_id: &str,
    manage_token: ManageToken,
) -> Result<Json<serde_json::Value>, (Status, Json<serde_json::Value>)> {
    let slug = slug.to_lowercase();
    let conn = db.lock().unwrap();

    if !verify_manage_token(&conn, &slug, &manage_token.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid manage token"}))));
    }

    let profile_id: String = conn.query_row(
        "SELECT id FROM profiles WHERE slug = ?1", params![slug], |row| row.get(0)
    ).map_err(|_| (Status::NotFound, Json(json!({"error": "Profile not found"}))))?;

    let rows = conn.execute(
        "DELETE FROM profile_skills WHERE id = ?1 AND profile_id = ?2",
        params![skill_id, profile_id],
    ).map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    if rows == 0 {
        return Err((Status::NotFound, Json(json!({"error": "Skill not found"}))));
    }

    Ok(Json(json!({"status": "deleted", "id": skill_id})))
}

// Request guard for manage token from header

pub struct ManageToken(pub String);

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for ManageToken {
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        match request.headers().get_one("X-Manage-Token") {
            Some(token) if !token.is_empty() => rocket::request::Outcome::Success(ManageToken(token.to_string())),
            _ => rocket::request::Outcome::Error((Status::Unauthorized, ())),
        }
    }
}
