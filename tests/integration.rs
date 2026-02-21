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

    // Browser request gets the React SPA HTML shell
    let resp = client.get("/htmltest")
        .header(Header::new("Accept", "text/html,application/xhtml+xml"))
        .header(Header::new("User-Agent", "Mozilla/5.0 (browser)"))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let html = resp.into_string().unwrap();
    assert!(html.contains("<!DOCTYPE html>"), "should be HTML");
    // SPA shell: profile content is loaded dynamically by React, so just verify it's HTML
    assert!(html.contains("<div") || html.contains("<script"), "should contain SPA markup");
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

#[test]
fn test_openapi_json() {
    let client = test_client();
    let resp = client.get("/openapi.json").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    // Verify it's a valid OpenAPI 3.1 spec for v0.4.0
    assert_eq!(body["openapi"], "3.1.0");
    assert_eq!(body["info"]["version"], "0.4.0");
    assert!(body["paths"].is_object());
    // All key paths present
    let paths = body["paths"].as_object().unwrap();
    assert!(paths.contains_key("/health"));
    assert!(paths.contains_key("/register"));
    assert!(paths.contains_key("/profiles/{username}"));
    assert!(paths.contains_key("/profiles/{username}/challenge"));
    assert!(paths.contains_key("/profiles/{username}/verify"));
    assert!(paths.contains_key("/profiles/{username}/sections"));
    assert!(paths.contains_key("/profiles/{username}/score"));
    assert!(paths.contains_key("/profiles/{username}/avatar"));
}

// ===== Rate Limiting =====

#[test]
fn test_register_rate_limit() {
    let client = test_client();
    // Limit is 5 per hour. First 5 should succeed (or 409 conflict if duplicate).
    // 6th should be 429.
    let usernames = ["rl-a1", "rl-a2", "rl-a3", "rl-a4", "rl-a5"];
    for u in &usernames {
        let resp = client.post("/api/v1/register")
            .header(ContentType::JSON)
            .body(serde_json::json!({"username": u}).to_string())
            .dispatch();
        let s = resp.status();
        assert!(
            s == Status::Created || s == Status::Conflict,
            "expected 201 or 409, got {} for {}", s, u
        );
    }
    // 6th should hit rate limit
    let resp = client.post("/api/v1/register")
        .header(ContentType::JSON)
        .body(r#"{"username":"rl-a6"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::TooManyRequests, "6th registration should be rate limited");
}

#[test]
fn test_verify_rate_limit() {
    let client = test_client();
    let pubkey = "02a1633cafcc01ebfb6d78e39f687a1f0995c62fc95f51ead10a02ee0be551b5dc";
    let (_, reg) = register(&client, "rl-verify");
    let api_key = reg["api_key"].as_str().unwrap();

    // Set pubkey
    client.patch("/api/v1/profiles/rl-verify")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(serde_json::json!({"pubkey": pubkey}).to_string())
        .dispatch();

    // Get 3 challenges (limit per verify is 3 per 5 min)
    for _ in 0..3 {
        client.get("/api/v1/profiles/rl-verify/challenge").dispatch();
        let resp = client.post("/api/v1/profiles/rl-verify/verify")
            .header(ContentType::JSON)
            .body(r#"{"signature":"deadbeef"}"#)
            .dispatch();
        // Should be 200 (verified: false) not 429 yet
        assert_eq!(resp.status(), Status::Ok);
    }

    // 4th verify should be rate limited
    // (Need a fresh challenge first — challenge limit is 10/min so still OK)
    client.get("/api/v1/profiles/rl-verify/challenge").dispatch();
    let resp = client.post("/api/v1/profiles/rl-verify/verify")
        .header(ContentType::JSON)
        .body(r#"{"signature":"deadbeef"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::TooManyRequests, "4th verify should be rate limited");
}

// ===== Endorsements =====

#[test]
fn test_add_endorsement_basic() {
    let client = test_client();
    // Register two agents
    let (_, reg_a) = register(&client, "endorser-a");
    let (_, _reg_b) = register(&client, "endorsee-b");
    let key_a = reg_a["api_key"].as_str().unwrap();

    // A endorses B
    let resp = client.post("/api/v1/profiles/endorsee-b/endorsements")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key_a.to_string()))
        .body(serde_json::json!({
            "from": "endorser-a",
            "message": "Great collaborator. Highly trustworthy agent."
        }).to_string())
        .dispatch();

    assert_eq!(resp.status(), Status::Ok, "endorsement should succeed");
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["endorser"], "endorser-a");
    assert_eq!(body["endorsee"], "endorsee-b");
    assert_eq!(body["verified"], false); // no signature provided
    assert!(body["id"].as_str().is_some());
}

#[test]
fn test_get_endorsements() {
    let client = test_client();
    let (_, reg_a) = register(&client, "get-end-a");
    let (_, _reg_b) = register(&client, "get-end-b");
    let key_a = reg_a["api_key"].as_str().unwrap();

    // Add endorsement
    client.post("/api/v1/profiles/get-end-b/endorsements")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key_a.to_string()))
        .body(serde_json::json!({"from": "get-end-a", "message": "Solid agent."}).to_string())
        .dispatch();

    // Fetch endorsements (public, no auth needed)
    let resp = client.get("/api/v1/profiles/get-end-b/endorsements").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["total"], 1);
    assert_eq!(body["endorsements"][0]["endorser_username"], "get-end-a");
    assert_eq!(body["endorsements"][0]["message"], "Solid agent.");
}

#[test]
fn test_endorsement_in_profile_json() {
    let client = test_client();
    let (_, reg_a) = register(&client, "ep-src");
    let (_, _reg_b) = register(&client, "ep-dst");
    let key_a = reg_a["api_key"].as_str().unwrap();

    // Endorse
    client.post("/api/v1/profiles/ep-dst/endorsements")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key_a.to_string()))
        .body(serde_json::json!({"from": "ep-src", "message": "Excellent partner."}).to_string())
        .dispatch();

    // Profile JSON includes endorsements
    let resp = client.get("/api/v1/profiles/ep-dst").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let endorsements = body["endorsements"].as_array().unwrap();
    assert_eq!(endorsements.len(), 1);
    assert_eq!(endorsements[0]["endorser_username"], "ep-src");
}

#[test]
fn test_endorsement_no_self_endorse() {
    let client = test_client();
    let (_, reg_a) = register(&client, "self-end");
    let key_a = reg_a["api_key"].as_str().unwrap();

    let resp = client.post("/api/v1/profiles/self-end/endorsements")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key_a.to_string()))
        .body(serde_json::json!({"from": "self-end", "message": "I'm great!"}).to_string())
        .dispatch();

    assert_eq!(resp.status(), Status::UnprocessableEntity);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert!(body["error"].as_str().unwrap().contains("own profile"));
}

#[test]
fn test_endorsement_wrong_api_key() {
    let client = test_client();
    let (_, _reg_a) = register(&client, "wk-end-a");
    let (_, reg_b) = register(&client, "wk-end-b");
    let key_b = reg_b["api_key"].as_str().unwrap();

    // B's key used but from=a — should fail
    let resp = client.post("/api/v1/profiles/wk-end-b/endorsements")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key_b.to_string()))
        .body(serde_json::json!({"from": "wk-end-a", "message": "Not legitimate."}).to_string())
        .dispatch();

    assert_eq!(resp.status(), Status::Unauthorized);
}

#[test]
fn test_endorsement_upsert() {
    // Endorsing again updates the message rather than creating a duplicate
    let client = test_client();
    let (_, reg_a) = register(&client, "ups-end-a");
    let (_, _reg_b) = register(&client, "ups-end-b");
    let key_a = reg_a["api_key"].as_str().unwrap();

    // First endorsement
    client.post("/api/v1/profiles/ups-end-b/endorsements")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key_a.to_string()))
        .body(serde_json::json!({"from": "ups-end-a", "message": "First impression."}).to_string())
        .dispatch();

    // Second endorsement (update)
    client.post("/api/v1/profiles/ups-end-b/endorsements")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key_a.to_string()))
        .body(serde_json::json!({"from": "ups-end-a", "message": "Updated: even better."}).to_string())
        .dispatch();

    // Should only have 1 endorsement with the updated message
    let resp = client.get("/api/v1/profiles/ups-end-b/endorsements").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["total"], 1, "should only have one endorsement");
    assert_eq!(body["endorsements"][0]["message"], "Updated: even better.");
}

#[test]
fn test_delete_endorsement_by_endorser() {
    let client = test_client();
    let (_, reg_a) = register(&client, "del-end-a");
    let (_, _reg_b) = register(&client, "del-end-b");
    let key_a = reg_a["api_key"].as_str().unwrap();

    // Add endorsement
    client.post("/api/v1/profiles/del-end-b/endorsements")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key_a.to_string()))
        .body(serde_json::json!({"from": "del-end-a", "message": "Temp endorsement."}).to_string())
        .dispatch();

    // Delete it (endorser uses their own key)
    let resp = client.delete("/api/v1/profiles/del-end-b/endorsements/del-end-a")
        .header(Header::new("X-API-Key", key_a.to_string()))
        .dispatch();

    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["deleted"], true);

    // Verify it's gone
    let resp2 = client.get("/api/v1/profiles/del-end-b/endorsements").dispatch();
    let body2: serde_json::Value = serde_json::from_str(&resp2.into_string().unwrap()).unwrap();
    assert_eq!(body2["total"], 0);
}

#[test]
fn test_delete_endorsement_by_endorsee() {
    let client = test_client();
    let (_, reg_a) = register(&client, "dlee-src");
    let (_, reg_b) = register(&client, "dlee-dst");
    let key_a = reg_a["api_key"].as_str().unwrap();
    let key_b = reg_b["api_key"].as_str().unwrap();

    // A endorses B
    client.post("/api/v1/profiles/dlee-dst/endorsements")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key_a.to_string()))
        .body(serde_json::json!({"from": "dlee-src", "message": "Unwanted endorsement."}).to_string())
        .dispatch();

    // B removes it using their own key (endorsee can also delete)
    let resp = client.delete("/api/v1/profiles/dlee-dst/endorsements/dlee-src")
        .header(Header::new("X-API-Key", key_b.to_string()))
        .dispatch();

    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["deleted"], true);
}

#[test]
fn test_endorsement_message_too_long() {
    let client = test_client();
    let (_, reg_a) = register(&client, "long-end-a");
    let (_, _reg_b) = register(&client, "long-end-b");
    let key_a = reg_a["api_key"].as_str().unwrap();

    let long_msg = "x".repeat(501);
    let resp = client.post("/api/v1/profiles/long-end-b/endorsements")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key_a.to_string()))
        .body(serde_json::json!({"from": "long-end-a", "message": long_msg}).to_string())
        .dispatch();

    assert_eq!(resp.status(), Status::UnprocessableEntity);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert!(body["error"].as_str().unwrap().contains("500"));
}

// ===== Skill-based search =====

#[test]
fn test_list_profiles_by_skill() {
    let client = test_client();
    let (_, reg_a) = register(&client, "skill-rust-agent");
    let (_, reg_b) = register(&client, "skill-python-agent");
    let key_a = reg_a["api_key"].as_str().unwrap();
    let key_b = reg_b["api_key"].as_str().unwrap();

    // Add skills
    client.post("/api/v1/profiles/skill-rust-agent/skills")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key_a.to_string()))
        .body(r#"{"skill":"Rust"}"#)
        .dispatch();
    client.post("/api/v1/profiles/skill-python-agent/skills")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key_b.to_string()))
        .body(r#"{"skill":"Python"}"#)
        .dispatch();

    // Filter by skill=Rust → only rust agent
    let resp = client.get("/api/v1/profiles?skill=Rust").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let profiles = body["profiles"].as_array().unwrap();
    assert_eq!(profiles.len(), 1, "should find exactly 1 Rust profile");
    assert_eq!(profiles[0]["username"], "skill-rust-agent");
}

#[test]
fn test_list_profiles_skill_case_insensitive() {
    let client = test_client();
    let (_, reg) = register(&client, "skill-ci-agent");
    let key = reg["api_key"].as_str().unwrap();
    client.post("/api/v1/profiles/skill-ci-agent/skills")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.to_string()))
        .body(r#"{"skill":"TypeScript"}"#)
        .dispatch();

    // Search with different case
    let resp = client.get("/api/v1/profiles?skill=typescript").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let profiles = body["profiles"].as_array().unwrap();
    assert!(profiles.iter().any(|p| p["username"] == "skill-ci-agent"),
        "case-insensitive skill search should find TypeScript with 'typescript'");
}

#[test]
fn test_list_profiles_skill_no_match() {
    let client = test_client();
    let resp = client.get("/api/v1/profiles?skill=COBOL9000NONEXISTENT").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["total"], 0);
    assert_eq!(body["profiles"].as_array().unwrap().len(), 0);
}

#[test]
fn test_list_profiles_has_pubkey() {
    let client = test_client();
    let pubkey = "02a1633cafcc01ebfb6d78e39f687a1f0995c62fc95f51ead10a02ee0be551b5dc";
    let (_, reg_pk) = register(&client, "hpk-with-key");
    let (_, _reg_no) = register(&client, "hpk-no-key");
    let key_pk = reg_pk["api_key"].as_str().unwrap();

    client.patch("/api/v1/profiles/hpk-with-key")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key_pk.to_string()))
        .body(serde_json::json!({"pubkey": pubkey}).to_string())
        .dispatch();

    let resp = client.get("/api/v1/profiles?has_pubkey=true").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let profiles = body["profiles"].as_array().unwrap();
    // All returned profiles must have a pubkey
    assert!(!profiles.is_empty(), "should find at least one profile with pubkey");
    assert!(profiles.iter().all(|p| {
        p["username"] != "hpk-no-key"
    }), "profile without pubkey should not appear");
}

#[test]
fn test_list_profiles_skill_and_query_combined() {
    let client = test_client();
    let (_, reg) = register(&client, "combo-search-agent");
    let key = reg["api_key"].as_str().unwrap();

    client.post("/api/v1/profiles/combo-search-agent/skills")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.to_string()))
        .body(r#"{"skill":"Go"}"#)
        .dispatch();
    client.patch("/api/v1/profiles/combo-search-agent")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.to_string()))
        .body(r#"{"bio":"builds distributed systems with zebraduck42 token"}"#)
        .dispatch();

    // Combined skill + text search
    let resp = client.get("/api/v1/profiles?skill=Go&q=zebraduck42").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let profiles = body["profiles"].as_array().unwrap();
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0]["username"], "combo-search-agent");
}

// ===== Skills Directory =====

#[test]
fn test_list_skills_empty() {
    let client = test_client();
    let resp = client.get("/api/v1/skills").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["skills"].as_array().unwrap().len(), 0);
    assert_eq!(body["total_distinct"], 0);
}

#[test]
fn test_list_skills_with_data() {
    let client = test_client();
    let (_, reg_a) = register(&client, "sd-agent-a");
    let (_, reg_b) = register(&client, "sd-agent-b");
    let key_a = reg_a["api_key"].as_str().unwrap();
    let key_b = reg_b["api_key"].as_str().unwrap();

    for skill in &["Rust", "Python"] {
        client.post("/api/v1/profiles/sd-agent-a/skills")
            .header(ContentType::JSON)
            .header(Header::new("X-API-Key", key_a.to_string()))
            .body(serde_json::json!({"skill": skill}).to_string())
            .dispatch();
    }
    client.post("/api/v1/profiles/sd-agent-b/skills")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key_b.to_string()))
        .body(r#"{"skill":"Rust"}"#)
        .dispatch();

    let resp = client.get("/api/v1/skills").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let skills = body["skills"].as_array().unwrap();
    // Rust should be first (count=2) and python second (count=1)
    assert_eq!(skills[0]["skill"], "rust");
    assert_eq!(skills[0]["count"], 2);
    assert_eq!(body["total_distinct"], 2);
}

#[test]
fn test_list_skills_search() {
    let client = test_client();
    let (_, reg) = register(&client, "sd-search-agent");
    let key = reg["api_key"].as_str().unwrap();
    for skill in &["TypeScript", "JavaScript", "Rust"] {
        client.post("/api/v1/profiles/sd-search-agent/skills")
            .header(ContentType::JSON)
            .header(Header::new("X-API-Key", key.to_string()))
            .body(serde_json::json!({"skill": skill}).to_string())
            .dispatch();
    }

    let resp = client.get("/api/v1/skills?q=script").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let skills = body["skills"].as_array().unwrap();
    assert_eq!(skills.len(), 2, "should find typescript and javascript");
    assert!(skills.iter().all(|s| s["skill"].as_str().unwrap().contains("script")));
}

// ===== Stats =====

#[test]
fn test_get_stats() {
    let client = test_client();
    let resp = client.get("/api/v1/stats").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    // Check structure
    assert!(body["profiles"]["total"].is_number());
    assert!(body["skills"]["distinct"].is_number());
    assert!(body["endorsements"]["total"].is_number());
    assert_eq!(body["service"]["name"], "agent-profile");
}

#[test]
fn test_get_stats_counts() {
    let client = test_client();
    let (_, reg_a) = register(&client, "stats-agent-a");
    let (_, reg_b) = register(&client, "stats-agent-b");
    let key_a = reg_a["api_key"].as_str().unwrap();
    let _key_b = reg_b["api_key"].as_str().unwrap();

    // Add skill + endorsement
    client.post("/api/v1/profiles/stats-agent-a/skills")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key_a.to_string()))
        .body(r#"{"skill":"Rust"}"#)
        .dispatch();
    client.post("/api/v1/profiles/stats-agent-b/endorsements")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key_a.to_string()))
        .body(serde_json::json!({"from": "stats-agent-a", "message": "Great agent."}).to_string())
        .dispatch();

    let resp = client.get("/api/v1/stats").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert!(body["profiles"]["total"].as_i64().unwrap() >= 2);
    assert!(body["skills"]["total_tags"].as_i64().unwrap() >= 1);
    assert!(body["endorsements"]["total"].as_i64().unwrap() >= 1);
}

// ===== Badge SVG =====

#[test]
fn test_badge_svg_existing_profile() {
    let client = test_client();
    let (_, reg) = register(&client, "badge-test-agent");
    let key = reg["api_key"].as_str().unwrap();

    // Add display name so score is non-zero
    client.patch("/api/v1/profiles/badge-test-agent")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.to_string()))
        .body(r#"{"display_name": "Badge Test Agent", "bio": "A test agent", "tagline": "Testing badges"}"#)
        .dispatch();

    let resp = client.get("/api/v1/profiles/badge-test-agent/badge.svg").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let ct = resp.content_type().unwrap();
    assert_eq!(ct.to_string(), "image/svg+xml");
    let body = resp.into_string().unwrap();
    assert!(body.contains("<svg"), "should return SVG");
    assert!(body.contains("agent score"), "label should be present");
    assert!(body.contains("/100"), "score /100 should be present");
    assert!(!body.contains("unknown"), "existing profile should not show unknown");
}

#[test]
fn test_badge_svg_not_found() {
    let client = test_client();
    let resp = client.get("/api/v1/profiles/nonexistent-badge-agent-xyz/badge.svg").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body = resp.into_string().unwrap();
    assert!(body.contains("<svg"), "should return SVG even for unknown profiles");
    assert!(body.contains("unknown"), "missing profile should show unknown");
    assert!(body.contains("9f9f9f"), "unknown badge should use gray color");
}

#[test]
fn test_badge_svg_high_score_is_green() {
    let client = test_client();
    let (_, reg) = register(&client, "badge-highscore-agent");
    let key = reg["api_key"].as_str().unwrap();

    // Boost score: display_name, bio, tagline, pubkey, skill
    let pubkey = "03b3e35e88e04bb22f30a9e98c5d2fabadf21da9d8b0e5b95e4e5f2d6c8a7b5c91";
    client.patch("/api/v1/profiles/badge-highscore-agent")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.to_string()))
        .body(serde_json::json!({"display_name":"High Score Agent","bio":"A reliable and capable autonomous agent.","tagline":"The best","pubkey":pubkey}).to_string())
        .dispatch();
    client.post("/api/v1/profiles/badge-highscore-agent/skills")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.to_string()))
        .body(r#"{"skill":"Rust"}"#).dispatch();

    let resp = client.get("/api/v1/profiles/badge-highscore-agent/badge.svg").dispatch();
    let body = resp.into_string().unwrap();
    // Score should be ≥60 with name+bio+tagline+pubkey+skill → green or yellow
    assert!(body.contains("4c1") || body.contains("dfb317"),
        "high score should produce green or yellow badge, got: {}", &body[..200.min(body.len())]);
}

// ===== Web Discovery (robots.txt, sitemap.xml) =====

#[test]
fn test_robots_txt() {
    let client = test_client();
    let resp = client.get("/robots.txt").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let ct = resp.content_type().unwrap();
    assert!(ct.to_string().contains("text/plain"), "content-type should be text/plain");
    let body = resp.into_string().unwrap();
    assert!(body.contains("User-agent: *"), "should have User-agent directive");
    assert!(body.contains("Allow: /"), "should allow all");
    assert!(body.contains("Sitemap:"), "should reference sitemap");
    assert!(body.contains("sitemap.xml"), "sitemap URL should mention sitemap.xml");
}

#[test]
fn test_sitemap_xml_structure() {
    let client = test_client();
    let resp = client.get("/sitemap.xml").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let ct = resp.content_type().unwrap();
    assert_eq!(ct.to_string(), "text/xml");
    let body = resp.into_string().unwrap();
    assert!(body.contains(r#"<?xml version="1.0""#), "should be valid XML prologue");
    assert!(body.contains("urlset"), "should have urlset element");
    assert!(body.contains("sitemaps.org"), "should reference sitemap schema");
    assert!(body.contains("<url>"), "should have at least one url entry");
    assert!(body.contains("/llms.txt"), "should include discovery pages");
    assert!(body.contains("/openapi.json"), "should include openapi");
}

#[test]
fn test_sitemap_xml_includes_profiles() {
    let client = test_client();
    register(&client, "sitemap-test-agent-xyz");
    let resp = client.get("/sitemap.xml").dispatch();
    let body = resp.into_string().unwrap();
    assert!(body.contains("sitemap-test-agent-xyz"), "newly registered profile should appear in sitemap");
}

// ===== WebFinger (RFC 7033) =====

#[test]
fn test_webfinger_existing_profile() {
    let client = test_client();
    register(&client, "webfinger-test-agent");

    let resp = client.get("/.well-known/webfinger?resource=acct:webfinger-test-agent@example.com").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let ct = resp.content_type().unwrap().to_string();
    assert!(ct.contains("jrd+json") || ct.contains("application"), "content-type should be jrd+json, got {}", ct);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["subject"], "acct:webfinger-test-agent@example.com");
    assert!(body["links"].as_array().unwrap().len() >= 2, "should have at least 2 links");
    let rels: Vec<&str> = body["links"].as_array().unwrap().iter()
        .filter_map(|l| l["rel"].as_str())
        .collect();
    assert!(rels.iter().any(|r| r.contains("profile-page")), "should have profile-page rel");
    assert!(rels.iter().any(|r| *r == "self"), "should have self rel");
}

#[test]
fn test_webfinger_not_found() {
    let client = test_client();
    let resp = client.get("/.well-known/webfinger?resource=acct:nobody-xyz-123@example.com").dispatch();
    assert_eq!(resp.status(), Status::NotFound);
}

#[test]
fn test_webfinger_missing_resource() {
    let client = test_client();
    let resp = client.get("/.well-known/webfinger").dispatch();
    assert_eq!(resp.status(), Status::BadRequest);
}

#[test]
fn test_webfinger_bad_scheme() {
    let client = test_client();
    let resp = client.get("/.well-known/webfinger?resource=https://example.com/nanook").dispatch();
    assert_eq!(resp.status(), Status::BadRequest);
}
