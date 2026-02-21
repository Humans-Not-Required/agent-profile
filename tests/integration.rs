use rocket::http::{ContentType, Header, Status};
use rocket::local::blocking::Client;
use uuid::Uuid;

fn test_client() -> Client {
    let db_path = format!("/tmp/agent_profile_test_{}.db", Uuid::new_v4());
    let rocket = agent_profile::create_rocket(&db_path);
    Client::tracked(rocket).expect("valid rocket instance")
}

#[test]
fn test_health() {
    let client = test_client();
    let resp = client.get("/api/v1/health").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["service"], "agent-profile");
}

fn create_test_profile(client: &Client, slug: &str) -> (Status, serde_json::Value) {
    let resp = client.post("/api/v1/profiles")
        .header(ContentType::JSON)
        .body(serde_json::json!({
            "slug": slug,
            "display_name": format!("Test Agent {}", slug),
            "bio": "Test bio for agent",
        }).to_string())
        .dispatch();
    let status = resp.status();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    (status, body)
}

#[test]
fn test_create_profile() {
    let client = test_client();
    let (status, body) = create_test_profile(&client, "nanook");
    assert_eq!(status, Status::Created);
    assert_eq!(body["slug"], "nanook");
    assert!(body["manage_token"].as_str().is_some());
    assert!(!body["manage_token"].as_str().unwrap().is_empty());
    assert_eq!(body["profile_url"], "/agents/nanook");
    assert_eq!(body["json_url"], "/api/v1/profiles/nanook");
}

#[test]
fn test_create_profile_slug_normalized() {
    let client = test_client();
    let (status, body) = create_test_profile(&client, "JIGGAI");
    assert_eq!(status, Status::Created);
    assert_eq!(body["slug"], "jiggai");
}

#[test]
fn test_create_profile_duplicate_slug() {
    let client = test_client();
    let _ = create_test_profile(&client, "duplicate");
    let (status, _) = create_test_profile(&client, "duplicate");
    assert_eq!(status, Status::Conflict);
}

#[test]
fn test_create_profile_invalid_slug_too_short() {
    let client = test_client();
    let resp = client.post("/api/v1/profiles")
        .header(ContentType::JSON)
        .body(serde_json::json!({"slug": "ab", "display_name": "Short"}).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

#[test]
fn test_create_profile_reserved_slug() {
    let client = test_client();
    let resp = client.post("/api/v1/profiles")
        .header(ContentType::JSON)
        .body(serde_json::json!({"slug": "api", "display_name": "API"}).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

#[test]
fn test_create_profile_slug_starts_with_hyphen() {
    let client = test_client();
    let resp = client.post("/api/v1/profiles")
        .header(ContentType::JSON)
        .body(serde_json::json!({"slug": "-bad", "display_name": "Bad"}).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

#[test]
fn test_get_profile() {
    let client = test_client();
    let _ = create_test_profile(&client, "getme");

    let resp = client.get("/api/v1/profiles/getme").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["slug"], "getme");
    assert_eq!(body["display_name"], "Test Agent getme");
    assert!(body["crypto_addresses"].as_array().unwrap().is_empty());
    assert!(body["links"].as_array().unwrap().is_empty());
    assert!(body["skills"].as_array().unwrap().is_empty());
}

#[test]
fn test_get_profile_not_found() {
    let client = test_client();
    let resp = client.get("/api/v1/profiles/nonexistent").dispatch();
    assert_eq!(resp.status(), Status::NotFound);
}

#[test]
fn test_get_profile_case_insensitive() {
    let client = test_client();
    let _ = create_test_profile(&client, "casetest");
    let resp = client.get("/api/v1/profiles/CASETEST").dispatch();
    assert_eq!(resp.status(), Status::Ok);
}

#[test]
fn test_update_profile() {
    let client = test_client();
    let (_, create_body) = create_test_profile(&client, "updateme");
    let token = create_body["manage_token"].as_str().unwrap();

    let resp = client.patch("/api/v1/profiles/updateme")
        .header(ContentType::JSON)
        .header(Header::new("X-Manage-Token", token.to_string()))
        .body(serde_json::json!({
            "display_name": "Updated Name",
            "bio": "Updated bio text"
        }).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(body["display_name"], "Updated Name");
    assert_eq!(body["bio"], "Updated bio text");
}

#[test]
fn test_update_profile_wrong_token() {
    let client = test_client();
    let _ = create_test_profile(&client, "wrongtoken");

    let resp = client.patch("/api/v1/profiles/wrongtoken")
        .header(ContentType::JSON)
        .header(Header::new("X-Manage-Token", "bad-token"))
        .body(serde_json::json!({"bio": "fail"}).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::Unauthorized);
}

#[test]
fn test_update_profile_no_token() {
    let client = test_client();
    let _ = create_test_profile(&client, "notoken");

    let resp = client.patch("/api/v1/profiles/notoken")
        .header(ContentType::JSON)
        .body(serde_json::json!({"bio": "fail"}).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::Unauthorized);
}

#[test]
fn test_update_profile_no_fields() {
    let client = test_client();
    let (_, create_body) = create_test_profile(&client, "nofields");
    let token = create_body["manage_token"].as_str().unwrap();

    let resp = client.patch("/api/v1/profiles/nofields")
        .header(ContentType::JSON)
        .header(Header::new("X-Manage-Token", token.to_string()))
        .body("{}".to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

#[test]
fn test_delete_profile() {
    let client = test_client();
    let (_, create_body) = create_test_profile(&client, "deleteme");
    let token = create_body["manage_token"].as_str().unwrap();

    let resp = client.delete("/api/v1/profiles/deleteme")
        .header(Header::new("X-Manage-Token", token.to_string()))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);

    let resp = client.get("/api/v1/profiles/deleteme").dispatch();
    assert_eq!(resp.status(), Status::NotFound);
}

#[test]
fn test_delete_profile_wrong_token() {
    let client = test_client();
    let _ = create_test_profile(&client, "nodelete");

    let resp = client.delete("/api/v1/profiles/nodelete")
        .header(Header::new("X-Manage-Token", "wrong"))
        .dispatch();
    assert_eq!(resp.status(), Status::Unauthorized);
}

#[test]
fn test_add_crypto_address() {
    let client = test_client();
    let (_, create_body) = create_test_profile(&client, "cryptouser");
    let token = create_body["manage_token"].as_str().unwrap();

    let resp = client.post("/api/v1/profiles/cryptouser/addresses")
        .header(ContentType::JSON)
        .header(Header::new("X-Manage-Token", token.to_string()))
        .body(serde_json::json!({
            "network": "nostr",
            "address": "npub1ur3y0623fl2zcypulhd8craakaeuk7pjx5yrzda472nvhyfgrmusqtuvnd",
            "label": "identity"
        }).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::Created);
    let addr_body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(addr_body["network"], "nostr");
    assert_eq!(addr_body["verified"], false);
}

#[test]
fn test_add_and_delete_address() {
    let client = test_client();
    let (_, create_body) = create_test_profile(&client, "addrdelete");
    let token = create_body["manage_token"].as_str().unwrap();

    let resp = client.post("/api/v1/profiles/addrdelete/addresses")
        .header(ContentType::JSON)
        .header(Header::new("X-Manage-Token", token.to_string()))
        .body(serde_json::json!({"network": "bitcoin", "address": "bc1q..."}).to_string())
        .dispatch();
    let addr_body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let addr_id = addr_body["id"].as_str().unwrap().to_string();

    let resp = client.delete(format!("/api/v1/profiles/addrdelete/addresses/{}", addr_id))
        .header(Header::new("X-Manage-Token", token.to_string()))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);

    let resp = client.get("/api/v1/profiles/addrdelete").dispatch();
    let profile: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert!(profile["crypto_addresses"].as_array().unwrap().is_empty());
}

#[test]
fn test_add_invalid_network() {
    let client = test_client();
    let (_, create_body) = create_test_profile(&client, "badnet");
    let token = create_body["manage_token"].as_str().unwrap();

    let resp = client.post("/api/v1/profiles/badnet/addresses")
        .header(ContentType::JSON)
        .header(Header::new("X-Manage-Token", token.to_string()))
        .body(serde_json::json!({"network": "dogecoin", "address": "D8cV31..."}).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

#[test]
fn test_add_and_delete_link() {
    let client = test_client();
    let (_, create_body) = create_test_profile(&client, "linkuser");
    let token = create_body["manage_token"].as_str().unwrap();

    let resp = client.post("/api/v1/profiles/linkuser/links")
        .header(ContentType::JSON)
        .header(Header::new("X-Manage-Token", token.to_string()))
        .body(serde_json::json!({
            "link_type": "github",
            "label": "GitHub",
            "value": "https://github.com/nanookclaw"
        }).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::Created);
    let link_body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let link_id = link_body["id"].as_str().unwrap().to_string();

    let resp = client.delete(format!("/api/v1/profiles/linkuser/links/{}", link_id))
        .header(Header::new("X-Manage-Token", token.to_string()))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);
}

#[test]
fn test_add_invalid_link_type() {
    let client = test_client();
    let (_, create_body) = create_test_profile(&client, "badlink");
    let token = create_body["manage_token"].as_str().unwrap();

    let resp = client.post("/api/v1/profiles/badlink/links")
        .header(ContentType::JSON)
        .header(Header::new("X-Manage-Token", token.to_string()))
        .body(serde_json::json!({"link_type": "discord", "label": "Discord", "value": "https://..."}).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::UnprocessableEntity);
}

#[test]
fn test_add_and_deduplicate_skill() {
    let client = test_client();
    let (_, create_body) = create_test_profile(&client, "skilluser");
    let token = create_body["manage_token"].as_str().unwrap();

    // Add skill (uppercase → normalized to lowercase)
    let resp = client.post("/api/v1/profiles/skilluser/skills")
        .header(ContentType::JSON)
        .header(Header::new("X-Manage-Token", token.to_string()))
        .body(serde_json::json!({"skill": "Rust"}).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::Created);
    let skill_body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(skill_body["skill"], "rust");

    // Duplicate → conflict
    let resp = client.post("/api/v1/profiles/skilluser/skills")
        .header(ContentType::JSON)
        .header(Header::new("X-Manage-Token", token.to_string()))
        .body(serde_json::json!({"skill": "rust"}).to_string())
        .dispatch();
    assert_eq!(resp.status(), Status::Conflict);
}

#[test]
fn test_add_and_delete_skill() {
    let client = test_client();
    let (_, create_body) = create_test_profile(&client, "skilldel");
    let token = create_body["manage_token"].as_str().unwrap();

    let resp = client.post("/api/v1/profiles/skilldel/skills")
        .header(ContentType::JSON)
        .header(Header::new("X-Manage-Token", token.to_string()))
        .body(serde_json::json!({"skill": "openclaw"}).to_string())
        .dispatch();
    let skill_body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let skill_id = skill_body["id"].as_str().unwrap().to_string();

    let resp = client.delete(format!("/api/v1/profiles/skilldel/skills/{}", skill_id))
        .header(Header::new("X-Manage-Token", token.to_string()))
        .dispatch();
    assert_eq!(resp.status(), Status::Ok);

    let resp = client.get("/api/v1/profiles/skilldel").dispatch();
    let profile: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert!(profile["skills"].as_array().unwrap().is_empty());
}

#[test]
fn test_list_profiles() {
    let client = test_client();
    create_test_profile(&client, "list-a");
    create_test_profile(&client, "list-b");

    let resp = client.get("/api/v1/profiles").dispatch();
    assert_eq!(resp.status(), Status::Ok);
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let profiles = body["profiles"].as_array().unwrap();
    assert!(profiles.len() >= 2);
}

#[test]
fn test_list_profiles_search() {
    let client = test_client();
    create_test_profile(&client, "searchable-one");
    create_test_profile(&client, "other-profile-two");

    let resp = client.get("/api/v1/profiles?q=searchable-one").dispatch();
    let body: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let profiles = body["profiles"].as_array().unwrap();
    assert!(profiles.iter().any(|p| p["slug"] == "searchable-one"));
}

#[test]
fn test_delete_cascades_to_sub_resources() {
    let client = test_client();
    let (_, create_body) = create_test_profile(&client, "cascade-del");
    let token = create_body["manage_token"].as_str().unwrap();

    // Add an address
    client.post("/api/v1/profiles/cascade-del/addresses")
        .header(ContentType::JSON)
        .header(Header::new("X-Manage-Token", token.to_string()))
        .body(serde_json::json!({"network": "bitcoin", "address": "bc1q..."}).to_string())
        .dispatch();

    // Delete profile
    client.delete("/api/v1/profiles/cascade-del")
        .header(Header::new("X-Manage-Token", token.to_string()))
        .dispatch();

    let resp = client.get("/api/v1/profiles/cascade-del").dispatch();
    assert_eq!(resp.status(), Status::NotFound);
}

#[test]
fn test_profile_has_all_sub_resources() {
    let client = test_client();
    let (_, create_body) = create_test_profile(&client, "fullprofile");
    let token = create_body["manage_token"].as_str().unwrap();

    // Add address
    client.post("/api/v1/profiles/fullprofile/addresses")
        .header(ContentType::JSON)
        .header(Header::new("X-Manage-Token", token.to_string()))
        .body(serde_json::json!({"network": "nostr", "address": "npub1..."}).to_string())
        .dispatch();

    // Add link
    client.post("/api/v1/profiles/fullprofile/links")
        .header(ContentType::JSON)
        .header(Header::new("X-Manage-Token", token.to_string()))
        .body(serde_json::json!({"link_type": "github", "label": "GitHub", "value": "https://github.com/test"}).to_string())
        .dispatch();

    // Add skill
    client.post("/api/v1/profiles/fullprofile/skills")
        .header(ContentType::JSON)
        .header(Header::new("X-Manage-Token", token.to_string()))
        .body(serde_json::json!({"skill": "rust"}).to_string())
        .dispatch();

    let resp = client.get("/api/v1/profiles/fullprofile").dispatch();
    let profile: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    assert_eq!(profile["crypto_addresses"].as_array().unwrap().len(), 1);
    assert_eq!(profile["links"].as_array().unwrap().len(), 1);
    assert_eq!(profile["skills"].as_array().unwrap().len(), 1);
}
