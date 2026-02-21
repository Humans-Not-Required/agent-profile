use rusqlite::{Connection, Result};

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;

        CREATE TABLE IF NOT EXISTS profiles (
            id TEXT PRIMARY KEY,
            slug TEXT UNIQUE NOT NULL,
            display_name TEXT NOT NULL,
            bio TEXT NOT NULL DEFAULT '',
            avatar_url TEXT NOT NULL DEFAULT '',
            manage_token TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_profiles_slug ON profiles(slug);

        CREATE TABLE IF NOT EXISTS crypto_addresses (
            id TEXT PRIMARY KEY,
            profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
            network TEXT NOT NULL,
            address TEXT NOT NULL,
            label TEXT NOT NULL DEFAULT '',
            verified INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_addresses_profile ON crypto_addresses(profile_id);

        CREATE TABLE IF NOT EXISTS profile_links (
            id TEXT PRIMARY KEY,
            profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
            link_type TEXT NOT NULL,
            label TEXT NOT NULL,
            value TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_links_profile ON profile_links(profile_id);

        CREATE TABLE IF NOT EXISTS profile_skills (
            id TEXT PRIMARY KEY,
            profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
            skill TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_skills_profile ON profile_skills(profile_id);
    ")?;
    Ok(())
}

pub fn get_db_path() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| "agent-profile.db".to_string())
}
