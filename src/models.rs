use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;
use regex::Regex;

static USERNAME_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-z0-9][a-z0-9-]{1,28}[a-z0-9]$|^[a-z0-9]{1,30}$").unwrap()
});

const RESERVED_USERNAMES: &[&str] = &[
    "api", "admin", "static", "health", "agents", "profiles",
    "register", "avatars", "openapi", "llms", "well-known",
];

pub const VALID_THEMES: &[&str] = &[
    "dark", "light", "midnight", "forest", "ocean", "desert", "aurora",
    "cream", "sky", "lavender", "sage", "peach",
    "terminator", "matrix", "replicant", "br2049", "br2049-sandstorm",
    "snow", "christmas", "halloween", "spring", "summer", "autumn",
    "newyear", "valentine", "boba", "fruitsalad", "junkfood",
    "candy", "coffee", "lava",
];
pub const VALID_PARTICLE_EFFECTS: &[&str] = &["none", "snow", "leaves", "rain", "fireflies", "stars", "sakura", "embers", "digital-rain", "flames", "water", "boba", "clouds", "fruit", "junkfood", "warzone", "hearts", "cactus", "candy", "coffee", "wasteland", "fireworks", "forest", "sandstorm", "lava"];
pub const VALID_NETWORKS: &[&str] = &[
    "bitcoin", "lightning", "ethereum", "cardano", "ergo",
    "nervos", "solana", "monero", "dogecoin", "nostr", "custom",
];
pub const VALID_PLATFORMS: &[&str] = &[
    "github", "twitter", "moltbook", "nostr", "telegram",
    "discord", "youtube", "linkedin", "website", "email", "custom",
];
pub const VALID_SECTION_TYPES: &[&str] = &[
    "about", "interests", "projects", "skills", "values", "fun_facts",
    "currently_working_on", "currently_learning", "looking_for", "open_to", "custom",
];

/// Maximum number of sub-resources per profile.
pub const MAX_LINKS: usize = 20;
pub const MAX_SECTIONS: usize = 20;
pub const MAX_SKILLS: usize = 50;
pub const MAX_ADDRESSES: usize = 10;
pub const MAX_ENDORSEMENTS: usize = 100;

/// Maximum field lengths for fields without inline limits.
pub const MAX_DISPLAY_NAME: usize = 100;
pub const MAX_THIRD_LINE: usize = 200;
pub const MAX_AVATAR_URL: usize = 2000;
pub const MAX_LINK_URL: usize = 2000;
pub const MAX_LINK_LABEL: usize = 100;
pub const MAX_SKILL_NAME: usize = 50;
pub const MAX_ADDRESS_VALUE: usize = 500;
pub const MAX_ADDRESS_LABEL: usize = 100;
pub const MAX_SECTION_TITLE: usize = 200;

// --- Sub-resource types ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CryptoAddress {
    pub id: String,
    pub profile_id: String,
    pub network: String,
    pub address: String,
    pub label: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileLink {
    pub id: String,
    pub profile_id: String,
    pub url: String,
    pub label: String,
    pub platform: String,
    pub display_order: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileSection {
    pub id: String,
    pub profile_id: String,
    pub section_type: String,
    pub title: String,
    pub content: String,
    pub display_order: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileSkill {
    pub id: String,
    pub profile_id: String,
    pub skill: String,
    pub created_at: String,
}

// --- Endorsements ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Endorsement {
    pub id: String,
    pub endorsee_id: String,
    pub endorser_username: String,
    pub message: String,
    pub signature: String,
    pub verified: bool,
    pub created_at: String,
}

// --- Main profile type ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub tagline: String,
    pub bio: String,
    pub third_line: String,
    pub avatar_url: String,
    pub theme: String,
    pub particle_effect: String,
    pub particle_enabled: bool,
    pub particle_seasonal: bool,
    pub pubkey: String,
    pub profile_score: i64,
    pub view_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub crypto_addresses: Vec<CryptoAddress>,
    pub links: Vec<ProfileLink>,
    pub sections: Vec<ProfileSection>,
    pub skills: Vec<ProfileSkill>,
    pub endorsements: Vec<Endorsement>,
}

// --- Request types ---

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub display_name: Option<String>,
    pub pubkey: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub tagline: Option<String>,
    pub bio: Option<String>,
    pub third_line: Option<String>,
    pub avatar_url: Option<String>,
    pub theme: Option<String>,
    pub particle_effect: Option<String>,
    pub particle_enabled: Option<bool>,
    pub particle_seasonal: Option<bool>,
    pub pubkey: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddAddressRequest {
    pub network: String,
    pub address: String,
    pub label: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAddressRequest {
    pub network: Option<String>,
    pub address: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddLinkRequest {
    pub url: String,
    pub label: String,
    pub platform: Option<String>,
    pub display_order: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLinkRequest {
    pub url: Option<String>,
    pub label: Option<String>,
    pub platform: Option<String>,
    pub display_order: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct AddSectionRequest {
    pub section_type: Option<String>,
    pub title: String,
    pub content: String,
    pub display_order: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSectionRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub display_order: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct AddSkillRequest {
    pub skill: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifySignatureRequest {
    pub signature: String,
}

#[derive(Debug, Deserialize)]
pub struct AddEndorsementRequest {
    /// Username of the endorsing agent (must match the API key used)
    pub from: String,
    /// Short endorsement message (1–500 chars)
    pub message: String,
    /// Optional secp256k1 signature over the message (hex-encoded)
    pub signature: Option<String>,
}

// --- Response types ---

#[derive(Serialize)]
pub struct RegisterResponse {
    pub username: String,
    pub api_key: String,
    pub profile_url: String,
    pub json_url: String,
}

#[derive(Serialize)]
pub struct ReissueKeyResponse {
    pub username: String,
    pub api_key: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct ChallengeResponse {
    pub challenge: String,
    pub expires_in_seconds: u64,
    pub instructions: String,
}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub verified: bool,
    pub username: String,
    pub timestamp: String,
}

#[derive(Serialize)]
pub struct ProfileScoreResponse {
    pub score: i64,
    pub max_score: i64,
    pub breakdown: Vec<ScoreItem>,
    pub next_steps: Vec<String>,
}

#[derive(Serialize)]
pub struct ScoreItem {
    pub field: String,
    pub label: String,
    pub points: i64,
    pub earned: bool,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub service: String,
}

// --- Validation ---

pub fn validate_username(username: &str) -> Result<String, String> {
    let normalized = username.to_lowercase();
    if normalized.len() < 3 || normalized.len() > 30 {
        return Err("Username must be 3–30 characters".to_string());
    }
    if !USERNAME_REGEX.is_match(&normalized) {
        return Err("Username must contain only letters, numbers, and hyphens; cannot start/end with hyphen".to_string());
    }
    if RESERVED_USERNAMES.contains(&normalized.as_str()) {
        return Err(format!("'{}' is a reserved username", normalized));
    }
    Ok(normalized)
}

pub fn validate_pubkey(pubkey: &str) -> bool {
    // Accept 33-byte compressed (66 hex chars) or 65-byte uncompressed (130 hex chars)
    let hex_only = pubkey.chars().all(|c| c.is_ascii_hexdigit());
    hex_only && (pubkey.len() == 66 || pubkey.len() == 130)
}

/// Input fields for profile score computation.
pub struct ScoreInput<'a> {
    pub display_name: &'a str,
    pub tagline: &'a str,
    pub bio: &'a str,
    pub avatar_url: &'a str,
    pub pubkey: &'a str,
    pub has_address: bool,
    pub has_link: bool,
    pub has_section: bool,
    pub has_skill: bool,
}

/// Compute profile completeness score 0-100.
pub fn compute_profile_score(input: &ScoreInput) -> i64 {
    let mut score: i64 = 0;
    // Base fields
    if !input.display_name.is_empty() { score += 10; }
    if !input.tagline.is_empty() { score += 10; }
    if input.bio.len() >= 20 { score += 20; }  // At least a meaningful bio
    if !input.avatar_url.is_empty() { score += 10; }
    // Identity
    if !input.pubkey.is_empty() { score += 20; }  // Big bonus for cryptographic identity
    // Sub-resources
    if input.has_address { score += 10; }
    if input.has_link { score += 5; }
    if input.has_section { score += 10; }
    if input.has_skill { score += 5; }
    score.clamp(0, 100)
}

pub fn score_breakdown(input: &ScoreInput) -> Vec<ScoreItem> {
    vec![
        ScoreItem { field: "display_name".to_string(), label: "Display name set".to_string(), points: 10, earned: !input.display_name.is_empty() },
        ScoreItem { field: "tagline".to_string(), label: "Tagline set".to_string(), points: 10, earned: !input.tagline.is_empty() },
        ScoreItem { field: "bio".to_string(), label: "Bio (20+ chars)".to_string(), points: 20, earned: input.bio.len() >= 20 },
        ScoreItem { field: "avatar".to_string(), label: "Avatar set".to_string(), points: 10, earned: !input.avatar_url.is_empty() },
        ScoreItem { field: "pubkey".to_string(), label: "secp256k1 public key (cryptographic identity)".to_string(), points: 20, earned: !input.pubkey.is_empty() },
        ScoreItem { field: "crypto_address".to_string(), label: "At least one crypto address".to_string(), points: 10, earned: input.has_address },
        ScoreItem { field: "link".to_string(), label: "At least one link".to_string(), points: 5, earned: input.has_link },
        ScoreItem { field: "section".to_string(), label: "At least one freeform section".to_string(), points: 10, earned: input.has_section },
        ScoreItem { field: "skill".to_string(), label: "At least one skill tag".to_string(), points: 5, earned: input.has_skill },
    ]
}

pub fn score_next_steps(input: &ScoreInput) -> Vec<String> {
    let mut steps = vec![];
    if input.display_name.is_empty() { steps.push("Set a display name".to_string()); }
    if input.tagline.is_empty() { steps.push("Add a tagline (short subtitle)".to_string()); }
    if input.bio.len() < 20 { steps.push("Write a bio (at least 20 characters)".to_string()); }
    if input.avatar_url.is_empty() { steps.push("Add an avatar URL or upload an image".to_string()); }
    if input.pubkey.is_empty() { steps.push("Add a secp256k1 public key for cryptographic identity (+20 points)".to_string()); }
    if !input.has_address { steps.push("Add a crypto address (Bitcoin, Lightning, Nostr, etc.)".to_string()); }
    if !input.has_link { steps.push("Add a link (GitHub, website, etc.)".to_string()); }
    if !input.has_section { steps.push("Add a profile section (bio, interests, projects, etc.)".to_string()); }
    if !input.has_skill { steps.push("Add at least one skill tag".to_string()); }
    steps
}

// --- Unit tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_usernames() {
        assert!(validate_username("nanook42").is_ok());
        assert!(validate_username("jiggai").is_ok());
        assert!(validate_username("coder-of-the-west").is_ok());
        assert!(validate_username("abc").is_ok());
        assert!(validate_username("JIGGAI").is_ok()); // normalized
        assert_eq!(validate_username("JIGGAI").unwrap(), "jiggai");
    }

    #[test]
    fn test_invalid_usernames() {
        assert!(validate_username("ab").is_err()); // too short
        assert!(validate_username("-bad").is_err()); // starts with hyphen
        assert!(validate_username("bad-").is_err()); // ends with hyphen
        assert!(validate_username("api").is_err()); // reserved
        assert!(validate_username("admin").is_err()); // reserved
        assert!(validate_username("register").is_err()); // reserved
    }

    #[test]
    fn test_validate_pubkey() {
        // 66-char compressed (valid)
        assert!(validate_pubkey("02a1633cafcc01ebfb6d78e39f687a1f0995c62fc95f51ead10a02ee0be551b5dc"));
        // 130-char uncompressed (valid)
        let uncompressed = "04".to_string() + &"a".repeat(128);
        assert!(validate_pubkey(&uncompressed));
        // Invalid lengths
        assert!(!validate_pubkey("deadbeef"));
        assert!(!validate_pubkey(""));
        // Non-hex chars
        assert!(!validate_pubkey(&"g".repeat(66)));
    }

    #[test]
    fn test_compute_profile_score_empty() {
        let input = ScoreInput {
            display_name: "", tagline: "", bio: "", avatar_url: "", pubkey: "",
            has_address: false, has_link: false, has_section: false, has_skill: false,
        };
        assert_eq!(compute_profile_score(&input), 0);
    }

    #[test]
    fn test_compute_profile_score_full() {
        let bio = "a".repeat(25);
        let input = ScoreInput {
            display_name: "Test Name", tagline: "A tagline", bio: &bio,
            avatar_url: "https://example.com/avatar.png", pubkey: "02a1633caf...",
            has_address: true, has_link: true, has_section: true, has_skill: true,
        };
        assert_eq!(compute_profile_score(&input), 100);
    }

    #[test]
    fn test_compute_profile_score_partial() {
        let input = ScoreInput {
            display_name: "Name", tagline: "", bio: "", avatar_url: "", pubkey: "",
            has_address: false, has_link: false, has_section: false, has_skill: false,
        };
        assert_eq!(compute_profile_score(&input), 10); // only display_name
    }

    #[test]
    fn test_compute_profile_score_no_pubkey() {
        let bio = "a".repeat(25);
        let input = ScoreInput {
            display_name: "Name", tagline: "tag", bio: &bio,
            avatar_url: "https://x.com/a.png", pubkey: "",
            has_address: true, has_link: true, has_section: true, has_skill: true,
        };
        // 10+10+20+10+0+10+5+10+5 = 80 (no pubkey)
        assert_eq!(compute_profile_score(&input), 80);
    }

    #[test]
    fn test_score_breakdown_count() {
        let input = ScoreInput {
            display_name: "", tagline: "", bio: "", avatar_url: "", pubkey: "",
            has_address: false, has_link: false, has_section: false, has_skill: false,
        };
        let breakdown = score_breakdown(&input);
        assert_eq!(breakdown.len(), 9);
        assert!(breakdown.iter().all(|item| !item.earned));
    }

    #[test]
    fn test_score_next_steps_full_profile() {
        let bio = "a".repeat(25);
        let input = ScoreInput {
            display_name: "Name", tagline: "tag", bio: &bio,
            avatar_url: "https://x.com/a.png",
            pubkey: "02a1633cafcc01ebfb6d78e39f687a1f0995c62fc95f51ead10a02ee0be551b5dc",
            has_address: true, has_link: true, has_section: true, has_skill: true,
        };
        let steps = score_next_steps(&input);
        assert!(steps.is_empty(), "Full profile should have no next steps");
    }
}
