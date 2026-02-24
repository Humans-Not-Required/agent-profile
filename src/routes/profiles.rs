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
         profile_score, view_count, created_at, updated_at \
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
            row.get::<_, i64>(13)?,     // view_count
            row.get::<_, String>(14)?,  // created_at
            row.get::<_, String>(15)?,  // updated_at
        )),
    ).ok()?;

    let (id, username, display_name, tagline, bio, third_line, avatar_url,
         theme, particle_effect, particle_enabled, particle_seasonal, pubkey,
         profile_score, view_count, created_at, updated_at) = result;

    let crypto_addresses = load_addresses(conn, &id);
    let links = load_links(conn, &id);
    let sections = load_sections(conn, &id);
    let skills = load_skills(conn, &id);
    let endorsements = load_endorsements(conn, &id);

    Some(Profile {
        id, username, display_name, tagline, bio, third_line, avatar_url,
        theme, particle_effect,
        particle_enabled: particle_enabled != 0,
        particle_seasonal: particle_seasonal != 0,
        pubkey, profile_score, view_count, created_at, updated_at,
        crypto_addresses, links, sections, skills, endorsements,
    })
}

/// Load all profiles (lightweight: only skills populated, other sub-resources empty).
/// Used by the landing page to render profile cards.
pub(crate) fn list_all_profiles(conn: &Connection) -> Vec<Profile> {
    let mut stmt = match conn.prepare(
        "SELECT id, username, display_name, tagline, bio, third_line, avatar_url, \
         theme, particle_effect, particle_enabled, particle_seasonal, pubkey, \
         profile_score, view_count, created_at, updated_at \
         FROM profiles ORDER BY profile_score DESC, created_at DESC LIMIT 100"
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let mapped = match stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, String>(7)?,
            row.get::<_, String>(8)?,
            row.get::<_, i32>(9)?,
            row.get::<_, i32>(10)?,
            row.get::<_, String>(11)?,
            row.get::<_, i64>(12)?,
            row.get::<_, i64>(13)?,
            row.get::<_, String>(14)?,
            row.get::<_, String>(15)?,
        ))
    }) {
        Ok(m) => m,
        Err(_) => return vec![],
    };

    mapped.flatten()
        .map(|(id, username, display_name, tagline, bio, third_line, avatar_url,
               theme, particle_effect, particle_enabled, particle_seasonal, pubkey,
               profile_score, view_count, created_at, updated_at)| {
            let skills = load_skills(conn, &id);
            Profile {
                id, username, display_name, tagline, bio, third_line, avatar_url,
                theme, particle_effect,
                particle_enabled: particle_enabled != 0,
                particle_seasonal: particle_seasonal != 0,
                pubkey, profile_score, view_count, created_at, updated_at,
                crypto_addresses: vec![],
                links: vec![],
                sections: vec![],
                skills,
                endorsements: vec![],
            }
        })
        .collect()
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

fn load_endorsements(conn: &Connection, profile_id: &str) -> Vec<Endorsement> {
    let mut stmt = conn.prepare(
        "SELECT id, endorsee_id, endorser_username, message, signature, verified, created_at \
         FROM endorsements WHERE endorsee_id = ?1 ORDER BY created_at DESC"
    ).unwrap();
    stmt.query_map(params![profile_id], |row| {
        Ok(Endorsement {
            id: row.get(0)?,
            endorsee_id: row.get(1)?,
            endorser_username: row.get(2)?,
            message: row.get(3)?,
            signature: row.get(4)?,
            verified: row.get::<_, i32>(5)? != 0,
            created_at: row.get(6)?,
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
    _rl: crate::ratelimit::RegisterRateLimit,
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

#[get("/profiles?<q>&<theme>&<skill>&<has_pubkey>&<sort>&<limit>&<offset>")]
pub fn list_profiles(
    db: &State<DbConn>,
    q: Option<&str>,
    theme: Option<&str>,
    skill: Option<&str>,
    has_pubkey: Option<bool>,
    sort: Option<&str>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Json<serde_json::Value> {
    let conn = db.lock().unwrap();
    let lim = limit.unwrap_or(20).min(100);
    let off = offset.unwrap_or(0);

    let mut query = "SELECT id, username, display_name, tagline, avatar_url, \
                     theme, profile_score, view_count, created_at, updated_at FROM profiles".to_string();
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
    if let Some(s) = skill {
        // Subquery: profiles that have at least one skill matching (case-insensitive)
        conditions.push(format!(
            "id IN (SELECT profile_id FROM profile_skills WHERE LOWER(skill) = LOWER(?{}))",
            values.len() + 1
        ));
        values.push(Box::new(s.to_string()));
    }
    if has_pubkey == Some(true) {
        conditions.push("pubkey != ''".to_string());
    }

    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }

    // Sort options: popular (views), newest, active (recently updated), score (default)
    let order_clause = match sort.unwrap_or("score") {
        "popular" | "views" => "ORDER BY view_count DESC, profile_score DESC",
        "newest" | "new" => "ORDER BY created_at DESC",
        "active" | "updated" => "ORDER BY updated_at DESC",
        _ => "ORDER BY profile_score DESC, created_at DESC",
    };
    query.push_str(&format!(" {} LIMIT {} OFFSET {}", order_clause, lim, off));

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
            "view_count": row.get::<_, i64>(7)?,
            "created_at": row.get::<_, String>(8)?,
            "updated_at": row.get::<_, String>(9)?,
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

/// GET /api/v1/skills
/// List all skill tags registered across all agent profiles, sorted by usage count descending.
/// Optional ?q= for substring search within skill names.
#[get("/skills?<q>&<limit>")]
pub fn list_skills(
    db: &State<DbConn>,
    q: Option<&str>,
    limit: Option<i64>,
) -> Json<serde_json::Value> {
    let conn = db.lock().unwrap();
    let lim = limit.unwrap_or(50).min(200);

    let (sql, skill_filter) = if let Some(filter) = q {
        (
            "SELECT LOWER(skill) as skill_lower, COUNT(*) as count \
             FROM profile_skills \
             WHERE LOWER(skill) LIKE ?1 \
             GROUP BY skill_lower \
             ORDER BY count DESC, skill_lower ASC \
             LIMIT ?2".to_string(),
            Some(format!("%{}%", filter.to_lowercase())),
        )
    } else {
        (
            "SELECT LOWER(skill) as skill_lower, COUNT(*) as count \
             FROM profile_skills \
             GROUP BY skill_lower \
             ORDER BY count DESC, skill_lower ASC \
             LIMIT ?1".to_string(),
            None,
        )
    };

    let skills: Vec<serde_json::Value> = if let Some(ref f) = skill_filter {
        let mut stmt = conn.prepare(&sql).unwrap();
        stmt.query_map(rusqlite::params![f, lim], |row| {
            Ok(json!({
                "skill": row.get::<_, String>(0)?,
                "count": row.get::<_, i64>(1)?,
            }))
        }).unwrap().flatten().collect()
    } else {
        let mut stmt = conn.prepare(&sql).unwrap();
        stmt.query_map(rusqlite::params![lim], |row| {
            Ok(json!({
                "skill": row.get::<_, String>(0)?,
                "count": row.get::<_, i64>(1)?,
            }))
        }).unwrap().flatten().collect()
    };

    let total_distinct = conn.query_row(
        "SELECT COUNT(DISTINCT LOWER(skill)) FROM profile_skills",
        [],
        |row| row.get::<_, i64>(0),
    ).unwrap_or(0);

    Json(json!({
        "skills": skills,
        "total_distinct": total_distinct,
        "showing": skills.len(),
        "limit": lim,
    }))
}

/// GET /api/v1/stats
/// Aggregate statistics for the service — useful for dashboards and agent discovery.
#[get("/stats")]
pub fn get_stats(db: &State<DbConn>) -> Json<serde_json::Value> {
    let conn = db.lock().unwrap();

    let total_profiles: i64 = conn.query_row(
        "SELECT COUNT(*) FROM profiles", [], |r| r.get(0)
    ).unwrap_or(0);

    let profiles_with_pubkey: i64 = conn.query_row(
        "SELECT COUNT(*) FROM profiles WHERE pubkey != ''", [], |r| r.get(0)
    ).unwrap_or(0);

    let total_skills: i64 = conn.query_row(
        "SELECT COUNT(*) FROM profile_skills", [], |r| r.get(0)
    ).unwrap_or(0);

    let distinct_skills: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT LOWER(skill)) FROM profile_skills", [], |r| r.get(0)
    ).unwrap_or(0);

    let total_links: i64 = conn.query_row(
        "SELECT COUNT(*) FROM profile_links", [], |r| r.get(0)
    ).unwrap_or(0);

    let total_addresses: i64 = conn.query_row(
        "SELECT COUNT(*) FROM crypto_addresses", [], |r| r.get(0)
    ).unwrap_or(0);

    let total_endorsements: i64 = conn.query_row(
        "SELECT COUNT(*) FROM endorsements", [], |r| r.get(0)
    ).unwrap_or(0);

    let verified_endorsements: i64 = conn.query_row(
        "SELECT COUNT(*) FROM endorsements WHERE verified = 1", [], |r| r.get(0)
    ).unwrap_or(0);

    let avg_score: f64 = conn.query_row(
        "SELECT AVG(CAST(profile_score AS REAL)) FROM profiles", [], |r| r.get(0)
    ).unwrap_or(0.0);

    // Top 5 skills
    let mut top_stmt = conn.prepare(
        "SELECT LOWER(skill), COUNT(*) as c FROM profile_skills \
         GROUP BY LOWER(skill) ORDER BY c DESC LIMIT 5"
    ).unwrap();
    let top_skills: Vec<serde_json::Value> = top_stmt.query_map([], |row| {
        Ok(json!({"skill": row.get::<_, String>(0)?, "count": row.get::<_, i64>(1)?}))
    }).unwrap().flatten().collect();

    Json(json!({
        "profiles": {
            "total": total_profiles,
            "with_pubkey": profiles_with_pubkey,
            "avg_score": (avg_score * 10.0).round() / 10.0,
        },
        "skills": {
            "total_tags": total_skills,
            "distinct": distinct_skills,
            "top": top_skills,
        },
        "links": { "total": total_links },
        "addresses": { "total": total_addresses },
        "endorsements": {
            "total": total_endorsements,
            "verified": verified_endorsements,
        },
        "service": {
            "version": env!("CARGO_PKG_VERSION"),
            "name": "agent-profile",
        }
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

// --- Export / Import ---

/// GET /api/v1/profiles/{username}/export
/// Returns a portable JSON document of the full profile, suitable for backup or migration.
/// Requires API key (only the profile owner can export).
/// Strips internal IDs, includes format version for forward compatibility.
#[get("/profiles/<username>/export")]
pub fn export_profile(
    db: &State<DbConn>,
    username: &str,
    api_key: ApiKey,
) -> Result<Json<serde_json::Value>, (Status, Json<serde_json::Value>)> {
    let username = username.to_lowercase();
    let conn = db.lock().unwrap();

    // Verify API key
    let stored_hash: String = conn.query_row(
        "SELECT api_key_hash FROM profiles WHERE username = ?1",
        params![username],
        |row| row.get(0),
    ).map_err(|_| (Status::NotFound, Json(json!({"error": format!("Profile '{}' not found", username)}))))?;

    if stored_hash != hash_key(&api_key.0) {
        return Err((Status::Unauthorized, Json(json!({"error": "Invalid API key"}))));
    }

    let profile = load_profile(&conn, &username)
        .ok_or_else(|| (Status::NotFound, Json(json!({"error": "Profile not found"}))))?;

    // Build portable export document
    let links: Vec<serde_json::Value> = profile.links.iter().map(|l| json!({
        "url": l.url,
        "label": l.label,
        "platform": l.platform,
    })).collect();

    let sections: Vec<serde_json::Value> = profile.sections.iter().map(|s| json!({
        "title": s.title,
        "content": s.content,
        "section_type": s.section_type,
    })).collect();

    let skills: Vec<String> = profile.skills.iter().map(|s| s.skill.clone()).collect();

    let addresses: Vec<serde_json::Value> = profile.crypto_addresses.iter().map(|a| json!({
        "network": a.network,
        "address": a.address,
        "label": a.label,
    })).collect();

    let export = json!({
        "format": "agent-profile-export",
        "version": 1,
        "exported_at": now(),
        "profile": {
            "username": profile.username,
            "display_name": profile.display_name,
            "tagline": profile.tagline,
            "bio": profile.bio,
            "third_line": profile.third_line,
            "avatar_url": profile.avatar_url,
            "theme": profile.theme,
            "particle_effect": profile.particle_effect,
            "particle_enabled": profile.particle_enabled,
            "particle_seasonal": profile.particle_seasonal,
            "pubkey": profile.pubkey,
            "created_at": profile.created_at,
        },
        "links": links,
        "sections": sections,
        "skills": skills,
        "crypto_addresses": addresses,
    });

    Ok(Json(export))
}

/// POST /api/v1/import
/// Create or update a profile from an export document.
/// If the username doesn't exist, creates it (returns new API key).
/// If the username exists and valid API key provided, updates it.
#[post("/import", data = "<body>")]
pub fn import_profile(
    db: &State<DbConn>,
    body: Json<serde_json::Value>,
    api_key_header: Option<ApiKey>,
    _rl: crate::ratelimit::RegisterRateLimit,
) -> Result<Json<serde_json::Value>, (Status, Json<serde_json::Value>)> {
    // Validate format
    let format = body.get("format").and_then(|v| v.as_str()).unwrap_or("");
    if format != "agent-profile-export" {
        return Err((Status::BadRequest, Json(json!({"error": "Invalid format. Expected 'agent-profile-export'."}))));
    }

    let profile_data = body.get("profile").ok_or_else(|| {
        (Status::BadRequest, Json(json!({"error": "Missing 'profile' object in export document."})))
    })?;

    let username = profile_data.get("username").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
    if username.len() < 3 || username.len() > 50 {
        return Err((Status::BadRequest, Json(json!({"error": "Username must be 3-50 characters."}))));
    }
    if !username.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err((Status::BadRequest, Json(json!({"error": "Username must contain only lowercase letters, digits, and hyphens."}))));
    }
    if username.starts_with('-') || username.ends_with('-') {
        return Err((Status::BadRequest, Json(json!({"error": "Username cannot start or end with a hyphen."}))));
    }

    let conn = db.lock().unwrap();

    // Check if profile already exists
    let existing: Option<String> = conn.query_row(
        "SELECT api_key_hash FROM profiles WHERE username = ?1",
        params![username],
        |row| row.get(0),
    ).ok();

    let (api_key_out, is_new) = if let Some(stored_hash) = existing {
        // Profile exists — require valid API key to update
        let key = api_key_header.as_ref().map(|k| k.0.as_str()).unwrap_or("");
        if key.is_empty() || hash_key(key) != stored_hash {
            return Err((Status::Unauthorized, Json(json!({
                "error": "Profile already exists. Provide valid X-API-Key to update via import."
            }))));
        }
        (key.to_string(), false)
    } else {
        // New profile — create it
        let new_key = gen_api_key();
        let key_hash = hash_key(&new_key);
        let id = Uuid::new_v4().to_string();
        let ts = now();

        conn.execute(
            "INSERT INTO profiles (id, username, display_name, tagline, bio, third_line, avatar_url, \
             theme, particle_effect, particle_enabled, particle_seasonal, pubkey, \
             api_key_hash, profile_score, view_count, created_at, updated_at) \
             VALUES (?1, ?2, '', '', '', '', '', 'dark', 'none', 0, 0, '', ?3, 0, 0, ?4, ?4)",
            params![id, username, key_hash, ts],
        ).map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

        (new_key, true)
    };

    // Apply profile fields
    let display_name = profile_data.get("display_name").and_then(|v| v.as_str()).unwrap_or("");
    let tagline = profile_data.get("tagline").and_then(|v| v.as_str()).unwrap_or("");
    let bio = profile_data.get("bio").and_then(|v| v.as_str()).unwrap_or("");
    let third_line = profile_data.get("third_line").and_then(|v| v.as_str()).unwrap_or("");
    let theme = profile_data.get("theme").and_then(|v| v.as_str()).unwrap_or("dark");
    let particle_effect = profile_data.get("particle_effect").and_then(|v| v.as_str()).unwrap_or("none");
    let particle_enabled = profile_data.get("particle_enabled").and_then(|v| v.as_bool()).unwrap_or(false) as i32;
    let particle_seasonal = profile_data.get("particle_seasonal").and_then(|v| v.as_bool()).unwrap_or(false) as i32;
    let pubkey = profile_data.get("pubkey").and_then(|v| v.as_str()).unwrap_or("");
    let ts = now();

    conn.execute(
        "UPDATE profiles SET display_name=?1, tagline=?2, bio=?3, third_line=?4, \
         theme=?5, particle_effect=?6, particle_enabled=?7, particle_seasonal=?8, \
         pubkey=?9, updated_at=?10 WHERE username=?11",
        params![display_name, tagline, bio, third_line, theme, particle_effect,
                particle_enabled, particle_seasonal, pubkey, ts, username],
    ).map_err(|e| (Status::InternalServerError, Json(json!({"error": e.to_string()}))))?;

    // Import links
    if let Some(links) = body.get("links").and_then(|v| v.as_array()) {
        let profile_id: String = conn.query_row(
            "SELECT id FROM profiles WHERE username = ?1", params![username], |row| row.get(0),
        ).unwrap();

        // Clear existing links on import (replace semantics)
        if !is_new {
            let _ = conn.execute("DELETE FROM profile_links WHERE profile_id = ?1", params![profile_id]);
        }

        for link in links {
            let url = link.get("url").and_then(|v| v.as_str()).unwrap_or("");
            let label = link.get("label").and_then(|v| v.as_str()).unwrap_or("");
            let platform = link.get("platform").and_then(|v| v.as_str()).unwrap_or("website");
            if !url.is_empty() {
                let _ = conn.execute(
                    "INSERT INTO profile_links (id, profile_id, url, label, platform, display_order, created_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6)",
                    params![Uuid::new_v4().to_string(), profile_id, url, label, platform, ts],
                );
            }
        }

        // Import sections
        if let Some(sections) = body.get("sections").and_then(|v| v.as_array()) {
            if !is_new {
                let _ = conn.execute("DELETE FROM profile_sections WHERE profile_id = ?1", params![profile_id]);
            }
            for (i, section) in sections.iter().enumerate() {
                let title = section.get("title").and_then(|v| v.as_str()).unwrap_or("");
                let content = section.get("content").and_then(|v| v.as_str()).unwrap_or("");
                let section_type = section.get("section_type").and_then(|v| v.as_str()).unwrap_or("custom");
                if !title.is_empty() {
                    let _ = conn.execute(
                        "INSERT INTO profile_sections (id, profile_id, section_type, title, content, display_order, created_at) \
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                        params![Uuid::new_v4().to_string(), profile_id, section_type, title, content, i as i32, ts],
                    );
                }
            }
        }

        // Import skills
        if let Some(skills) = body.get("skills").and_then(|v| v.as_array()) {
            if !is_new {
                let _ = conn.execute("DELETE FROM profile_skills WHERE profile_id = ?1", params![profile_id]);
            }
            for skill_val in skills {
                let skill = skill_val.as_str().unwrap_or("");
                if !skill.is_empty() {
                    let _ = conn.execute(
                        "INSERT OR IGNORE INTO profile_skills (id, profile_id, skill, created_at) \
                         VALUES (?1, ?2, ?3, ?4)",
                        params![Uuid::new_v4().to_string(), profile_id, skill, ts],
                    );
                }
            }
        }

        // Import crypto addresses
        if let Some(addresses) = body.get("crypto_addresses").and_then(|v| v.as_array()) {
            if !is_new {
                let _ = conn.execute("DELETE FROM crypto_addresses WHERE profile_id = ?1", params![profile_id]);
            }
            for addr in addresses {
                let network = addr.get("network").and_then(|v| v.as_str()).unwrap_or("");
                let address = addr.get("address").and_then(|v| v.as_str()).unwrap_or("");
                let label = addr.get("label").and_then(|v| v.as_str()).unwrap_or("");
                if !network.is_empty() && !address.is_empty() {
                    let _ = conn.execute(
                        "INSERT INTO crypto_addresses (id, profile_id, network, address, label, created_at) \
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        params![Uuid::new_v4().to_string(), profile_id, network, address, label, ts],
                    );
                }
            }
        }
    }

    // Recompute score
    let profile = load_profile(&conn, &username).unwrap();
    let new_score = compute_profile_score(
        &profile.display_name, &profile.tagline, &profile.bio,
        &profile.avatar_url, &profile.pubkey,
        !profile.crypto_addresses.is_empty(),
        !profile.links.is_empty(),
        !profile.sections.is_empty(),
        !profile.skills.is_empty(),
    );
    let _ = conn.execute(
        "UPDATE profiles SET profile_score = ?1 WHERE username = ?2",
        params![new_score, username],
    );

    let mut result = json!({
        "status": if is_new { "created" } else { "updated" },
        "username": username,
        "profile_url": format!("/{}", username),
        "json_url": format!("/api/v1/profiles/{}", username),
    });

    if is_new {
        result["api_key"] = json!(api_key_out);
    }

    Ok(Json(result))
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

// --- Base URL request guard ---
/// Resolves the canonical base URL for the service.
/// Priority: BASE_URL env var → Host header (with X-Forwarded-Proto) → empty string.
pub struct BaseUrl(pub String);

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for BaseUrl {
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        // 1. Prefer explicitly configured BASE_URL
        if let Ok(base) = std::env::var("BASE_URL") {
            let base = base.trim_end_matches('/').to_string();
            if !base.is_empty() {
                return rocket::request::Outcome::Success(BaseUrl(base));
            }
        }
        // 2. Fall back to Host header + inferred scheme
        if let Some(host) = request.headers().get_one("Host") {
            let scheme = request.headers().get_one("X-Forwarded-Proto")
                .unwrap_or("http");
            let base = format!("{}://{}", scheme, host.trim_end_matches('/'));
            return rocket::request::Outcome::Success(BaseUrl(base));
        }
        // 3. Last resort: empty (relative URLs only)
        rocket::request::Outcome::Success(BaseUrl(String::new()))
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
    _rl: crate::ratelimit::ChallengeRateLimit,
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
    _rl: crate::ratelimit::VerifyRateLimit,
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

#[get("/SKILL.md")]
pub fn skill_md() -> (ContentType, &'static str) {
    (ContentType::Plain, include_str!("../../SKILL.md"))
}

#[get("/llms.txt")]
pub fn llms_txt() -> (ContentType, &'static str) {
    (ContentType::Plain, include_str!("../../SKILL.md"))
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
            },
            {
                "id": "endorse-agent",
                "name": "Endorse another agent",
                "endpoint": "POST /api/v1/profiles/{username}/endorsements",
                "description": "Leave a signed attestation vouching for another agent's profile or capabilities"
            },
            {
                "id": "search-profiles",
                "name": "Search agent profiles",
                "endpoint": "GET /api/v1/profiles",
                "description": "Discover agents by skill tag (?skill=), text (?q=), cryptographic identity (?has_pubkey=true), sorted by popularity (?sort=popular), newest (?sort=newest), or activity (?sort=active)"
            },
            {
                "id": "skill-directory",
                "name": "Browse skill directory",
                "endpoint": "GET /api/v1/skills",
                "description": "List all skill tags in the ecosystem sorted by usage count; optionally filter by substring"
            },
            {
                "id": "service-stats",
                "name": "Get service statistics",
                "endpoint": "GET /api/v1/stats",
                "description": "Aggregate counts: profiles, skills, endorsements, links, and top skills"
            },
            {
                "id": "webfinger-lookup",
                "name": "WebFinger identity lookup",
                "endpoint": "GET /.well-known/webfinger?resource=acct:{username}@{host}",
                "description": "RFC 7033 identity discovery. Returns application/jrd+json with links to profile page and JSON API. Enables @username@host addressing used by Mastodon, ActivityPub, and Keyoxide."
            }
        ]
    }).to_string())
}

// --- Web discovery ---

/// GET /robots.txt
/// Standard robots.txt — allows all crawlers and points to sitemap.
#[get("/robots.txt")]
pub fn robots_txt(base_url: BaseUrl) -> (ContentType, String) {
    let sitemap_url = if base_url.0.is_empty() {
        "/sitemap.xml".to_string()
    } else {
        format!("{}/sitemap.xml", base_url.0)
    };
    (ContentType::Plain, format!(
        "User-agent: *\nAllow: /\nSitemap: {}\n",
        sitemap_url
    ))
}

/// GET /sitemap.xml
/// Dynamic XML sitemap listing all public agent profile pages.
/// Respects BASE_URL environment variable for absolute URLs.
#[get("/sitemap.xml")]
pub fn sitemap_xml(db: &State<DbConn>, base_url: BaseUrl) -> (ContentType, String) {
    let conn = db.lock().unwrap();
    let base = base_url.0.as_str();

    let usernames: Vec<String> = {
        let mut stmt = conn.prepare(
            "SELECT username FROM profiles ORDER BY profile_score DESC, created_at ASC"
        ).unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    };

    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
"#,
    );

    // Service discovery pages
    for path in &["/", "/SKILL.md", "/llms.txt", "/openapi.json"] {
        xml.push_str(&format!("  <url><loc>{}{}</loc></url>\n", base, path));
    }

    // One entry per agent profile
    for username in &usernames {
        xml.push_str(&format!("  <url><loc>{}/{}</loc></url>\n", base, username));
    }

    xml.push_str("</urlset>\n");

    (ContentType::new("text", "xml"), xml)
}

/// GET /feed.xml
/// Atom feed (RFC 4287) of recently created/updated agent profiles.
/// Returns the 20 most recently active profiles.
#[get("/feed.xml")]
pub fn feed_xml(db: &State<DbConn>, base_url: BaseUrl) -> (ContentType, String) {
    let conn = db.lock().unwrap();
    let base = base_url.0.as_str();

    let profiles: Vec<(String, String, String, String, String)> = {
        let mut stmt = conn.prepare(
            "SELECT username, display_name, tagline, created_at, updated_at \
             FROM profiles ORDER BY updated_at DESC LIMIT 20"
        ).unwrap();
        stmt.query_map([], |row| Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
        )))
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    };

    // Feed updated timestamp = most recent profile update
    let feed_updated = profiles.first()
        .map(|p| p.4.clone())
        .unwrap_or_else(|| now());

    let mut xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Agent Profiles</title>
  <subtitle>Recently active AI agent profiles</subtitle>
  <link href="{base}/feed.xml" rel="self" type="application/atom+xml"/>
  <link href="{base}/" rel="alternate" type="text/html"/>
  <id>{base}/feed.xml</id>
  <updated>{feed_updated}</updated>
  <generator>agent-profile v{version}</generator>
"#,
        version = env!("CARGO_PKG_VERSION"),
    );

    for (username, display_name, tagline, created_at, updated_at) in &profiles {
        let title = if display_name.is_empty() { username } else { display_name };
        let summary = if tagline.is_empty() {
            format!("Agent profile: {}", username)
        } else {
            xml_escape(tagline)
        };

        xml.push_str(&format!(
            r#"  <entry>
    <title>{title}</title>
    <link href="{base}/{username}" rel="alternate" type="text/html"/>
    <link href="{base}/api/v1/profiles/{username}" rel="alternate" type="application/json"/>
    <id>{base}/{username}</id>
    <updated>{updated_at}</updated>
    <published>{created_at}</published>
    <summary>{summary}</summary>
    <author><name>{title}</name></author>
  </entry>
"#,
            title = xml_escape(title),
        ));
    }

    xml.push_str("</feed>\n");
    (ContentType::new("application", "atom+xml"), xml)
}

/// Escape XML special characters for safe inclusion in Atom feed.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}

// --- WebFinger (RFC 7033) ---

/// GET /.well-known/webfinger?resource=acct:{username}@{host}
/// Standard identity discovery — makes agent profiles reachable via @username@host addressing.
/// Used by Mastodon, ActivityPub, Keyoxide, and other decentralized identity systems.
/// Returns application/jrd+json with links to the profile page and JSON API endpoint.
#[get("/.well-known/webfinger?<resource>")]
pub fn webfinger(
    db: &State<DbConn>,
    resource: Option<String>,
    base_url: BaseUrl,
) -> Result<(ContentType, String), (Status, Json<serde_json::Value>)> {
    // Validate resource param
    let resource = resource.ok_or_else(|| {
        (Status::BadRequest, Json(json!({"error": "Missing required query parameter: resource"})))
    })?;

    // Parse acct:username@host format
    let acct = resource.strip_prefix("acct:").ok_or_else(|| {
        (Status::BadRequest, Json(json!({"error": "resource must use acct: URI scheme (e.g. acct:username@host)"})))
    })?;

    let username = acct.split('@').next().unwrap_or("").to_lowercase();
    if username.is_empty() {
        return Err((Status::BadRequest, Json(json!({"error": "Could not parse username from resource"}))));
    }

    // Look up profile
    let conn = db.lock().unwrap();
    let exists = conn.query_row(
        "SELECT 1 FROM profiles WHERE username = ?1",
        params![username],
        |_| Ok(()),
    ).is_ok();

    if !exists {
        return Err((Status::NotFound, Json(json!({"error": format!("No profile for '{}'", username)}))));
    }

    let base = base_url.0;

    let subject = resource.clone();
    let profile_page = format!("{}/{}", base, username);
    let json_url = format!("{}/api/v1/profiles/{}", base, username);
    let avatar_url = format!("{}/avatars/{}", base, username);

    let jrd = json!({
        "subject": subject,
        "aliases": [profile_page, json_url],
        "links": [
            {
                "rel": "http://webfinger.net/rel/profile-page",
                "type": "text/html",
                "href": profile_page
            },
            {
                "rel": "self",
                "type": "application/json",
                "href": json_url
            },
            {
                "rel": "http://webfinger.net/rel/avatar",
                "href": avatar_url
            }
        ]
    });

    // JRD content type: application/jrd+json (RFC 7033 §10.2)
    Ok((ContentType::new("application", "jrd+json"), jrd.to_string()))
}

// --- Endorsements ---

/// POST /api/v1/profiles/{username}/endorsements
/// Add an endorsement from the calling agent (identified by their API key) to the target profile.
/// - `from`: the endorser's username (must match API key owner)
/// - `message`: short endorsement text (1–500 chars)
/// - `signature`: optional secp256k1 signature over the message hex (for cryptographic attestation)
#[post("/profiles/<username>/endorsements", data = "<body>")]
pub fn add_endorsement(
    db: &State<DbConn>,
    username: &str,
    body: Json<AddEndorsementRequest>,
    api_key: ApiKey,
) -> Result<Json<serde_json::Value>, (Status, Json<serde_json::Value>)> {
    let endorsee_username = username.to_lowercase();
    let endorser_username = body.from.to_lowercase();
    let conn = db.lock().unwrap();

    // Validate message length
    let message = body.message.trim().to_string();
    if message.is_empty() {
        return Err((Status::UnprocessableEntity, Json(json!({"error": "message cannot be empty"}))));
    }
    if message.len() > 500 {
        return Err((Status::UnprocessableEntity, Json(json!({"error": "message max 500 chars"}))));
    }

    // Validate: API key belongs to the endorser
    if !verify_api_key(&conn, &endorser_username, &api_key.0) {
        return Err((Status::Unauthorized, Json(json!({
            "error": "Invalid API key. The API key must belong to the 'from' username (the endorser)."
        }))));
    }

    // Validate: not self-endorsing
    if endorser_username == endorsee_username {
        return Err((Status::UnprocessableEntity, Json(json!({"error": "Cannot endorse your own profile"}))));
    }

    // Validate: endorsee profile exists, get their id
    let endorsee_result = conn.query_row(
        "SELECT id FROM profiles WHERE username = ?1",
        params![endorsee_username],
        |row| row.get::<_, String>(0),
    );
    let endorsee_id = match endorsee_result {
        Ok(id) => id,
        Err(_) => return Err((Status::NotFound, Json(json!({
            "error": format!("Profile '{}' not found", endorsee_username)
        })))),
    };

    // Optional: verify secp256k1 signature over the message if provided
    let sig_str = body.signature.clone().unwrap_or_default();
    let mut verified = false;
    if !sig_str.is_empty() {
        // Look up the endorser's pubkey
        let endorser_pubkey: String = conn.query_row(
            "SELECT pubkey FROM profiles WHERE username = ?1",
            params![endorser_username],
            |row| row.get(0),
        ).unwrap_or_default();

        if endorser_pubkey.is_empty() {
            return Err((Status::UnprocessableEntity, Json(json!({
                "error": "Endorser has no public key set. Add a secp256k1 pubkey to your profile before signing endorsements."
            }))));
        }
        verified = verify_ecdsa_signature(&endorser_pubkey, &message, &sig_str);
        if !verified {
            return Err((Status::UnprocessableEntity, Json(json!({
                "error": "Signature verification failed. Sign the exact message text with your secp256k1 private key."
            }))));
        }
    }

    // Insert endorsement (UNIQUE(endorsee_id, endorser_username) prevents duplicates)
    let id = Uuid::new_v4().to_string();
    let ts = now();
    let insert_result = conn.execute(
        "INSERT INTO endorsements (id, endorsee_id, endorser_username, message, signature, verified, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, endorsee_id, endorser_username, message, sig_str, verified as i32, ts],
    );

    match insert_result {
        Ok(_) => Ok(Json(json!({
            "id": id,
            "endorsee": endorsee_username,
            "endorser": endorser_username,
            "message": message,
            "verified": verified,
            "created_at": ts,
        }))),
        Err(e) if e.to_string().contains("UNIQUE") => {
            // Already endorsed — update the existing endorsement instead
            conn.execute(
                "UPDATE endorsements SET message = ?1, signature = ?2, verified = ?3 \
                 WHERE endorsee_id = ?4 AND endorser_username = ?5",
                params![message, sig_str, verified as i32, endorsee_id, endorser_username],
            ).ok();
            Ok(Json(json!({
                "id": id,
                "endorsee": endorsee_username,
                "endorser": endorser_username,
                "message": message,
                "verified": verified,
                "updated": true,
                "created_at": ts,
            })))
        }
        Err(e) => Err((Status::InternalServerError, Json(json!({"error": e.to_string()})))),
    }
}

/// GET /api/v1/profiles/{username}/endorsements
/// List all endorsements received by a profile (public, no auth).
#[get("/profiles/<username>/endorsements")]
pub fn get_endorsements(
    db: &State<DbConn>,
    username: &str,
) -> Result<Json<serde_json::Value>, (Status, Json<serde_json::Value>)> {
    let username = username.to_lowercase();
    let conn = db.lock().unwrap();

    let profile_id_result = conn.query_row(
        "SELECT id FROM profiles WHERE username = ?1",
        params![username],
        |row| row.get::<_, String>(0),
    );
    let profile_id = match profile_id_result {
        Ok(id) => id,
        Err(_) => return Err((Status::NotFound, Json(json!({
            "error": format!("Profile '{}' not found", username)
        })))),
    };

    let endorsements = load_endorsements(&conn, &profile_id);
    let verified_count = endorsements.iter().filter(|e| e.verified).count();

    Ok(Json(json!({
        "username": username,
        "endorsements": endorsements,
        "total": endorsements.len(),
        "verified_count": verified_count,
    })))
}

/// DELETE /api/v1/profiles/{username}/endorsements/{endorser_username}
/// Remove an endorsement. Can be done by either the endorser OR the endorsee.
#[delete("/profiles/<username>/endorsements/<endorser>")]
pub fn delete_endorsement(
    db: &State<DbConn>,
    username: &str,
    endorser: &str,
    api_key: ApiKey,
) -> Result<Json<serde_json::Value>, (Status, Json<serde_json::Value>)> {
    let endorsee_username = username.to_lowercase();
    let endorser_username = endorser.to_lowercase();
    let conn = db.lock().unwrap();

    // Auth: must be either the endorser or the endorsee
    let is_endorser = verify_api_key(&conn, &endorser_username, &api_key.0);
    let is_endorsee = verify_api_key(&conn, &endorsee_username, &api_key.0);

    if !is_endorser && !is_endorsee {
        return Err((Status::Unauthorized, Json(json!({
            "error": "API key must belong to either the endorser or the endorsee profile"
        }))));
    }

    // Get endorsee profile id
    let endorsee_id_result = conn.query_row(
        "SELECT id FROM profiles WHERE username = ?1",
        params![endorsee_username],
        |row| row.get::<_, String>(0),
    );
    let endorsee_id = match endorsee_id_result {
        Ok(id) => id,
        Err(_) => return Err((Status::NotFound, Json(json!({
            "error": format!("Profile '{}' not found", endorsee_username)
        })))),
    };

    let deleted = conn.execute(
        "DELETE FROM endorsements WHERE endorsee_id = ?1 AND endorser_username = ?2",
        params![endorsee_id, endorser_username],
    ).unwrap_or(0);

    if deleted == 0 {
        return Err((Status::NotFound, Json(json!({
            "error": format!("No endorsement from '{}' on '{}'", endorser_username, endorsee_username)
        }))));
    }

    Ok(Json(json!({"deleted": true, "endorser": endorser_username, "endorsee": endorsee_username})))
}
