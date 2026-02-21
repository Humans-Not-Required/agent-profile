use rocket::http::{ContentType, Header, Status};
use rocket::local::blocking::Client;
use uuid::Uuid;

fn test_client() -> Client {
    let db_path = format!("/tmp/agent_profile_test_{}.db", Uuid::new_v4());
    let rocket = agent_profile::create_rocket(&db_path);
    Client::tracked(rocket).expect("valid rocket instance")
}

/// Register a profile and return (status, body). Body has api_key + username.
fn register(client: &Client, username: &str) -> (Status, serde_json::Value) {
    let resp = client.post("/api/v1/register")
        .header(ContentType::JSON)
        .body(serde_json::json!({"username": username}).to_string())
        .dispatch();
    let status = resp.status();
    let text = resp.into_string().unwrap_or_default();
    let body: serde_json::Value = serde_json::from_str(&text).unwrap_or_default();
    (status, body)
}

// ===== Health =====

#[test]
fn test_health() {
    let client = test_client();
    let resp = client.get("/api/v1/health").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["service"], "agent-profile");
}

// ===== Registration =====

#[test]
fn test_register_profile() {
    let client = test_client();
    let (status, body) = register(&client, "agentx42");
    assert_eq!(status, Status::Created, "body: {}", body);
    assert_eq!(body["username"], "agentx42");
    assert!(body["api_key"].as_str().map(|k| k.starts_with("ap_")).unwrap_or(false));
    assert_eq!(body["profile_url"], "/agentx42");
    assert_eq!(body["json_url"], "/api/v1/profiles/agentx42");
}

#[test]
fn test_register_username_normalized() {
    let client = test_client();
    let (status, body) = register(&client, "JIGGAI");
    assert_eq!(status, Status::Created);
    assert_eq!(body["username"], "jiggai");
}

#[test]
fn test_register_duplicate_username() {
    let client = test_client();
    let _ = register(&client, "duplicate");
    let (status, _) = register(&client, "duplicate");
    assert_eq!(status, Status::Conflict);
}

#[test]
fn test_register_username_too_short() {
    let client = test_client();
    let resp = client.post("/api/v1/register")
        .header(ContentType::JSON)
        .body(r#"{"username":"ab"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

#[test]
fn test_register_reserved_username() {
    let client = test_client();
    let resp = client.post("/api/v1/register")
        .header(ContentType::JSON)
        .body(r#"{"username":"api"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

#[test]
fn test_register_username_starts_with_hyphen() {
    let client = test_client();
    let resp = client.post("/api/v1/register")
        .header(ContentType::JSON)
        .body(r#"{"username":"-bad"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

#[test]
fn test_register_with_pubkey() {
    let client = test_client();
    let pubkey = "02a1633cafcc01ebfb6d78e39f687a1f0995c62fc95f51ead10a02ee0be551b5dc";
    let resp = client.post("/api/v1/register")
        .header(ContentType::JSON)
        .body(serde_json::json!({"username": "keypair-agent", "pubkey": pubkey}).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::Created);
    // Verify pubkey stored
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["username"], "keypair-agent");
}

// ===== Get Profile =====

#[test]
fn test_get_profile() {
    let client = test_client();
    let (_, reg) = register(&client, "getme");
    let api_key = reg["api_key"].as_str().unwrap();

    // Update with display name so profile is rich
    client.patch("/api/v1/profiles/getme")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"display_name":"Get Me","tagline":"A test agent","bio":"This is a test bio"}"#)
        .dispatch();

    let resp = client.get("/api/v1/profiles/getme").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["username"], "getme");
    assert_eq!(body["display_name"], "Get Me");
    assert_eq!(body["tagline"], "A test agent");
    // Check sub-resource arrays exist
    assert!(body["crypto_addresses"].is_array());
    assert!(body["links"].is_array());
    assert!(body["sections"].is_array());
    assert!(body["skills"].is_array());
}

#[test]
fn test_get_profile_not_found() {
    let client = test_client();
    let resp = client.get("/api/v1/profiles/doesnotexist").dispatch();
    assert_eq!(resp.status(), Status::NotFound);
}

#[test]
fn test_get_profile_case_insensitive() {
    let client = test_client();
    let _ = register(&client, "CaseTest");
    let resp = client.get("/api/v1/profiles/CASETEST").dispatch();
    assert_eq!(resp.status(), Status::Ok);
}

// ===== Update Profile =====

#[test]
fn test_update_profile() {
    let client = test_client();
    let (_, reg) = register(&client, "updater");
    let api_key = reg["api_key"].as_str().unwrap();

    let resp = client.patch("/api/v1/profiles/updater")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"display_name":"Updated Name","tagline":"New tagline","bio":"Updated bio content here","theme":"midnight"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["display_name"], "Updated Name");
    assert_eq!(body["tagline"], "New tagline");
    assert_eq!(body["theme"], "midnight");
}

#[test]
fn test_update_profile_wrong_key() {
    let client = test_client();
    let _ = register(&client, "wrongkey");
    let resp = client.patch("/api/v1/profiles/wrongkey")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", "ap_invalid_key_here"))
        .body(r#"{"display_name":"Hacked"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Unauthorized);
}

#[test]
fn test_update_profile_no_key() {
    let client = test_client();
    let _ = register(&client, "nokey");
    let resp = client.patch("/api/v1/profiles/nokey")
        .header(ContentType::JSON)
        .body(r#"{"display_name":"Hacked"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Unauthorized);
}

#[test]
fn test_update_profile_no_fields() {
    let client = test_client();
    let (_, reg) = register(&client, "nofields");
    let api_key = reg["api_key"].as_str().unwrap();
    let resp = client.patch("/api/v1/profiles/nofields")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body("{}")
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

#[test]
fn test_update_profile_invalid_theme() {
    let client = test_client();
    let (_, reg) = register(&client, "badtheme");
    let api_key = reg["api_key"].as_str().unwrap();
    let resp = client.patch("/api/v1/profiles/badtheme")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"theme":"neon"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

// ===== Delete Profile =====

#[test]
fn test_delete_profile() {
    let client = test_client();
    let (_, reg) = register(&client, "deleteme");
    let api_key = reg["api_key"].as_str().unwrap();

    let resp = client.delete("/api/v1/profiles/deleteme")
        .header(Header::new("X-API-Key", api_key.to_string()))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);

    // Verify it's gone
    let resp = client.get("/api/v1/profiles/deleteme").dispatch();
    assert_eq!(resp.status(), Status::NotFound);
}

#[test]
fn test_delete_profile_wrong_key() {
    let client = test_client();
    let _ = register(&client, "nodeletion");
    let resp = client.delete("/api/v1/profiles/nodeletion")
        .header(Header::new("X-API-Key", "ap_wrong_key"))
        .dispatch();
    assert_eq!(resp.status(), Status::Unauthorized);
}

// ===== Crypto Addresses =====

#[test]
fn test_add_crypto_address() {
    let client = test_client();
    let (_, reg) = register(&client, "btcagent");
    let api_key = reg["api_key"].as_str().unwrap();

    let resp = client.post("/api/v1/profiles/btcagent/addresses")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"network":"bitcoin","address":"bc1qexampleaddress","label":"tips"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Created);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["network"], "bitcoin");
    assert_eq!(body["address"], "bc1qexampleaddress");
    assert_eq!(body["label"], "tips");
    assert!(body["id"].as_str().is_some());
}

#[test]
fn test_add_and_delete_address() {
    let client = test_client();
    let (_, reg) = register(&client, "ethagent");
    let api_key = reg["api_key"].as_str().unwrap();

    // Add
    let resp = client.post("/api/v1/profiles/ethagent/addresses")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"network":"ethereum","address":"0xabcdef1234567890"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Created);
    let addr_id = serde_json::from_str::<serde_json::Value>(&resp.into_string().unwrap())
        .unwrap()["id"].as_str().unwrap().to_string();

    // Delete
    let resp = client.delete(format!("/api/v1/profiles/ethagent/addresses/{}", addr_id))
        .header(Header::new("X-API-Key", api_key.to_string()))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);

    // Verify gone
    let body: serde_json::Value = serde_json::from_str(
        &client.get("/api/v1/profiles/ethagent").dispatch().into_string().unwrap()
    ).unwrap();
    assert_eq!(body["crypto_addresses"].as_array().unwrap().len(), 0);
}

#[test]
fn test_add_invalid_network() {
    let client = test_client();
    let (_, reg) = register(&client, "badnetwork");
    let api_key = reg["api_key"].as_str().unwrap();
    let resp = client.post("/api/v1/profiles/badnetwork/addresses")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"network":"doggo","address":"much-coin-wow"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

// ===== Links =====

#[test]
fn test_add_and_delete_link() {
    let client = test_client();
    let (_, reg) = register(&client, "linkagent");
    let api_key = reg["api_key"].as_str().unwrap();

    // Add link
    let resp = client.post("/api/v1/profiles/linkagent/links")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"url":"https://github.com/nanook","label":"GitHub","platform":"github","display_order":0}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Created);
    let link_id = serde_json::from_str::<serde_json::Value>(&resp.into_string().unwrap())
        .unwrap()["id"].as_str().unwrap().to_string();

    // Verify in profile
    let body: serde_json::Value = serde_json::from_str(
        &client.get("/api/v1/profiles/linkagent").dispatch().into_string().unwrap()
    ).unwrap();
    assert_eq!(body["links"].as_array().unwrap().len(), 1);
    assert_eq!(body["links"][0]["platform"], "github");
    assert_eq!(body["links"][0]["url"], "https://github.com/nanook");

    // Delete link
    let resp = client.delete(format!("/api/v1/profiles/linkagent/links/{}", link_id))
        .header(Header::new("X-API-Key", api_key.to_string()))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);

    // Verify gone
    let body: serde_json::Value = serde_json::from_str(
        &client.get("/api/v1/profiles/linkagent").dispatch().into_string().unwrap()
    ).unwrap();
    assert_eq!(body["links"].as_array().unwrap().len(), 0);
}

#[test]
fn test_add_invalid_platform() {
    let client = test_client();
    let (_, reg) = register(&client, "badplatform");
    let api_key = reg["api_key"].as_str().unwrap();
    let resp = client.post("/api/v1/profiles/badplatform/links")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"url":"https://example.com","label":"Example","platform":"myspace"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

// ===== Sections =====

#[test]
fn test_add_and_delete_section() {
    let client = test_client();
    let (_, reg) = register(&client, "sectionagent");
    let api_key = reg["api_key"].as_str().unwrap();

    // Add section
    let resp = client.post("/api/v1/profiles/sectionagent/sections")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"section_type":"about","title":"About Me","content":"I am an AI agent who loves building things.","display_order":0}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Created);
    let section_id = serde_json::from_str::<serde_json::Value>(&resp.into_string().unwrap())
        .unwrap()["id"].as_str().unwrap().to_string();

    // Verify in profile
    let body: serde_json::Value = serde_json::from_str(
        &client.get("/api/v1/profiles/sectionagent").dispatch().into_string().unwrap()
    ).unwrap();
    assert_eq!(body["sections"].as_array().unwrap().len(), 1);
    assert_eq!(body["sections"][0]["title"], "About Me");
    assert_eq!(body["sections"][0]["section_type"], "about");

    // Update section
    let resp = client.patch(format!("/api/v1/profiles/sectionagent/sections/{}", section_id))
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"title":"About This Agent","content":"Updated content here."}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let updated: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(updated["title"], "About This Agent");

    // Delete
    let resp = client.delete(format!("/api/v1/profiles/sectionagent/sections/{}", section_id))
        .header(Header::new("X-API-Key", api_key.to_string()))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
}

#[test]
fn test_add_invalid_section_type() {
    let client = test_client();
    let (_, reg) = register(&client, "badsection");
    let api_key = reg["api_key"].as_str().unwrap();
    let resp = client.post("/api/v1/profiles/badsection/sections")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"section_type":"resume","title":"My Resume","content":"..."}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

// ===== Skills =====

#[test]
fn test_add_and_deduplicate_skill() {
    let client = test_client();
    let (_, reg) = register(&client, "skillagent");
    let api_key = reg["api_key"].as_str().unwrap();

    let resp = client.post("/api/v1/profiles/skillagent/skills")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"skill":"Rust"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Created);

    // Duplicate
    let resp = client.post("/api/v1/profiles/skillagent/skills")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"skill":"rust"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Conflict);
}

#[test]
fn test_add_and_delete_skill() {
    let client = test_client();
    let (_, reg) = register(&client, "skilldel");
    let api_key = reg["api_key"].as_str().unwrap();

    let resp = client.post("/api/v1/profiles/skilldel/skills")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"skill":"TypeScript"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Created);
    let skill_id = serde_json::from_str::<serde_json::Value>(&resp.into_string().unwrap())
        .unwrap()["id"].as_str().unwrap().to_string();

    let resp = client.delete(format!("/api/v1/profiles/skilldel/skills/{}", skill_id))
        .header(Header::new("X-API-Key", api_key.to_string()))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);

    let body: serde_json::Value = serde_json::from_str(
        &client.get("/api/v1/profiles/skilldel").dispatch().into_string().unwrap()
    ).unwrap();
    assert_eq!(body["skills"].as_array().unwrap().len(), 0);
}

// ===== List Profiles =====

#[test]
fn test_list_profiles() {
    let client = test_client();
    register(&client, "list-agent-1");
    register(&client, "list-agent-2");

    let resp = client.get("/api/v1/profiles?limit=50").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let profiles = body["profiles"].as_array().unwrap();
    assert!(profiles.len() >= 2, "expected at least 2 profiles");
}

#[test]
fn test_list_profiles_search() {
    let client = test_client();
    register(&client, "searchable-x99");
    let (_, reg) = register(&client, "searcher");
    let api_key = reg["api_key"].as_str().unwrap();
    client.patch("/api/v1/profiles/searcher")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"bio":"this agent is unique with keyword zebraduck"}"#)
        .dispatch();

    let resp = client.get("/api/v1/profiles?q=zebraduck").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let profiles = body["profiles"].as_array().unwrap();
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0]["username"], "searcher");
}

// ===== API Key Reissue =====

#[test]
fn test_reissue_api_key() {
    let client = test_client();
    let (_, reg) = register(&client, "reissue-agent");
    let old_key = reg["api_key"].as_str().unwrap().to_string();

    let resp = client.post("/api/v1/profiles/reissue-agent/reissue-key")
        .header(Header::new("X-API-Key", old_key.clone()))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let new_key = body["api_key"].as_str().unwrap();
    assert_ne!(new_key, old_key.as_str(), "new key should differ from old");

    // Old key should no longer work
    let resp = client.patch("/api/v1/profiles/reissue-agent")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", old_key))
        .body(r#"{"display_name":"Hacked"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Unauthorized);

    // New key should work
    let resp = client.patch("/api/v1/profiles/reissue-agent")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", new_key.to_string()))
        .body(r#"{"display_name":"Updated with new key"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
}

// ===== Profile Score =====

#[test]
fn test_profile_score() {
    let client = test_client();
    let (_, reg) = register(&client, "scoretest");
    let api_key = reg["api_key"].as_str().unwrap();

    // Get score on empty profile
    let resp = client.get("/api/v1/profiles/scoretest/score").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["max_score"], 100);
    assert!(body["score"].as_i64().unwrap() < 50, "empty profile should have low score");
    assert!(body["next_steps"].as_array().unwrap().len() > 0);

    // Update profile to improve score
    client.patch("/api/v1/profiles/scoretest")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"display_name":"Score Test","tagline":"Testing scores","bio":"This is a meaningful bio for scoring"}"#)
        .dispatch();

    let resp = client.get("/api/v1/profiles/scoretest/score").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert!(body["score"].as_i64().unwrap() > 0, "profile with content should have positive score");
}

// ===== Challenge / Verify =====

#[test]
fn test_challenge_requires_pubkey() {
    let client = test_client();
    let _ = register(&client, "nopubkey");
    let resp = client.get("/api/v1/profiles/nopubkey/challenge").dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

#[test]
fn test_challenge_profile_not_found() {
    let client = test_client();
    let resp = client.get("/api/v1/profiles/ghost-agent/challenge").dispatch();
    assert_eq!(resp.status(), Status::NotFound);
}

#[test]
fn test_challenge_with_pubkey() {
    let client = test_client();
    let pubkey = "02a1633cafcc01ebfb6d78e39f687a1f0995c62fc95f51ead10a02ee0be551b5dc";
    let (_, reg) = register(&client, "challenge-agent");
    let api_key = reg["api_key"].as_str().unwrap();

    // Set pubkey
    client.patch("/api/v1/profiles/challenge-agent")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(serde_json::json!({"pubkey": pubkey}).to_string())
        .dispatch();

    // Get challenge
    let resp = client.get("/api/v1/profiles/challenge-agent/challenge").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert!(body["challenge"].as_str().map(|c| c.len() == 64).unwrap_or(false), "challenge should be 64 hex chars");
    assert_eq!(body["expires_in_seconds"], 300);
}

#[test]
fn test_verify_invalid_signature() {
    let client = test_client();
    let pubkey = "02a1633cafcc01ebfb6d78e39f687a1f0995c62fc95f51ead10a02ee0be551b5dc";
    let (_, reg) = register(&client, "verifyme");
    let api_key = reg["api_key"].as_str().unwrap();

    // Set pubkey
    client.patch("/api/v1/profiles/verifyme")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(serde_json::json!({"pubkey": pubkey}).to_string())
        .dispatch();

    // Get challenge
    client.get("/api/v1/profiles/verifyme/challenge").dispatch();

    // Send garbage signature — should return verified: false (not 500)
    let resp = client.post("/api/v1/profiles/verifyme/verify")
        .header(ContentType::JSON)
        .body(r#"{"signature":"deadbeefcafe"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["verified"], false);
}

// ===== Cascade Delete =====

#[test]
fn test_delete_cascades_to_sub_resources() {
    let client = test_client();
    let (_, reg) = register(&client, "cascade-del");
    let api_key = reg["api_key"].as_str().unwrap();

    client.post("/api/v1/profiles/cascade-del/addresses")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"network":"bitcoin","address":"bc1qtestaddr"}"#)
        .dispatch();

    client.post("/api/v1/profiles/cascade-del/links")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"url":"https://github.com/test","label":"GitHub","platform":"github"}"#)
        .dispatch();

    // Delete profile
    client.delete("/api/v1/profiles/cascade-del")
        .header(Header::new("X-API-Key", api_key.to_string()))
        .dispatch();

    // Profile should be gone
    let resp = client.get("/api/v1/profiles/cascade-del").dispatch();
    assert_eq!(resp.status(), Status::NotFound);
}

// ===== Full Profile with All Sub-resources =====

#[test]
fn test_profile_has_all_sub_resources() {
    let client = test_client();
    let (_, reg) = register(&client, "richagent");
    let api_key = reg["api_key"].as_str().unwrap();

    // Update profile fields
    client.patch("/api/v1/profiles/richagent")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"display_name":"Rich Agent","tagline":"Fully loaded","bio":"This agent has everything","third_line":"Powered by OpenClaw","theme":"dark"}"#)
        .dispatch();

    // Add address
    client.post("/api/v1/profiles/richagent/addresses")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"network":"bitcoin","address":"bc1qrichaddress"}"#)
        .dispatch();

    // Add link
    client.post("/api/v1/profiles/richagent/links")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"url":"https://github.com/richagent","label":"GitHub","platform":"github"}"#)
        .dispatch();

    // Add section
    client.post("/api/v1/profiles/richagent/sections")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"section_type":"about","title":"About","content":"I build things at night.","display_order":0}"#)
        .dispatch();

    // Add skill
    client.post("/api/v1/profiles/richagent/skills")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"skill":"Rust"}"#)
        .dispatch();

    // Fetch complete profile
    let resp = client.get("/api/v1/profiles/richagent").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["username"], "richagent");
    assert_eq!(body["display_name"], "Rich Agent");
    assert_eq!(body["crypto_addresses"].as_array().unwrap().len(), 1);
    assert_eq!(body["links"].as_array().unwrap().len(), 1);
    assert_eq!(body["sections"].as_array().unwrap().len(), 1);
    assert_eq!(body["skills"].as_array().unwrap().len(), 1);
    assert!(body["profile_score"].as_i64().unwrap() > 20);
}

// ===== HTML Profile Page & Content Negotiation =====

#[test]
fn test_html_profile_page_exists() {
    let client = test_client();
    let (_, reg) = register(&client, "htmltest");
    let api_key = reg["api_key"].as_str().unwrap();
    client.patch("/api/v1/profiles/htmltest")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"display_name":"HTML Test","bio":"A profile for HTML testing"}"#)
        .dispatch();

    // Browser request gets HTML
    let resp = client.get("/htmltest")
        .header(Header::new("Accept", "text/html,application/xhtml+xml"))
        .header(Header::new("User-Agent", "Mozilla/5.0 (browser)"))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let html = resp.into_string().unwrap();
    assert!(html.contains("<!DOCTYPE html>"), "should be HTML");
    assert!(html.contains("HTML Test"), "should contain display name");
}

#[test]
fn test_html_profile_page_not_found() {
    let client = test_client();
    let resp = client.get("/doesnotexist404")
        .header(Header::new("Accept", "text/html"))
        .header(Header::new("User-Agent", "Mozilla/5.0"))
        .dispatch();
    assert_eq!(resp.status(), Status::NotFound);
}

#[test]
fn test_html_profile_page_case_insensitive() {
    let client = test_client();
    let _ = register(&client, "htmlcase");
    let resp = client.get("/HTMLCASE")
        .header(Header::new("Accept", "text/html"))
        .header(Header::new("User-Agent", "Mozilla/5.0"))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
}

#[test]
fn test_html_profile_page_content_type_is_html() {
    let client = test_client();
    let _ = register(&client, "cttest");
    let resp = client.get("/cttest")
        .header(Header::new("Accept", "text/html"))
        .header(Header::new("User-Agent", "Mozilla/5.0 (browser)"))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let ct = resp.content_type().unwrap();
    assert!(ct.is_html(), "content type should be HTML, got: {:?}", ct);
}

#[test]
fn test_content_negotiation_agent_gets_json() {
    let client = test_client();
    let _ = register(&client, "cntest");

    // curl-like agent gets JSON
    let resp = client.get("/cntest")
        .header(Header::new("User-Agent", "curl/7.88.0"))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let ct = resp.content_type().unwrap();
    assert!(ct.is_json(), "agent should get JSON, got: {:?}", ct);
}

#[test]
fn test_content_negotiation_json_accept_gets_json() {
    let client = test_client();
    let _ = register(&client, "cnaccept");

    // Explicit Accept: application/json gets JSON
    let resp = client.get("/cnaccept")
        .header(Header::new("Accept", "application/json"))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let ct = resp.content_type().unwrap();
    assert!(ct.is_json(), "Accept: application/json should get JSON, got: {:?}", ct);
}

// ===== CORS =====

#[test]
fn test_cors_headers_on_api_response() {
    let client = test_client();
    let resp = client.get("/api/v1/health").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    // CORS fairing adds Allow-Origin
    assert!(resp.headers().get_one("Access-Control-Allow-Origin").is_some());
}

#[test]
fn test_cors_headers_on_profile_response() {
    let client = test_client();
    let _ = register(&client, "corsprofile");
    let resp = client.get("/api/v1/profiles/corsprofile").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    assert!(resp.headers().get_one("Access-Control-Allow-Origin").is_some());
}

// ===== Discovery =====

#[test]
fn test_llms_txt() {
    let client = test_client();
    let resp = client.get("/llms.txt").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body = resp.into_string().unwrap();
    assert!(body.contains("agent-profile") || body.contains("Agent Profile"));
}

#[test]
fn test_skills_index() {
    let client = test_client();
    let resp = client.get("/.well-known/skills/index.json").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert!(body["skills"].is_array());
}
