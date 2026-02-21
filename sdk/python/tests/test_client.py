"""Unit tests for AgentProfileClient using respx to mock httpx."""

import pytest
import respx
import httpx

from agent_profile import (
    AgentProfileClient,
    ConflictError,
    NotFoundError,
    RateLimitError,
    UnauthorizedError,
    ValidationError,
)

BASE = "https://test.example.com"


@pytest.fixture
def client():
    with AgentProfileClient(BASE) as c:
        yield c


# ── Health ─────────────────────────────────────────────────────────────────────

@respx.mock
def test_health(client):
    respx.get(f"{BASE}/api/v1/health").mock(
        return_value=httpx.Response(200, json={"status": "ok", "version": "0.4.0", "service": "agent-profile"})
    )
    result = client.health()
    assert result["status"] == "ok"
    assert result["service"] == "agent-profile"


# ── Registration ───────────────────────────────────────────────────────────────

@respx.mock
def test_register(client):
    respx.post(f"{BASE}/api/v1/register").mock(
        return_value=httpx.Response(201, json={
            "username": "nanook",
            "api_key": "ap_abc123",
            "profile_url": "/nanook",
            "json_url": "/api/v1/profiles/nanook",
        })
    )
    result = client.register("nanook")
    assert result["username"] == "nanook"
    assert result["api_key"] == "ap_abc123"


@respx.mock
def test_register_conflict(client):
    respx.post(f"{BASE}/api/v1/register").mock(
        return_value=httpx.Response(409, json={"error": "Username 'nanook' already taken"})
    )
    with pytest.raises(ConflictError) as exc_info:
        client.register("nanook")
    assert "already taken" in str(exc_info.value)
    assert exc_info.value.status_code == 409


@respx.mock
def test_register_validation_error(client):
    respx.post(f"{BASE}/api/v1/register").mock(
        return_value=httpx.Response(422, json={"error": "Username must be 3–30 characters"})
    )
    with pytest.raises(ValidationError):
        client.register("ab")


@respx.mock
def test_register_rate_limited(client):
    respx.post(f"{BASE}/api/v1/register").mock(
        return_value=httpx.Response(429, json={
            "error": "Too many requests. Please slow down and try again later.",
            "retry_after_seconds": 60,
        })
    )
    with pytest.raises(RateLimitError):
        client.register("newuser")


# ── Get Profile ────────────────────────────────────────────────────────────────

@respx.mock
def test_get_profile(client):
    profile_data = {
        "id": "abc-123",
        "username": "nanook",
        "display_name": "Nanook ❄️",
        "tagline": "Autonomous AI agent",
        "bio": "I build things.",
        "third_line": "",
        "avatar_url": "",
        "theme": "dark",
        "particle_effect": "none",
        "particle_enabled": False,
        "particle_seasonal": False,
        "pubkey": "",
        "profile_score": 40,
        "created_at": "2026-02-21T00:00:00Z",
        "updated_at": "2026-02-21T00:00:00Z",
        "crypto_addresses": [],
        "links": [],
        "sections": [],
        "skills": [],
    }
    respx.get(f"{BASE}/api/v1/profiles/nanook").mock(
        return_value=httpx.Response(200, json=profile_data)
    )
    result = client.get_profile("nanook")
    assert result["username"] == "nanook"
    assert result["display_name"] == "Nanook ❄️"
    assert result["profile_score"] == 40


@respx.mock
def test_get_profile_not_found(client):
    respx.get(f"{BASE}/api/v1/profiles/ghost").mock(
        return_value=httpx.Response(404, json={"error": "Profile 'ghost' not found"})
    )
    with pytest.raises(NotFoundError) as exc_info:
        client.get_profile("ghost")
    assert exc_info.value.status_code == 404


# ── Update Profile ────────────────────────────────────────────────────────────

@respx.mock
def test_update_profile(client):
    updated = {"username": "nanook", "display_name": "Updated", "profile_score": 55}
    respx.patch(f"{BASE}/api/v1/profiles/nanook").mock(
        return_value=httpx.Response(200, json=updated)
    )
    result = client.update_profile("nanook", "ap_key", display_name="Updated")
    assert result["display_name"] == "Updated"


def test_update_profile_no_fields(client):
    with pytest.raises(ValueError, match="No fields to update"):
        client.update_profile("nanook", "ap_key")


@respx.mock
def test_update_profile_unauthorized(client):
    respx.patch(f"{BASE}/api/v1/profiles/nanook").mock(
        return_value=httpx.Response(401, json={"error": "Invalid API key"})
    )
    with pytest.raises(UnauthorizedError):
        client.update_profile("nanook", "wrong_key", display_name="Hacked")


# ── Links ──────────────────────────────────────────────────────────────────────

@respx.mock
def test_add_link(client):
    respx.post(f"{BASE}/api/v1/profiles/nanook/links").mock(
        return_value=httpx.Response(201, json={
            "id": "link-123",
            "url": "https://github.com/nanook",
            "label": "GitHub",
            "platform": "github",
            "display_order": 0,
            "created_at": "2026-02-21T00:00:00Z",
        })
    )
    result = client.add_link("nanook", "ap_key",
        url="https://github.com/nanook", label="GitHub", platform="github")
    assert result["platform"] == "github"
    assert result["id"] == "link-123"


# ── Crypto Addresses ──────────────────────────────────────────────────────────

@respx.mock
def test_add_address(client):
    respx.post(f"{BASE}/api/v1/profiles/nanook/addresses").mock(
        return_value=httpx.Response(201, json={
            "id": "addr-456",
            "network": "bitcoin",
            "address": "bc1qtest",
            "label": "tips",
            "created_at": "2026-02-21T00:00:00Z",
        })
    )
    result = client.add_address("nanook", "ap_key",
        network="bitcoin", address="bc1qtest", label="tips")
    assert result["network"] == "bitcoin"


# ── Sections ──────────────────────────────────────────────────────────────────

@respx.mock
def test_add_section(client):
    respx.post(f"{BASE}/api/v1/profiles/nanook/sections").mock(
        return_value=httpx.Response(201, json={
            "id": "sec-789",
            "section_type": "about",
            "title": "About Me",
            "content": "I am an AI agent.",
            "display_order": 0,
            "created_at": "2026-02-21T00:00:00Z",
        })
    )
    result = client.add_section("nanook", "ap_key",
        title="About Me", content="I am an AI agent.", section_type="about")
    assert result["title"] == "About Me"
    assert result["id"] == "sec-789"


# ── Skills ────────────────────────────────────────────────────────────────────

@respx.mock
def test_add_skill(client):
    respx.post(f"{BASE}/api/v1/profiles/nanook/skills").mock(
        return_value=httpx.Response(201, json={
            "id": "skill-001",
            "skill": "rust",
            "created_at": "2026-02-21T00:00:00Z",
        })
    )
    result = client.add_skill("nanook", "ap_key", "Rust")
    assert result["skill"] == "rust"


# ── Score ──────────────────────────────────────────────────────────────────────

@respx.mock
def test_get_score(client):
    respx.get(f"{BASE}/api/v1/profiles/nanook/score").mock(
        return_value=httpx.Response(200, json={
            "score": 65,
            "max_score": 100,
            "breakdown": [
                {"field": "display_name", "label": "Display name set", "points": 10, "earned": True},
            ],
            "next_steps": ["Add a crypto address"],
        })
    )
    result = client.get_score("nanook")
    assert result["score"] == 65
    assert len(result["breakdown"]) == 1
    assert result["next_steps"] == ["Add a crypto address"]


# ── Challenge / Verify ────────────────────────────────────────────────────────

@respx.mock
def test_get_challenge(client):
    challenge_hex = "a" * 64
    respx.get(f"{BASE}/api/v1/profiles/nanook/challenge").mock(
        return_value=httpx.Response(200, json={
            "challenge": challenge_hex,
            "expires_in_seconds": 300,
            "instructions": "Sign this with your secp256k1 key.",
        })
    )
    result = client.get_challenge("nanook")
    assert len(result["challenge"]) == 64
    assert result["expires_in_seconds"] == 300


@respx.mock
def test_verify(client):
    respx.post(f"{BASE}/api/v1/profiles/nanook/verify").mock(
        return_value=httpx.Response(200, json={
            "verified": True,
            "username": "nanook",
            "timestamp": "2026-02-21T00:00:00Z",
        })
    )
    result = client.verify("nanook", "deadbeef" * 8)
    assert result["verified"] is True


# ── List Profiles ─────────────────────────────────────────────────────────────

@respx.mock
def test_list_profiles(client):
    respx.get(f"{BASE}/api/v1/profiles").mock(
        return_value=httpx.Response(200, json={
            "profiles": [{"username": "nanook", "display_name": "Nanook ❄️", "profile_score": 80}],
            "total": 1,
            "limit": 20,
            "offset": 0,
        })
    )
    result = client.list_profiles()
    assert len(result["profiles"]) == 1
    assert result["profiles"][0]["username"] == "nanook"


@respx.mock
def test_list_profiles_search(client):
    respx.get(f"{BASE}/api/v1/profiles").mock(
        return_value=httpx.Response(200, json={"profiles": [], "total": 0, "limit": 20, "offset": 0})
    )
    result = client.list_profiles(q="zebraduck")
    assert result["total"] == 0


@respx.mock
def test_list_profiles_by_skill(client):
    respx.get(f"{BASE}/api/v1/profiles").mock(
        return_value=httpx.Response(200, json={
            "profiles": [{"username": "rust-bot", "display_name": "Rust Bot", "profile_score": 70}],
            "total": 1, "limit": 20, "offset": 0,
        })
    )
    result = client.list_profiles(skill="Rust")
    assert result["total"] == 1
    assert result["profiles"][0]["username"] == "rust-bot"


@respx.mock
def test_list_profiles_has_pubkey(client):
    respx.get(f"{BASE}/api/v1/profiles").mock(
        return_value=httpx.Response(200, json={
            "profiles": [{"username": "crypto-agent", "display_name": "Crypto", "profile_score": 90}],
            "total": 1, "limit": 20, "offset": 0,
        })
    )
    result = client.list_profiles(has_pubkey=True)
    assert result["total"] == 1
    assert result["profiles"][0]["username"] == "crypto-agent"


@respx.mock
def test_list_profiles_skill_and_query(client):
    respx.get(f"{BASE}/api/v1/profiles").mock(
        return_value=httpx.Response(200, json={"profiles": [], "total": 0, "limit": 20, "offset": 0})
    )
    result = client.list_profiles(skill="Go", q="distributed")
    assert result["total"] == 0


# ── Error handling ────────────────────────────────────────────────────────────

@respx.mock
def test_server_error(client):
    respx.get(f"{BASE}/api/v1/profiles/broken").mock(
        return_value=httpx.Response(500, json={"error": "Internal server error"})
    )
    from agent_profile import ServerError
    with pytest.raises(ServerError) as exc_info:
        client.get_profile("broken")
    assert exc_info.value.status_code == 500


@respx.mock
def test_non_json_error_response(client):
    respx.get(f"{BASE}/api/v1/profiles/badresponse").mock(
        return_value=httpx.Response(503, text="Service Unavailable")
    )
    from agent_profile import AgentProfileError
    with pytest.raises(AgentProfileError) as exc_info:
        client.get_profile("badresponse")
    assert exc_info.value.status_code == 503


# ── Context manager ───────────────────────────────────────────────────────────

def test_context_manager():
    """Client should work as a context manager."""
    with AgentProfileClient(BASE) as c:
        assert c is not None


# ── Endorsements ───────────────────────────────────────────────────────────────

ENDORSEMENT_RESPONSE = {
    "id": "end-uuid-001",
    "endorsee": "nanook",
    "endorser": "jiggai",
    "message": "Outstanding collaborator.",
    "verified": False,
    "created_at": "2026-02-21T10:00:00Z",
}

ENDORSEMENT_LIST_RESPONSE = {
    "username": "nanook",
    "endorsements": [
        {
            "id": "end-uuid-001",
            "endorsee_id": "profile-uuid-001",
            "endorser_username": "jiggai",
            "message": "Outstanding collaborator.",
            "signature": "",
            "verified": False,
            "created_at": "2026-02-21T10:00:00Z",
        }
    ],
    "total": 1,
    "verified_count": 0,
}


@respx.mock
def test_get_endorsements(client):
    respx.get(f"{BASE}/api/v1/profiles/nanook/endorsements").mock(
        return_value=httpx.Response(200, json=ENDORSEMENT_LIST_RESPONSE)
    )
    result = client.get_endorsements("nanook")
    assert result["total"] == 1
    assert result["endorsements"][0]["endorser_username"] == "jiggai"
    assert result["verified_count"] == 0


@respx.mock
def test_get_endorsements_empty(client):
    respx.get(f"{BASE}/api/v1/profiles/empty/endorsements").mock(
        return_value=httpx.Response(200, json={"username": "empty", "endorsements": [], "total": 0, "verified_count": 0})
    )
    result = client.get_endorsements("empty")
    assert result["total"] == 0
    assert result["endorsements"] == []


@respx.mock
def test_get_endorsements_not_found(client):
    respx.get(f"{BASE}/api/v1/profiles/ghost/endorsements").mock(
        return_value=httpx.Response(404, json={"error": "Profile 'ghost' not found"})
    )
    with pytest.raises(NotFoundError):
        client.get_endorsements("ghost")


@respx.mock
def test_add_endorsement(client):
    respx.post(f"{BASE}/api/v1/profiles/nanook/endorsements").mock(
        return_value=httpx.Response(200, json=ENDORSEMENT_RESPONSE)
    )
    result = client.add_endorsement("nanook", "jiggai", "ap_key123", "Outstanding collaborator.")
    assert result["endorser"] == "jiggai"
    assert result["endorsee"] == "nanook"
    assert result["verified"] is False


@respx.mock
def test_add_endorsement_verified(client):
    verified_response = {**ENDORSEMENT_RESPONSE, "verified": True}
    respx.post(f"{BASE}/api/v1/profiles/nanook/endorsements").mock(
        return_value=httpx.Response(200, json=verified_response)
    )
    result = client.add_endorsement(
        "nanook", "jiggai", "ap_key123",
        "Outstanding collaborator.",
        signature="3045022100abcdef..."
    )
    assert result["verified"] is True


@respx.mock
def test_add_endorsement_upsert(client):
    upsert_response = {**ENDORSEMENT_RESPONSE, "updated": True}
    respx.post(f"{BASE}/api/v1/profiles/nanook/endorsements").mock(
        return_value=httpx.Response(200, json=upsert_response)
    )
    result = client.add_endorsement("nanook", "jiggai", "ap_key123", "Updated message.")
    assert result.get("updated") is True


@respx.mock
def test_add_endorsement_self_endorse(client):
    respx.post(f"{BASE}/api/v1/profiles/nanook/endorsements").mock(
        return_value=httpx.Response(422, json={"error": "Cannot endorse your own profile"})
    )
    with pytest.raises(ValidationError):
        client.add_endorsement("nanook", "nanook", "ap_key123", "I am great!")


@respx.mock
def test_add_endorsement_unauthorized(client):
    respx.post(f"{BASE}/api/v1/profiles/nanook/endorsements").mock(
        return_value=httpx.Response(401, json={"error": "Invalid API key"})
    )
    with pytest.raises(UnauthorizedError):
        client.add_endorsement("nanook", "jiggai", "wrong_key", "A message.")


@respx.mock
def test_delete_endorsement(client):
    respx.delete(f"{BASE}/api/v1/profiles/nanook/endorsements/jiggai").mock(
        return_value=httpx.Response(200, json={"deleted": True, "endorser": "jiggai", "endorsee": "nanook"})
    )
    result = client.delete_endorsement("nanook", "jiggai", "ap_key123")
    assert result["deleted"] is True
    assert result["endorser"] == "jiggai"


@respx.mock
def test_delete_endorsement_not_found(client):
    respx.delete(f"{BASE}/api/v1/profiles/nanook/endorsements/nobody").mock(
        return_value=httpx.Response(404, json={"error": "No endorsement from 'nobody' on 'nanook'"})
    )
    with pytest.raises(NotFoundError):
        client.delete_endorsement("nanook", "nobody", "ap_key123")
