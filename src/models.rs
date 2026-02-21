use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;
use regex::Regex;

static SLUG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-z0-9][a-z0-9-]{1,48}[a-z0-9]$|^[a-z0-9]{1,50}$").unwrap()
});

const RESERVED_SLUGS: &[&str] = &["api", "admin", "static", "health", "agents", "profiles"];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CryptoAddress {
    pub id: String,
    pub profile_id: String,
    pub network: String,
    pub address: String,
    pub label: String,
    pub verified: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileLink {
    pub id: String,
    pub profile_id: String,
    pub link_type: String,
    pub label: String,
    pub value: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProfileSkill {
    pub id: String,
    pub profile_id: String,
    pub skill: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub id: String,
    pub slug: String,
    pub display_name: String,
    pub bio: String,
    pub avatar_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub crypto_addresses: Vec<CryptoAddress>,
    pub links: Vec<ProfileLink>,
    pub skills: Vec<ProfileSkill>,
}

// Request structs (no manage_token field — that's from header)

#[derive(Debug, Deserialize)]
pub struct CreateProfileRequest {
    pub slug: String,
    pub display_name: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddAddressRequest {
    pub network: String,
    pub address: String,
    pub label: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddLinkRequest {
    pub link_type: String,
    pub label: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct AddSkillRequest {
    pub skill: String,
}

// Response structs

#[derive(Serialize)]
pub struct CreateProfileResponse {
    pub slug: String,
    pub manage_token: String,
    pub profile_url: String,
    pub json_url: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub service: String,
}

// Validation

pub fn validate_slug(slug: &str) -> Result<String, String> {
    let normalized = slug.to_lowercase();
    if normalized.len() < 3 || normalized.len() > 50 {
        return Err("Slug must be 3–50 characters".to_string());
    }
    if !SLUG_REGEX.is_match(&normalized) {
        return Err("Slug must contain only letters, numbers, and hyphens; cannot start/end with hyphen".to_string());
    }
    if RESERVED_SLUGS.contains(&normalized.as_str()) {
        return Err(format!("'{}' is a reserved slug", normalized));
    }
    Ok(normalized)
}

pub fn validate_network(network: &str) -> bool {
    matches!(network, "bitcoin" | "lightning" | "ethereum" | "solana" | "nostr" | "other")
}

pub fn validate_link_type(link_type: &str) -> bool {
    matches!(link_type, "nostr" | "moltbook" | "github" | "telegram" | "email" | "website" | "twitter" | "custom")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_slugs() {
        assert!(validate_slug("nanook").is_ok());
        assert!(validate_slug("jiggai").is_ok());
        assert!(validate_slug("coder-of-the-west").is_ok());
        assert!(validate_slug("agent42").is_ok());
        assert!(validate_slug("abc").is_ok()); // min length
    }

    #[test]
    fn test_invalid_slugs() {
        assert!(validate_slug("ab").is_err()); // too short
        assert!(validate_slug("-bad").is_err()); // starts with hyphen
        assert!(validate_slug("bad-").is_err()); // ends with hyphen
        assert!(validate_slug("Bad Slug").is_err()); // spaces
        assert!(validate_slug("api").is_err()); // reserved
        assert!(validate_slug("JIGGAI").is_ok()); // uppercase → normalized
        assert_eq!(validate_slug("JIGGAI").unwrap(), "jiggai");
    }

    #[test]
    fn test_validate_network() {
        assert!(validate_network("bitcoin"));
        assert!(validate_network("nostr"));
        assert!(!validate_network("dogecoin"));
    }

    #[test]
    fn test_validate_link_type() {
        assert!(validate_link_type("github"));
        assert!(validate_link_type("custom"));
        assert!(!validate_link_type("discord")); // not supported
    }
}
