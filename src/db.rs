use rusqlite::{Connection, Result};

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;

        CREATE TABLE IF NOT EXISTS profiles (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            display_name TEXT NOT NULL DEFAULT '',
            tagline TEXT NOT NULL DEFAULT '',
            bio TEXT NOT NULL DEFAULT '',
            third_line TEXT NOT NULL DEFAULT '',
            avatar_url TEXT NOT NULL DEFAULT '',
            avatar_data BLOB,
            avatar_mime TEXT NOT NULL DEFAULT '',
            theme TEXT NOT NULL DEFAULT 'dark',
            particle_effect TEXT NOT NULL DEFAULT 'none',
            particle_enabled INTEGER NOT NULL DEFAULT 0,
            particle_seasonal INTEGER NOT NULL DEFAULT 0,
            pubkey TEXT NOT NULL DEFAULT '',
            api_key_hash TEXT NOT NULL,
            profile_score INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_profiles_username ON profiles(username);

        CREATE TABLE IF NOT EXISTS crypto_addresses (
            id TEXT PRIMARY KEY,
            profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
            network TEXT NOT NULL,
            address TEXT NOT NULL,
            label TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_addresses_profile ON crypto_addresses(profile_id);

        CREATE TABLE IF NOT EXISTS profile_links (
            id TEXT PRIMARY KEY,
            profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
            url TEXT NOT NULL,
            label TEXT NOT NULL,
            platform TEXT NOT NULL DEFAULT 'website',
            display_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_links_profile ON profile_links(profile_id);

        CREATE TABLE IF NOT EXISTS profile_sections (
            id TEXT PRIMARY KEY,
            profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
            section_type TEXT NOT NULL DEFAULT 'custom',
            title TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL DEFAULT '',
            display_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_sections_profile ON profile_sections(profile_id);

        CREATE TABLE IF NOT EXISTS profile_skills (
            id TEXT PRIMARY KEY,
            profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
            skill TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_skills_profile ON profile_skills(profile_id);

        CREATE TABLE IF NOT EXISTS identity_challenges (
            id TEXT PRIMARY KEY,
            profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
            challenge TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            used INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS endorsements (
            id TEXT PRIMARY KEY,
            endorsee_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
            endorser_username TEXT NOT NULL,
            message TEXT NOT NULL,
            signature TEXT NOT NULL DEFAULT '',
            verified INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            UNIQUE(endorsee_id, endorser_username)
        );

        CREATE INDEX IF NOT EXISTS idx_endorsements_endorsee ON endorsements(endorsee_id);
    ")?;

    // ── Migrations ──────────────────────────────────────────────────────────
    // Add view_count column (v0.6.0) — safe to run on existing DBs
    if conn.prepare("SELECT view_count FROM profiles LIMIT 0").is_err() {
        conn.execute("ALTER TABLE profiles ADD COLUMN view_count INTEGER NOT NULL DEFAULT 0", [])?;
    }

    Ok(())
}

pub fn get_db_path() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| "agent-profile.db".to_string())
}
