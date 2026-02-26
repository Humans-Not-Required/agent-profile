use rocket::http::{ContentType, Header, Status};
use rocket::local::blocking::Client;
use uuid::Uuid;

fn test_client() -> Client {
    // Relax write rate limit for integration tests (default 30/min too low for bulk tests)
    std::env::set_var("WRITE_RATE_LIMIT", "10000");
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
        .body(r#"{"theme":"doesnotexist"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

#[test]
fn test_update_profile_br2049_sandstorm_theme() {
    let client = test_client();
    let (_, reg) = register(&client, "sandstorm");
    let api_key = reg["api_key"].as_str().unwrap();
    let resp = client.patch("/api/v1/profiles/sandstorm")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"theme":"br2049-sandstorm","particle_effect":"sandstorm","particle_enabled":true}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["theme"], "br2049-sandstorm");
    assert_eq!(body["particle_effect"], "sandstorm");
    assert_eq!(body["particle_enabled"], true);
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
fn test_list_profiles_sort_popular() {
    let client = test_client();
    let (_, _reg1) = register(&client, "sort-pop-a");
    let (_, _reg2) = register(&client, "sort-pop-b");

    // Give sort-pop-b more views by visiting as human
    for _ in 0..3 {
        client.get("/sort-pop-b").header(Header::new("Accept", "text/html")).dispatch();
    }
    client.get("/sort-pop-a").header(Header::new("Accept", "text/html")).dispatch();

    let resp = client.get("/api/v1/profiles?sort=popular&limit=50").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let profiles = body["profiles"].as_array().unwrap();
    // sort-pop-b (3 views) should come before sort-pop-a (1 view)
    let usernames: Vec<&str> = profiles.iter().filter_map(|p| p["username"].as_str()).collect();
    let pos_b = usernames.iter().position(|u| *u == "sort-pop-b");
    let pos_a = usernames.iter().position(|u| *u == "sort-pop-a");
    assert!(pos_b.is_some() && pos_a.is_some(), "both profiles should be in results");
    assert!(pos_b.unwrap() < pos_a.unwrap(), "sort-pop-b (3 views) should rank before sort-pop-a (1 view)");
}

#[test]
fn test_list_profiles_sort_newest() {
    let client = test_client();
    register(&client, "sort-new-first");
    // Small delay to ensure different timestamps
    std::thread::sleep(std::time::Duration::from_millis(50));
    register(&client, "sort-new-second");

    let resp = client.get("/api/v1/profiles?sort=newest&limit=50").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let profiles = body["profiles"].as_array().unwrap();
    assert!(profiles.len() >= 2, "should have at least 2 profiles");
    // Verify created_at is in descending order
    let dates: Vec<&str> = profiles.iter().filter_map(|p| p["created_at"].as_str()).collect();
    for w in dates.windows(2) {
        assert!(w[0] >= w[1], "sort=newest should return profiles in descending created_at order, got {} before {}", w[0], w[1]);
    }
}

#[test]
fn test_list_profiles_includes_view_count() {
    let client = test_client();
    register(&client, "list-views-test");

    let resp = client.get("/api/v1/profiles?limit=50").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let profiles = body["profiles"].as_array().unwrap();
    let p = profiles.iter().find(|p| p["username"] == "list-views-test").unwrap();
    assert!(p.get("view_count").is_some(), "list response should include view_count");
    assert!(p.get("updated_at").is_some(), "list response should include updated_at");
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
    assert!(!body["next_steps"].as_array().unwrap().is_empty());

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
fn test_skill_md() {
    let client = test_client();
    let resp = client.get("/SKILL.md").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body = resp.into_string().unwrap();
    assert!(body.contains("Agent Profile Service"));
    assert!(body.contains("POST /api/v1/register"));
}

#[test]
fn test_llms_txt_aliases_skill_md() {
    let client = test_client();
    let skill_resp = client.get("/SKILL.md").dispatch();
    let llms_resp = client.get("/llms.txt").dispatch();
    assert_eq!(skill_resp.status(), Status::Ok);
    assert_eq!(llms_resp.status(), Status::Ok);
    let skill_body = skill_resp.into_string().unwrap();
    let llms_body = llms_resp.into_string().unwrap();
    assert_eq!(skill_body, llms_body, "llms.txt should serve same content as SKILL.md");
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
    // Verify it's a valid OpenAPI 3.1 spec
    assert_eq!(body["openapi"], "3.1.0");
    assert_eq!(body["info"]["version"], env!("CARGO_PKG_VERSION"));
    assert!(body["paths"].is_object());
    // Core API paths present
    let paths = body["paths"].as_object().unwrap();
    assert!(paths.contains_key("/health"));
    assert!(paths.contains_key("/register"));
    assert!(paths.contains_key("/profiles/{username}"));
    assert!(paths.contains_key("/profiles/{username}/challenge"));
    assert!(paths.contains_key("/profiles/{username}/verify"));
    assert!(paths.contains_key("/profiles/{username}/sections"));
    assert!(paths.contains_key("/profiles/{username}/score"));
    assert!(paths.contains_key("/profiles/{username}/avatar"));
    assert!(paths.contains_key("/profiles/{username}/endorsements"));
    assert!(paths.contains_key("/skills"));
    assert!(paths.contains_key("/stats"));
    // Discovery paths present
    assert!(paths.contains_key("/.well-known/webfinger"));
    assert!(paths.contains_key("/robots.txt"));
    assert!(paths.contains_key("/sitemap.xml"));
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

// ===== Atom Feed =====

#[test]
fn test_feed_xml_structure() {
    let client = test_client();
    let resp = client.get("/feed.xml").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let ct = resp.content_type().unwrap().to_string();
    assert!(ct.contains("atom") || ct.contains("xml"), "content-type should be atom+xml, got {}", ct);
    let body = resp.into_string().unwrap();
    assert!(body.contains("<feed xmlns=\"http://www.w3.org/2005/Atom\">"));
    assert!(body.contains("<title>Agent Profiles</title>"));
    assert!(body.contains("</feed>"));
}

#[test]
fn test_feed_xml_includes_profiles() {
    let client = test_client();
    register(&client, "feed-test-agent");

    // Update display name for richer feed entry
    let (_, reg) = register(&client, "feed-test-named");
    let key = reg["api_key"].as_str().unwrap();
    client.patch("/api/v1/profiles/feed-test-named")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.to_string()))
        .body(r#"{"display_name": "Feed Test Bot", "tagline": "Testing Atom feeds"}"#)
        .dispatch();

    let resp = client.get("/feed.xml").dispatch();
    let body = resp.into_string().unwrap();
    assert!(body.contains("feed-test-agent"), "feed should include registered profile");
    assert!(body.contains("Feed Test Bot"), "feed should include display name");
    assert!(body.contains("Testing Atom feeds"), "feed should include tagline in summary");
}

#[test]
fn test_feed_xml_escapes_special_chars() {
    let client = test_client();
    let (_, reg) = register(&client, "feed-escape-test");
    let key = reg["api_key"].as_str().unwrap();
    client.patch("/api/v1/profiles/feed-escape-test")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.to_string()))
        .body(r#"{"display_name": "Bot <script>", "tagline": "A & B > C"}"#)
        .dispatch();

    let resp = client.get("/feed.xml").dispatch();
    let body = resp.into_string().unwrap();
    assert!(!body.contains("<script>"), "feed must escape HTML in display name");
    assert!(body.contains("&lt;script&gt;"), "feed should XML-escape angle brackets");
    assert!(body.contains("A &amp; B &gt; C"), "feed should XML-escape ampersand and gt");
}

#[test]
fn test_feed_xml_has_autodiscovery_link_in_landing() {
    let client = test_client();
    let resp = client.get("/")
        .header(Header::new("Accept", "text/html"))
        .dispatch();
    let body = resp.into_string().unwrap();
    assert!(body.contains(r#"type="application/atom+xml""#), "landing page should have Atom feed autodiscovery link");
    assert!(body.contains("feed.xml"), "landing page should reference feed.xml");
}

// ===== Export / Import =====

#[test]
fn test_export_profile() {
    let client = test_client();
    let (_, reg) = register(&client, "export-test");
    let key = reg["api_key"].as_str().unwrap();

    // Add some data
    client.patch("/api/v1/profiles/export-test")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.to_string()))
        .body(r#"{"display_name":"Export Bot","tagline":"Testing export","bio":"I test things","theme":"midnight"}"#)
        .dispatch();
    client.post("/api/v1/profiles/export-test/skills")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.to_string()))
        .body(r#"{"skill":"rust"}"#)
        .dispatch();
    client.post("/api/v1/profiles/export-test/links")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.to_string()))
        .body(r#"{"url":"https://example.com","label":"Website","platform":"website"}"#)
        .dispatch();

    // Export
    let resp = client.get("/api/v1/profiles/export-test/export")
        .header(Header::new("X-API-Key", key.to_string()))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["format"], "agent-profile-export");
    assert_eq!(body["version"], 1);
    assert_eq!(body["profile"]["username"], "export-test");
    assert_eq!(body["profile"]["display_name"], "Export Bot");
    assert_eq!(body["profile"]["theme"], "midnight");
    assert_eq!(body["skills"].as_array().unwrap().len(), 1);
    assert_eq!(body["skills"][0], "rust");
    assert_eq!(body["links"].as_array().unwrap().len(), 1);
    assert_eq!(body["links"][0]["url"], "https://example.com");
}

#[test]
fn test_export_requires_auth() {
    let client = test_client();
    register(&client, "export-noauth");
    let resp = client.get("/api/v1/profiles/export-noauth/export")
        .header(Header::new("X-API-Key", "wrong-key"))
        .dispatch();
    assert_eq!(resp.status(), Status::Unauthorized);
}

#[test]
fn test_import_new_profile() {
    let client = test_client();
    let export_doc = serde_json::json!({
        "format": "agent-profile-export",
        "version": 1,
        "profile": {
            "username": "import-new",
            "display_name": "Imported Bot",
            "tagline": "Fresh import",
            "bio": "I was imported",
            "third_line": "",
            "theme": "ocean",
            "particle_effect": "rain",
            "particle_enabled": true,
            "particle_seasonal": false,
            "pubkey": "",
        },
        "links": [{"url": "https://github.com/test", "label": "GitHub", "platform": "github"}],
        "sections": [{"title": "About", "content": "I'm imported", "section_type": "about"}],
        "skills": ["python", "testing"],
        "crypto_addresses": [],
    });

    let resp = client.post("/api/v1/import")
        .header(ContentType::JSON)
        .body(export_doc.to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["status"], "created");
    assert_eq!(body["username"], "import-new");
    assert!(body["api_key"].as_str().unwrap().starts_with("ap_"));

    // Verify the profile was created with correct data
    let resp = client.get("/api/v1/profiles/import-new")
        .header(Header::new("Accept", "application/json"))
        .dispatch();
    let profile: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(profile["display_name"], "Imported Bot");
    assert_eq!(profile["theme"], "ocean");
    assert_eq!(profile["skills"].as_array().unwrap().len(), 2);
    assert_eq!(profile["links"].as_array().unwrap().len(), 1);
    assert_eq!(profile["sections"].as_array().unwrap().len(), 1);
}

#[test]
fn test_import_update_existing() {
    let client = test_client();
    let (_, reg) = register(&client, "import-update");
    let key = reg["api_key"].as_str().unwrap();

    let export_doc = serde_json::json!({
        "format": "agent-profile-export",
        "version": 1,
        "profile": {
            "username": "import-update",
            "display_name": "Updated Name",
            "tagline": "Updated tagline",
            "bio": "",
            "third_line": "",
            "theme": "forest",
            "particle_effect": "leaves",
            "particle_enabled": false,
            "particle_seasonal": false,
            "pubkey": "",
        },
        "links": [],
        "sections": [],
        "skills": ["updated-skill"],
        "crypto_addresses": [],
    });

    let resp = client.post("/api/v1/import")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.to_string()))
        .body(export_doc.to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["status"], "updated");

    // Verify update applied
    let resp = client.get("/api/v1/profiles/import-update")
        .header(Header::new("Accept", "application/json"))
        .dispatch();
    let profile: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(profile["display_name"], "Updated Name");
    assert_eq!(profile["theme"], "forest");
}

#[test]
fn test_import_existing_requires_auth() {
    let client = test_client();
    register(&client, "import-authcheck");

    let export_doc = serde_json::json!({
        "format": "agent-profile-export",
        "version": 1,
        "profile": { "username": "import-authcheck" },
        "links": [], "sections": [], "skills": [], "crypto_addresses": [],
    });

    let resp = client.post("/api/v1/import")
        .header(ContentType::JSON)
        .body(export_doc.to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::Unauthorized);
}

#[test]
fn test_export_import_roundtrip() {
    let client = test_client();
    let (_, reg) = register(&client, "roundtrip-test");
    let key = reg["api_key"].as_str().unwrap();

    // Set up profile
    client.patch("/api/v1/profiles/roundtrip-test")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.to_string()))
        .body(r#"{"display_name":"Roundtrip Bot","tagline":"Full circle","bio":"Testing export→import","theme":"aurora"}"#)
        .dispatch();
    client.post("/api/v1/profiles/roundtrip-test/skills")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.to_string()))
        .body(r#"{"skill":"roundtripping"}"#)
        .dispatch();

    // Export
    let resp = client.get("/api/v1/profiles/roundtrip-test/export")
        .header(Header::new("X-API-Key", key.to_string()))
        .dispatch();
    let export: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();

    // Delete original
    client.delete("/api/v1/profiles/roundtrip-test")
        .header(Header::new("X-API-Key", key.to_string()))
        .dispatch();

    // Import (creates new)
    let resp = client.post("/api/v1/import")
        .header(ContentType::JSON)
        .body(export.to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["status"], "created");
    let new_key = body["api_key"].as_str().unwrap();
    assert!(!new_key.is_empty());

    // Verify roundtripped data
    let resp = client.get("/api/v1/profiles/roundtrip-test")
        .header(Header::new("Accept", "application/json"))
        .dispatch();
    let profile: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(profile["display_name"], "Roundtrip Bot");
    assert_eq!(profile["tagline"], "Full circle");
    assert_eq!(profile["theme"], "aurora");
    assert_eq!(profile["skills"].as_array().unwrap().len(), 1);
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
    assert!(rels.contains(&"self"), "should have self rel");
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

// ===== Open Graph / Social Preview Meta Tags =====

#[test]
fn test_og_tags_profile_page() {
    let client = test_client();
    let (_, reg) = register(&client, "og-test-agent");
    let api_key = reg["api_key"].as_str().unwrap();

    // Update the profile with display name, tagline, avatar
    client.patch("/api/v1/profiles/og-test-agent")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"display_name":"OG Test Bot","tagline":"Testing social previews","avatar_url":"https://example.com/avatar.png","theme":"aurora"}"#)
        .dispatch();

    // Request the profile page as a browser (Accept: text/html)
    let resp = client.get("/og-test-agent")
        .header(Header::new("Accept", "text/html"))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body = resp.into_string().unwrap();

    // Verify OG tags are injected with profile data
    assert!(body.contains(r##"og:title" content="OG Test Bot""##), "og:title should contain display name");
    assert!(body.contains("Testing social previews"), "og:description should contain tagline");
    assert!(body.contains("https://example.com/avatar.png"), "og:image should contain avatar URL");
    assert!(body.contains("/og-test-agent"), "og:url should contain profile path");

    // Verify Twitter Card tags are injected
    assert!(body.contains(r##"twitter:card" content="summary""##), "should have twitter:card meta");
    assert!(body.contains(r##"twitter:title" content="OG Test Bot""##), "twitter:title should contain display name");

    // Verify the HTML title is updated
    assert!(body.contains("<title>OG Test Bot"), "page title should contain display name");

    // Verify theme color matches aurora accent
    assert!(body.contains(r##"theme-color" content="#7b61ff""##), "theme-color should match aurora theme");
}

#[test]
fn test_og_tags_profile_minimal() {
    // Profile with no display name or tagline should still have sensible OG tags
    let client = test_client();
    register(&client, "og-minimal");

    let resp = client.get("/og-minimal")
        .header(Header::new("Accept", "text/html"))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body = resp.into_string().unwrap();

    // Should use username as fallback title
    assert!(body.contains(r##"og:title" content="og-minimal""##), "og:title should fall back to username");
    // Should have a generic description
    assert!(body.contains("Agent profile for @og-minimal"), "og:description should have fallback text");
}

#[test]
fn test_og_tags_not_injected_for_agents() {
    // Agent requests (JSON) should not get OG-enriched HTML
    let client = test_client();
    register(&client, "og-agent-check");

    let resp = client.get("/og-agent-check")
        .header(Header::new("Accept", "application/json"))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let ct = resp.content_type().unwrap().to_string();
    assert!(ct.contains("json"), "agent request should get JSON, got {}", ct);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["username"], "og-agent-check");
}

#[test]
fn test_og_tags_html_escaping() {
    // Verify that special characters in profile fields are properly escaped in OG tags
    let client = test_client();
    let (_, reg) = register(&client, "og-escape-test");
    let api_key = reg["api_key"].as_str().unwrap();

    client.patch("/api/v1/profiles/og-escape-test")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"display_name":"Bot <script>alert(1)</script>","tagline":"O'Malley & \"Friends\""}"#)
        .dispatch();

    let resp = client.get("/og-escape-test")
        .header(Header::new("Accept", "text/html"))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body = resp.into_string().unwrap();

    // Should NOT contain raw script tags
    assert!(!body.contains("<script>alert"), "HTML in display name must be escaped");
    // Should contain escaped version
    assert!(body.contains("&lt;script&gt;"), "angle brackets should be escaped");
    assert!(body.contains("&amp;"), "ampersands should be escaped");
}

#[test]
fn test_json_ld_structured_data() {
    let client = test_client();
    let (_, reg) = register(&client, "jsonld-agent");
    let api_key = reg["api_key"].as_str().unwrap();

    // Set up profile with display name, tagline, and links
    client.patch("/api/v1/profiles/jsonld-agent")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"display_name":"LD Bot","tagline":"Structured data test"}"#)
        .dispatch();

    // Add a link
    client.post("/api/v1/profiles/jsonld-agent/links")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"url":"https://github.com/ld-bot","label":"GitHub","platform":"github"}"#)
        .dispatch();

    // Add a skill
    client.post("/api/v1/profiles/jsonld-agent/skills")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"skill":"rust"}"#)
        .dispatch();

    let resp = client.get("/jsonld-agent")
        .header(Header::new("Accept", "text/html"))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body = resp.into_string().unwrap();

    // Verify JSON-LD script tag present
    assert!(body.contains(r#"<script type="application/ld+json">"#), "should have JSON-LD script tag");
    assert!(body.contains(r#""@context": "https://schema.org""#), "should have Schema.org context");
    assert!(body.contains(r#""@type": "Person""#), "should have Person type");
    assert!(body.contains("LD Bot"), "JSON-LD should contain display name");
    assert!(body.contains("Structured data test"), "JSON-LD should contain description");
    assert!(body.contains("https://github.com/ld-bot"), "JSON-LD sameAs should contain link");
    assert!(body.contains("knowsAbout"), "JSON-LD should contain knowsAbout with skills");
    assert!(body.contains("rust"), "JSON-LD knowsAbout should contain the skill");
}

#[test]
fn test_rel_me_links() {
    let client = test_client();
    let (_, reg) = register(&client, "relme-agent");
    let api_key = reg["api_key"].as_str().unwrap();

    // Add links
    client.post("/api/v1/profiles/relme-agent/links")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", api_key.to_string()))
        .body(r#"{"url":"https://mastodon.social/@relme","label":"Mastodon","platform":"custom"}"#)
        .dispatch();

    let resp = client.get("/relme-agent")
        .header(Header::new("Accept", "text/html"))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body = resp.into_string().unwrap();

    // Verify rel=me link tag for IndieWeb/Mastodon verification
    assert!(body.contains(r##"<link rel="me" href="https://mastodon.social/@relme" />"##),
        "should have rel=me link for Mastodon verification");
}

// ===== Profile View Counter =====

#[test]
fn test_view_count_increments_on_human_visit() {
    let client = test_client();
    register(&client, "views-agent");

    // Check initial view count via JSON (agent request)
    let resp = client.get("/api/v1/profiles/views-agent").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["view_count"], 0, "view count should start at 0");

    // Visit as human (text/html) — should increment view count
    client.get("/views-agent")
        .header(Header::new("Accept", "text/html"))
        .dispatch();
    client.get("/views-agent")
        .header(Header::new("Accept", "text/html"))
        .dispatch();
    client.get("/views-agent")
        .header(Header::new("Accept", "text/html"))
        .dispatch();

    // Check view count again
    let resp = client.get("/api/v1/profiles/views-agent").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["view_count"], 3, "view count should be 3 after 3 human visits");
}

#[test]
fn test_view_count_not_incremented_by_agent() {
    let client = test_client();
    register(&client, "views-no-agent");

    // Visit as agent (JSON) — should NOT increment
    client.get("/views-no-agent")
        .header(Header::new("Accept", "application/json"))
        .dispatch();
    client.get("/views-no-agent")
        .header(Header::new("Accept", "application/json"))
        .dispatch();

    // Check view count
    let resp = client.get("/api/v1/profiles/views-no-agent").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["view_count"], 0, "agent requests should not increment view count");
}

#[test]
fn test_view_count_in_profile_json() {
    let client = test_client();
    register(&client, "views-json");

    // The view_count field should exist in the profile JSON
    let resp = client.get("/api/v1/profiles/views-json").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert!(body.get("view_count").is_some(), "profile JSON should include view_count field");
    assert!(body["view_count"].is_number(), "view_count should be a number");
}

#[test]
fn test_canonical_link_tag() {
    let client = test_client();
    register(&client, "canonical-test");

    let resp = client.get("/canonical-test")
        .header(Header::new("Accept", "text/html"))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body = resp.into_string().unwrap();

    assert!(body.contains(r##"<link rel="canonical" href="/canonical-test" />"##),
        "profile page should have canonical link tag");
}

#[test]
fn test_landing_page_og_tags() {
    let client = test_client();

    let resp = client.get("/")
        .header(Header::new("Accept", "text/html"))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body = resp.into_string().unwrap();

    // Landing page should have OG tags
    assert!(body.contains(r##"og:title" content="Pinche.rs"##), "landing page should have og:title");
    assert!(body.contains(r##"og:type" content="website""##), "landing page should have og:type");
    assert!(body.contains(r##"twitter:card" content="summary""##), "landing page should have twitter:card");
}

// ===== Security Headers =====

#[test]
fn test_security_headers_on_api() {
    let client = test_client();
    let resp = client.get("/api/v1/health").dispatch();
    assert_eq!(resp.status(), Status::Ok);

    // Check all security headers present
    let headers = resp.headers();
    assert!(headers.get_one("Content-Security-Policy").is_some(),
        "should have Content-Security-Policy header");
    assert_eq!(headers.get_one("X-Content-Type-Options"), Some("nosniff"),
        "should have X-Content-Type-Options: nosniff");
    assert_eq!(headers.get_one("X-Frame-Options"), Some("DENY"),
        "should have X-Frame-Options: DENY");
    assert!(headers.get_one("Referrer-Policy").is_some(),
        "should have Referrer-Policy header");
    assert!(headers.get_one("Permissions-Policy").is_some(),
        "should have Permissions-Policy header");
}

#[test]
fn test_csp_allows_inline_styles() {
    let client = test_client();
    let resp = client.get("/api/v1/health").dispatch();
    let csp = resp.headers().get_one("Content-Security-Policy").unwrap();

    assert!(csp.contains("style-src 'self' 'unsafe-inline'"),
        "CSP should allow inline styles for React: {}", csp);
    assert!(csp.contains("img-src 'self' data: https:"),
        "CSP should allow data: and https: images: {}", csp);
    assert!(csp.contains("frame-src 'none'"),
        "CSP should block iframes: {}", csp);
    assert!(csp.contains("object-src 'none'"),
        "CSP should block objects: {}", csp);
}

#[test]
fn test_security_headers_on_profile_page() {
    let client = test_client();
    register(&client, "sec-headers-profile");

    let resp = client.get("/sec-headers-profile")
        .header(Header::new("Accept", "text/html"))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);

    assert!(resp.headers().get_one("Content-Security-Policy").is_some(),
        "profile page should have CSP");
    assert_eq!(resp.headers().get_one("X-Frame-Options"), Some("DENY"),
        "profile page should have X-Frame-Options");
}

// ===== Cache-Control Headers =====

#[test]
fn test_cache_control_health_no_cache() {
    let client = test_client();
    let resp = client.get("/api/v1/health").dispatch();
    assert_eq!(resp.headers().get_one("Cache-Control"), Some("no-cache"),
        "health endpoint should be no-cache");
}

#[test]
fn test_cache_control_api_short() {
    let client = test_client();
    register(&client, "cache-api-test");

    let resp = client.get("/api/v1/profiles/cache-api-test").dispatch();
    let cc = resp.headers().get_one("Cache-Control").unwrap_or("");
    assert!(cc.contains("max-age=60"),
        "API endpoints should have short cache: {}", cc);
}

#[test]
fn test_cache_control_skill_md() {
    let client = test_client();
    let resp = client.get("/SKILL.md").dispatch();
    let cc = resp.headers().get_one("Cache-Control").unwrap_or("");
    assert!(cc.contains("max-age=3600"),
        "SKILL.md should have 1-hour cache: {}", cc);
}

#[test]
fn test_cache_control_robots_txt() {
    let client = test_client();
    let resp = client.get("/robots.txt").dispatch();
    let cc = resp.headers().get_one("Cache-Control").unwrap_or("");
    assert!(cc.contains("max-age=86400"),
        "robots.txt should have 1-day cache: {}", cc);
}

#[test]
fn test_cache_control_profile_page() {
    let client = test_client();
    register(&client, "cache-page-test");

    let resp = client.get("/cache-page-test")
        .header(Header::new("Accept", "text/html"))
        .dispatch();
    let cc = resp.headers().get_one("Cache-Control").unwrap_or("");
    assert!(cc.contains("max-age=300"),
        "profile pages should have 5-minute cache: {}", cc);
}

#[test]
fn test_cache_control_feed_xml() {
    let client = test_client();
    let resp = client.get("/feed.xml").dispatch();
    let cc = resp.headers().get_one("Cache-Control").unwrap_or("");
    assert!(cc.contains("max-age=900"),
        "feed.xml should have 15-minute cache: {}", cc);
}

// ===== 404 Content Negotiation =====

#[test]
fn test_404_html_for_browsers() {
    let client = test_client();

    let resp = client.get("/nonexistent-agent-xyz")
        .header(Header::new("Accept", "text/html,application/xhtml+xml"))
        .header(Header::new("User-Agent", "Mozilla/5.0"))
        .dispatch();
    assert_eq!(resp.status(), Status::NotFound);

    let body = resp.into_string().unwrap();
    assert!(body.contains("<!DOCTYPE html>"), "browser 404 should be HTML");
    assert!(body.contains("404"), "should show 404");
    assert!(body.contains("Browse all agents"), "should link back to landing page");
}

#[test]
fn test_404_json_for_agents() {
    let client = test_client();

    let resp = client.get("/api/v1/profiles/nonexistent-agent-xyz")
        .header(Header::new("Accept", "application/json"))
        .dispatch();
    assert_eq!(resp.status(), Status::NotFound);

    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert!(body["error"].as_str().unwrap().contains("not found") || body["error"].as_str().unwrap().contains("Not found"),
        "should have error message: {}", body["error"]);
}

#[test]
fn test_404_json_for_curl() {
    let client = test_client();

    let resp = client.get("/nonexistent-agent-xyz")
        .header(Header::new("Accept", "text/html"))
        .header(Header::new("User-Agent", "curl/8.0"))
        .dispatch();
    assert_eq!(resp.status(), Status::NotFound);

    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert!(body["error"].as_str().unwrap().contains("not found") || body["error"].as_str().unwrap().contains("Not found"),
        "should have error message: {}", body["error"]);
}

// ===== Input Validation Limits =====

#[test]
fn test_display_name_max_length() {
    let client = test_client();
    let (_, reg) = register(&client, "dn-limit-test");
    let key = reg["api_key"].as_str().unwrap().to_string();

    // 101 chars should fail
    let long_name = "A".repeat(101);
    let resp = client.patch("/api/v1/profiles/dn-limit-test")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(serde_json::json!({"display_name": long_name}).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);

    // 100 chars should work
    let ok_name = "B".repeat(100);
    let resp = client.patch("/api/v1/profiles/dn-limit-test")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(serde_json::json!({"display_name": ok_name}).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
}

#[test]
fn test_link_url_max_length() {
    let client = test_client();
    let (_, reg) = register(&client, "link-limit-test");
    let key = reg["api_key"].as_str().unwrap().to_string();

    // URL over 2000 chars should fail
    let long_url = format!("https://example.com/{}", "a".repeat(2000));
    let resp = client.post("/api/v1/profiles/link-limit-test/links")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(serde_json::json!({"url": long_url, "label": "test"}).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert!(body["error"].as_str().unwrap().contains("max"), "error should mention max: {}", body);
}

#[test]
fn test_section_title_max_length() {
    let client = test_client();
    let (_, reg) = register(&client, "sec-title-limit");
    let key = reg["api_key"].as_str().unwrap().to_string();

    // Title over 200 chars should fail
    let long_title = "T".repeat(201);
    let resp = client.post("/api/v1/profiles/sec-title-limit/sections")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(serde_json::json!({"title": long_title, "content": "ok"}).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

#[test]
fn test_sub_resource_count_limit_links() {
    let client = test_client();
    let (_, reg) = register(&client, "link-count-limit");
    let key = reg["api_key"].as_str().unwrap().to_string();

    // Add 20 links (the maximum)
    for i in 0..20 {
        let resp = client.post("/api/v1/profiles/link-count-limit/links")
            .header(ContentType::JSON)
            .header(Header::new("X-API-Key", key.clone()))
            .body(serde_json::json!({"url": format!("https://example.com/{}", i), "label": format!("Link {}", i)}).to_string())
            .dispatch();
        assert_eq!(resp.status(), Status::Created, "link {} should succeed", i);
    }

    // 21st should fail
    let resp = client.post("/api/v1/profiles/link-count-limit/links")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(serde_json::json!({"url": "https://example.com/21", "label": "Too many"}).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert!(body["error"].as_str().unwrap().contains("Maximum"), "error should mention limit: {}", body);
}

#[test]
fn test_sub_resource_count_limit_skills() {
    let client = test_client();
    let (_, reg) = register(&client, "skill-count-limit");
    let key = reg["api_key"].as_str().unwrap().to_string();

    // Add 50 skills (the maximum)
    for i in 0..50 {
        let resp = client.post("/api/v1/profiles/skill-count-limit/skills")
            .header(ContentType::JSON)
            .header(Header::new("X-API-Key", key.clone()))
            .body(serde_json::json!({"skill": format!("skill-{}", i)}).to_string())
            .dispatch();
        assert_eq!(resp.status(), Status::Created, "skill {} should succeed", i);
    }

    // 51st should fail
    let resp = client.post("/api/v1/profiles/skill-count-limit/skills")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(serde_json::json!({"skill": "too-many"}).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

#[test]
fn test_write_rate_limit() {
    // Use a low write rate limit for this specific test
    std::env::set_var("WRITE_RATE_LIMIT", "3");
    let db_path = format!("/tmp/agent_profile_test_{}.db", uuid::Uuid::new_v4());
    let rocket = agent_profile::create_rocket(&db_path);
    let client = Client::tracked(rocket).expect("valid rocket instance");

    let (_, reg) = register(&client, "write-rl-test");
    let key = reg["api_key"].as_str().unwrap().to_string();

    // First 3 writes should succeed (within limit)
    for i in 0..3 {
        let resp = client.post("/api/v1/profiles/write-rl-test/skills")
            .header(ContentType::JSON)
            .header(Header::new("X-API-Key", key.clone()))
            .body(serde_json::json!({"skill": format!("rl-skill-{}", i)}).to_string())
            .dispatch();
        assert_eq!(resp.status(), Status::Created, "write {} should succeed", i);
    }

    // 4th should hit rate limit
    let resp = client.post("/api/v1/profiles/write-rl-test/skills")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(serde_json::json!({"skill": "rl-skill-blocked"}).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::TooManyRequests,
        "4th write should be rate limited");

    // Reset for other tests
    std::env::set_var("WRITE_RATE_LIMIT", "10000");
}

// ===== Pagination total count =====

#[test]
fn test_list_profiles_pagination_total() {
    let client = test_client();
    // Register 5 profiles
    for i in 0..5 {
        register(&client, &format!("page-total-{}", i));
    }

    // Fetch with limit=2 — total should be 5, has_more should be true
    let resp = client.get("/api/v1/profiles?limit=2&offset=0").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["profiles"].as_array().unwrap().len(), 2, "page should have 2 profiles");
    assert_eq!(body["total"].as_i64().unwrap(), 5, "total should be 5 regardless of limit");
    assert!(body["has_more"].as_bool().unwrap(), "has_more should be true when more pages exist");
    assert_eq!(body["limit"].as_i64().unwrap(), 2);
    assert_eq!(body["offset"].as_i64().unwrap(), 0);

    // Fetch last page
    let resp = client.get("/api/v1/profiles?limit=2&offset=4").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["profiles"].as_array().unwrap().len(), 1, "last page should have 1 profile");
    assert_eq!(body["total"].as_i64().unwrap(), 5, "total should still be 5");
    assert!(!body["has_more"].as_bool().unwrap(), "has_more should be false on last page");
}

#[test]
fn test_list_profiles_pagination_total_with_filter() {
    let client = test_client();
    // Register some profiles, update one to have a specific theme
    let (_, reg) = register(&client, "filter-total-a");
    let key = reg["api_key"].as_str().unwrap().to_string();
    client.patch("/api/v1/profiles/filter-total-a")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(r#"{"theme":"ocean"}"#)
        .dispatch();
    register(&client, "filter-total-b"); // default theme

    // Filter by theme=ocean — total should be 1
    let resp = client.get("/api/v1/profiles?theme=ocean&limit=50").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["total"].as_i64().unwrap(), 1, "total should reflect filter");
    assert!(!body["has_more"].as_bool().unwrap());
}

// ===== Health check with DB =====

#[test]
fn test_health_includes_ok_status() {
    let client = test_client();
    let resp = client.get("/api/v1/health").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["status"], "ok", "health should verify DB connectivity");
}

// ===== PATCH links =====

#[test]
fn test_update_link() {
    let client = test_client();
    let (_, reg) = register(&client, "link-patch-test");
    let key = reg["api_key"].as_str().unwrap().to_string();

    // Add a link
    let resp = client.post("/api/v1/profiles/link-patch-test/links")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(r#"{"url":"https://github.com/test","label":"GitHub","platform":"github"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Created);
    let link: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let link_id = link["id"].as_str().unwrap().to_string();

    // Update url and label
    let url = format!("/api/v1/profiles/link-patch-test/links/{}", link_id);
    let resp = client.patch(&url)
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(r#"{"url":"https://gitlab.com/test","label":"GitLab","platform":"website"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let updated: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(updated["url"], "https://gitlab.com/test");
    assert_eq!(updated["label"], "GitLab");
    assert_eq!(updated["platform"], "website");

    // Verify via full profile fetch
    let resp = client.get("/api/v1/profiles/link-patch-test")
        .header(Header::new("Accept", "application/json"))
        .dispatch();
    let profile: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let links = profile["links"].as_array().unwrap();
    assert_eq!(links[0]["url"], "https://gitlab.com/test");
}

#[test]
fn test_update_link_partial() {
    let client = test_client();
    let (_, reg) = register(&client, "link-partial-test");
    let key = reg["api_key"].as_str().unwrap().to_string();

    // Add a link
    let resp = client.post("/api/v1/profiles/link-partial-test/links")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(r#"{"url":"https://example.com","label":"My Site","platform":"website"}"#)
        .dispatch();
    let link: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let link_id = link["id"].as_str().unwrap().to_string();

    // Update only display_order — url and label should remain unchanged
    let url = format!("/api/v1/profiles/link-partial-test/links/{}", link_id);
    let resp = client.patch(&url)
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(r#"{"display_order": 5}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let updated: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(updated["url"], "https://example.com", "url should be unchanged");
    assert_eq!(updated["label"], "My Site", "label should be unchanged");
    assert_eq!(updated["display_order"], 5);
}

#[test]
fn test_update_link_not_found() {
    let client = test_client();
    let (_, reg) = register(&client, "link-notfound-test");
    let key = reg["api_key"].as_str().unwrap().to_string();

    let resp = client.patch("/api/v1/profiles/link-notfound-test/links/nonexistent-id")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(r#"{"label":"Updated"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::NotFound);
}

#[test]
fn test_update_link_no_fields() {
    let client = test_client();
    let (_, reg) = register(&client, "link-nofields-test");
    let key = reg["api_key"].as_str().unwrap().to_string();

    // Add a link
    let resp = client.post("/api/v1/profiles/link-nofields-test/links")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(r#"{"url":"https://example.com","label":"Test"}"#)
        .dispatch();
    let link: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let link_id = link["id"].as_str().unwrap().to_string();

    // Send empty update
    let url = format!("/api/v1/profiles/link-nofields-test/links/{}", link_id);
    let resp = client.patch(&url)
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(r#"{}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

#[test]
fn test_update_link_invalid_platform() {
    let client = test_client();
    let (_, reg) = register(&client, "link-badplatform");
    let key = reg["api_key"].as_str().unwrap().to_string();

    let resp = client.post("/api/v1/profiles/link-badplatform/links")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(r#"{"url":"https://example.com","label":"Test"}"#)
        .dispatch();
    let link: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let link_id = link["id"].as_str().unwrap().to_string();

    let url = format!("/api/v1/profiles/link-badplatform/links/{}", link_id);
    let resp = client.patch(&url)
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(r#"{"platform":"invalid-platform"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

#[test]
fn test_update_link_wrong_key() {
    let client = test_client();
    let (_, reg1) = register(&client, "link-wrongkey-owner");
    let (_, reg2) = register(&client, "link-wrongkey-other");
    let key1 = reg1["api_key"].as_str().unwrap().to_string();
    let key2 = reg2["api_key"].as_str().unwrap().to_string();

    // Add link under owner
    let resp = client.post("/api/v1/profiles/link-wrongkey-owner/links")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key1.clone()))
        .body(r#"{"url":"https://example.com","label":"Test"}"#)
        .dispatch();
    let link: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let link_id = link["id"].as_str().unwrap().to_string();

    // Try to update with wrong key
    let url = format!("/api/v1/profiles/link-wrongkey-owner/links/{}", link_id);
    let resp = client.patch(&url)
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key2.clone()))
        .body(r#"{"label":"Hacked"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Unauthorized);
}

// ===== GET /api/v1/me (whoami) =====

#[test]
fn test_whoami_valid_key() {
    let client = test_client();
    let (_, reg) = register(&client, "whoami-test");
    let key = reg["api_key"].as_str().unwrap().to_string();

    let resp = client.get("/api/v1/me")
        .header(Header::new("X-API-Key", key))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["username"], "whoami-test");
    assert!(body["profile_url"].as_str().unwrap().contains("whoami-test"));
    assert!(body["json_url"].as_str().unwrap().contains("whoami-test"));
    assert!(body.get("created_at").is_some());
    assert!(body.get("profile_score").is_some());
}

#[test]
fn test_whoami_invalid_key() {
    let client = test_client();
    let resp = client.get("/api/v1/me")
        .header(Header::new("X-API-Key", "ap_invalid_key_12345"))
        .dispatch();
    assert_eq!(resp.status(), Status::Unauthorized);
}

#[test]
fn test_whoami_no_key() {
    let client = test_client();
    let resp = client.get("/api/v1/me").dispatch();
    assert_eq!(resp.status(), Status::Unauthorized);
}

#[test]
fn test_whoami_after_reissue() {
    let client = test_client();
    let (_, reg) = register(&client, "whoami-reissue");
    let old_key = reg["api_key"].as_str().unwrap().to_string();

    // Reissue key
    let resp = client.post("/api/v1/profiles/whoami-reissue/reissue-key")
        .header(Header::new("X-API-Key", old_key.clone()))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let new_body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let new_key = new_body["api_key"].as_str().unwrap().to_string();

    // Old key should fail
    let resp = client.get("/api/v1/me")
        .header(Header::new("X-API-Key", old_key))
        .dispatch();
    assert_eq!(resp.status(), Status::Unauthorized);

    // New key should work
    let resp = client.get("/api/v1/me")
        .header(Header::new("X-API-Key", new_key))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["username"], "whoami-reissue");
}

// ===== PATCH crypto addresses =====

#[test]
fn test_update_address() {
    let client = test_client();
    let (_, reg) = register(&client, "addr-patch-test");
    let key = reg["api_key"].as_str().unwrap().to_string();

    // Add an address
    let resp = client.post("/api/v1/profiles/addr-patch-test/addresses")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(r#"{"network":"bitcoin","address":"bc1qtest123","label":"Tips"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Created);
    let addr: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let addr_id = addr["id"].as_str().unwrap().to_string();

    // Update network and label
    let url = format!("/api/v1/profiles/addr-patch-test/addresses/{}", addr_id);
    let resp = client.patch(&url)
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(r#"{"network":"ethereum","label":"ETH Tips"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let updated: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(updated["network"], "ethereum");
    assert_eq!(updated["label"], "ETH Tips");
    assert_eq!(updated["address"], "bc1qtest123", "address should be unchanged");
}

#[test]
fn test_update_address_invalid_network() {
    let client = test_client();
    let (_, reg) = register(&client, "addr-badnet");
    let key = reg["api_key"].as_str().unwrap().to_string();

    let resp = client.post("/api/v1/profiles/addr-badnet/addresses")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(r#"{"network":"bitcoin","address":"bc1qtest"}"#)
        .dispatch();
    let addr: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let addr_id = addr["id"].as_str().unwrap().to_string();

    let url = format!("/api/v1/profiles/addr-badnet/addresses/{}", addr_id);
    let resp = client.patch(&url)
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(r#"{"network":"fakecoin"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

#[test]
fn test_update_address_not_found() {
    let client = test_client();
    let (_, reg) = register(&client, "addr-notfound");
    let key = reg["api_key"].as_str().unwrap().to_string();

    let resp = client.patch("/api/v1/profiles/addr-notfound/addresses/nonexistent-id")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", key.clone()))
        .body(r#"{"label":"test"}"#)
        .dispatch();
    assert_eq!(resp.status(), Status::NotFound);
}

// ===== Import bug fix: skills/sections without links =====

#[test]
fn test_import_skills_without_links() {
    let client = test_client();

    // Import a profile that has skills and sections but NO links
    let resp = client.post("/api/v1/import")
        .header(ContentType::JSON)
        .body(serde_json::json!({
            "format": "agent-profile-export",
            "version": 1,
            "profile": {
                "username": "import-no-links",
                "display_name": "No Links Agent",
                "tagline": "Has skills but no links",
                "bio": "Testing import without links array",
                "theme": "dark"
            },
            "skills": ["rust", "python", "testing"],
            "sections": [
                {"title": "About", "content": "This is a test", "section_type": "custom"}
            ]
        }).to_string())
        .dispatch();
    assert!(resp.status() == Status::Ok || resp.status() == Status::Created, "import should succeed, got {:?}", resp.status());
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["username"], "import-no-links");

    // Verify skills were actually imported
    let resp = client.get("/api/v1/profiles/import-no-links")
        .header(Header::new("Accept", "application/json"))
        .dispatch();
    let profile: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let skills = profile["skills"].as_array().unwrap();
    assert_eq!(skills.len(), 3, "should have 3 skills even without links");

    let sections = profile["sections"].as_array().unwrap();
    assert_eq!(sections.len(), 1, "should have 1 section even without links");
}

#[test]
fn test_import_invalid_theme() {
    let client = test_client();

    let resp = client.post("/api/v1/import")
        .header(ContentType::JSON)
        .body(serde_json::json!({
            "format": "agent-profile-export",
            "version": 1,
            "profile": {
                "username": "import-bad-theme",
                "display_name": "Bad Theme",
                "theme": "nonexistent-theme"
            }
        }).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

// ===== Similar Profiles (Discovery) =====

/// Helper: add skills to a profile
fn add_skills(client: &Client, username: &str, api_key: &str, skills: &[&str]) {
    for skill in skills {
        client.post(format!("/api/v1/profiles/{}/skills", username))
            .header(ContentType::JSON)
            .header(Header::new("X-API-Key", api_key.to_string()))
            .body(serde_json::json!({"skill": skill}).to_string())
            .dispatch();
    }
}

#[test]
fn test_similar_not_found() {
    let client = test_client();
    let resp = client.get("/api/v1/profiles/nonexistent-agent/similar").dispatch();
    assert_eq!(resp.status(), Status::NotFound);
}

#[test]
fn test_similar_no_skills() {
    let client = test_client();
    let (_, _body) = register(&client, "sim-empty");

    let resp = client.get("/api/v1/profiles/sim-empty/similar").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["similar"].as_array().unwrap().len(), 0);
    assert_eq!(body["total"], 0);
}

#[test]
fn test_similar_no_overlap() {
    let client = test_client();
    let (_, a) = register(&client, "sim-a");
    let (_, b) = register(&client, "sim-b");
    add_skills(&client, "sim-a", a["api_key"].as_str().unwrap(), &["rust", "nats"]);
    add_skills(&client, "sim-b", b["api_key"].as_str().unwrap(), &["python", "django"]);

    let resp = client.get("/api/v1/profiles/sim-a/similar").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["similar"].as_array().unwrap().len(), 0);
}

#[test]
fn test_similar_basic_overlap() {
    let client = test_client();
    let (_, a) = register(&client, "sim-alice");
    let (_, b) = register(&client, "sim-bob");
    add_skills(&client, "sim-alice", a["api_key"].as_str().unwrap(), &["rust", "nats", "docker"]);
    add_skills(&client, "sim-bob", b["api_key"].as_str().unwrap(), &["rust", "docker", "python"]);

    let resp = client.get("/api/v1/profiles/sim-alice/similar").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let similar = body["similar"].as_array().unwrap();
    assert_eq!(similar.len(), 1);
    assert_eq!(similar[0]["username"], "sim-bob");
    assert_eq!(similar[0]["shared_count"], 2); // rust + docker
    assert!(similar[0]["shared_skills"].as_str().unwrap().contains("rust"));
    assert!(similar[0]["shared_skills"].as_str().unwrap().contains("docker"));
}

#[test]
fn test_similar_ranking() {
    let client = test_client();
    let (_, a) = register(&client, "sim-rank-a");
    let (_, b) = register(&client, "sim-rank-b");
    let (_, c) = register(&client, "sim-rank-c");

    // A has: rust, nats, docker, python
    add_skills(&client, "sim-rank-a", a["api_key"].as_str().unwrap(), &["rust", "nats", "docker", "python"]);
    // B has 3 overlap: rust, nats, docker
    add_skills(&client, "sim-rank-b", b["api_key"].as_str().unwrap(), &["rust", "nats", "docker"]);
    // C has 1 overlap: rust
    add_skills(&client, "sim-rank-c", c["api_key"].as_str().unwrap(), &["rust", "java"]);

    let resp = client.get("/api/v1/profiles/sim-rank-a/similar").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let similar = body["similar"].as_array().unwrap();
    assert_eq!(similar.len(), 2);
    // B should rank first (3 shared) > C (1 shared)
    assert_eq!(similar[0]["username"], "sim-rank-b");
    assert_eq!(similar[0]["shared_count"], 3);
    assert_eq!(similar[1]["username"], "sim-rank-c");
    assert_eq!(similar[1]["shared_count"], 1);
}

#[test]
fn test_similar_excludes_self() {
    let client = test_client();
    let (_, a) = register(&client, "sim-self");
    add_skills(&client, "sim-self", a["api_key"].as_str().unwrap(), &["rust"]);

    let resp = client.get("/api/v1/profiles/sim-self/similar").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let similar = body["similar"].as_array().unwrap();
    // Should not include self
    for s in similar {
        assert_ne!(s["username"], "sim-self");
    }
}

#[test]
fn test_similar_limit() {
    let client = test_client();
    let (_, a) = register(&client, "sim-lim-src");
    let (_, b) = register(&client, "sim-lim-1");
    let (_, c) = register(&client, "sim-lim-2");
    let (_, d) = register(&client, "sim-lim-3");

    // All share "rust" skill
    add_skills(&client, "sim-lim-src", a["api_key"].as_str().unwrap(), &["rust"]);
    add_skills(&client, "sim-lim-1", b["api_key"].as_str().unwrap(), &["rust"]);
    add_skills(&client, "sim-lim-2", c["api_key"].as_str().unwrap(), &["rust"]);
    add_skills(&client, "sim-lim-3", d["api_key"].as_str().unwrap(), &["rust"]);

    // Without limit: should return all 3 similar
    let resp = client.get("/api/v1/profiles/sim-lim-src/similar").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["similar"].as_array().unwrap().len(), 3);

    // With limit=1: should return only 1
    let resp = client.get("/api/v1/profiles/sim-lim-src/similar?limit=1").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["similar"].as_array().unwrap().len(), 1);
}

#[test]
fn test_similar_case_insensitive() {
    let client = test_client();
    let (_, a) = register(&client, "sim-case-a");
    let (_, b) = register(&client, "sim-case-b");
    add_skills(&client, "sim-case-a", a["api_key"].as_str().unwrap(), &["Rust"]);
    add_skills(&client, "sim-case-b", b["api_key"].as_str().unwrap(), &["rust"]);

    let resp = client.get("/api/v1/profiles/sim-case-a/similar").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let similar = body["similar"].as_array().unwrap();
    assert_eq!(similar.len(), 1);
    assert_eq!(similar[0]["username"], "sim-case-b");
}

#[test]
fn test_similar_returns_profile_fields() {
    let client = test_client();
    let (_, a) = register(&client, "sim-fields-a");
    let (_, b) = register(&client, "sim-fields-b");

    // Update B with display info
    client.patch("/api/v1/profiles/sim-fields-b")
        .header(ContentType::JSON)
        .header(Header::new("X-API-Key", b["api_key"].as_str().unwrap().to_string()))
        .body(serde_json::json!({
            "display_name": "Field Bot",
            "tagline": "Testing fields",
            "theme": "ocean"
        }).to_string())
        .dispatch();

    add_skills(&client, "sim-fields-a", a["api_key"].as_str().unwrap(), &["nats"]);
    add_skills(&client, "sim-fields-b", b["api_key"].as_str().unwrap(), &["nats"]);

    let resp = client.get("/api/v1/profiles/sim-fields-a/similar").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let similar = body["similar"].as_array().unwrap();
    assert_eq!(similar.len(), 1);
    let s = &similar[0];
    assert_eq!(s["username"], "sim-fields-b");
    assert_eq!(s["display_name"], "Field Bot");
    assert_eq!(s["tagline"], "Testing fields");
    assert_eq!(s["theme"], "ocean");
    assert!(s["profile_score"].is_number());
    assert!(s["view_count"].is_number());
    assert!(s["shared_count"].is_number());
    assert!(s["shared_skills"].is_string());
}
