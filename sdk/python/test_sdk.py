#!/usr/bin/env python3
"""
Integration tests for the Agent Profile Python SDK.

Usage:
    # Start the service first (cargo run or docker)
    AGENT_PROFILE_URL=http://localhost:8003 python -m pytest test_sdk.py -v
    # or: python test_sdk.py

Requires the service running at AGENT_PROFILE_URL (default http://localhost:8003).
"""

import os
import sys
import time
import unittest
import uuid

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from agent_profile import AgentProfile, AgentProfileError

BASE_URL = os.environ.get("AGENT_PROFILE_URL", "http://localhost:8003")

# Throttle registrations to avoid rate limiting.
# Set REGISTER_RATE_LIMIT=1000 on the server for fast test runs.
_last_register = 0.0
_THROTTLE_SECS = float(os.environ.get("SDK_TEST_THROTTLE", "0.1"))
def throttled_register(ap, name, **kw):
    global _last_register
    elapsed = time.time() - _last_register
    if elapsed < _THROTTLE_SECS:
        time.sleep(_THROTTLE_SECS - elapsed)
    _last_register = time.time()
    return ap.register(name, **kw)


def unique(prefix="sdk"):
    return f"{prefix}-{uuid.uuid4().hex[:8]}"


class TestHealth(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.ap = AgentProfile(BASE_URL)

    def test_health(self):
        r = self.ap.health()
        self.assertEqual(r["status"], "ok")
        self.assertIn("version", r)

    def test_stats(self):
        r = self.ap.stats()
        self.assertIn("profiles", r)
        self.assertIn("total", r["profiles"])
        self.assertIn("skills", r)
        self.assertIn("endorsements", r)

    def test_openapi(self):
        r = self.ap.openapi()
        self.assertIn("openapi", r)
        self.assertIn("paths", r)


class TestRegistration(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.ap = AgentProfile(BASE_URL)

    def test_register(self):
        name = unique("reg")
        r = throttled_register(self.ap, name)
        self.assertIn("api_key", r)
        self.assertEqual(r["username"], name)
        self.assertIn("profile_url", r)
        self.assertIn("json_url", r)
        self.ap.delete(name, r["api_key"])

    def test_register_with_display_name(self):
        name = unique("reg-dn")
        r = throttled_register(self.ap, name, display_name="SDK Test Bot")
        self.assertIn("api_key", r)
        profile = self.ap.get(name)
        self.assertEqual(profile["display_name"], "SDK Test Bot")
        self.ap.delete(name, r["api_key"])

    def test_register_duplicate(self):
        name = unique("reg-dup")
        r = throttled_register(self.ap, name)
        with self.assertRaises(AgentProfileError) as ctx:
            throttled_register(self.ap, name)
        self.assertEqual(ctx.exception.status, 409)
        self.ap.delete(name, r["api_key"])

    def test_register_invalid_username(self):
        with self.assertRaises(AgentProfileError) as ctx:
            self.ap.register("ab")  # too short, no throttle needed (will fail fast)
        self.assertIn(ctx.exception.status, [400, 422])


class TestProfileCRUD(unittest.TestCase):
    """Uses a single registered profile for all tests in this class."""

    @classmethod
    def setUpClass(cls):
        cls.ap = AgentProfile(BASE_URL)
        cls.name = unique("crud")
        r = throttled_register(cls.ap, cls.name)
        cls.api_key = r["api_key"]

    @classmethod
    def tearDownClass(cls):
        try:
            cls.ap.delete(cls.name, cls.api_key)
        except AgentProfileError:
            pass

    def test_01_get_profile(self):
        p = self.ap.get(self.name)
        self.assertEqual(p["username"], self.name)
        self.assertIn("created_at", p)
        self.assertIn("view_count", p)
        self.assertIsInstance(p["links"], list)
        self.assertIsInstance(p["sections"], list)
        self.assertIsInstance(p["skills"], list)
        self.assertIsInstance(p["endorsements"], list)

    def test_02_get_nonexistent(self):
        with self.assertRaises(AgentProfileError) as ctx:
            self.ap.get("no-such-agent-xyz-99")
        self.assertEqual(ctx.exception.status, 404)

    def test_03_update_profile(self):
        self.ap.update(
            self.name, self.api_key,
            display_name="Updated Bot",
            tagline="Test tagline",
            bio="Test bio for SDK",
            theme="midnight",
        )
        p = self.ap.get(self.name)
        self.assertEqual(p["display_name"], "Updated Bot")
        self.assertEqual(p["tagline"], "Test tagline")
        self.assertEqual(p["bio"], "Test bio for SDK")
        self.assertEqual(p["theme"], "midnight")

    def test_04_update_no_fields(self):
        with self.assertRaises(ValueError):
            self.ap.update(self.name, self.api_key)

    def test_05_update_wrong_api_key(self):
        with self.assertRaises(AgentProfileError) as ctx:
            self.ap.update(self.name, "wrong-key", display_name="Hacker")
        self.assertEqual(ctx.exception.status, 401)

    def test_06_score(self):
        r = self.ap.score(self.name)
        self.assertIn("score", r)
        self.assertIn("max_score", r)
        self.assertEqual(r["max_score"], 100)
        self.assertIn("breakdown", r)
        self.assertIn("next_steps", r)


class TestDeleteAndReissue(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.ap = AgentProfile(BASE_URL)

    def test_delete_profile(self):
        name = unique("del")
        r = throttled_register(self.ap, name)
        self.ap.delete(name, r["api_key"])
        with self.assertRaises(AgentProfileError) as ctx:
            self.ap.get(name)
        self.assertEqual(ctx.exception.status, 404)

    def test_reissue_key(self):
        name = unique("reissue")
        r = throttled_register(self.ap, name)
        old_key = r["api_key"]
        r2 = self.ap.reissue_key(name, old_key)
        new_key = r2["api_key"]
        self.assertNotEqual(new_key, old_key)
        # Old key should fail
        with self.assertRaises(AgentProfileError) as ctx:
            self.ap.update(name, old_key, tagline="old key")
        self.assertEqual(ctx.exception.status, 401)
        # New key works
        self.ap.update(name, new_key, tagline="new key works")
        self.ap.delete(name, new_key)


class TestSubResources(unittest.TestCase):
    """Tests links, addresses, sections, skills on a single profile."""

    @classmethod
    def setUpClass(cls):
        cls.ap = AgentProfile(BASE_URL)
        cls.name = unique("subres")
        r = throttled_register(cls.ap, cls.name)
        cls.api_key = r["api_key"]

    @classmethod
    def tearDownClass(cls):
        try:
            cls.ap.delete(cls.name, cls.api_key)
        except AgentProfileError:
            pass

    def test_01_add_and_remove_link(self):
        r = self.ap.add_link(
            self.name, self.api_key,
            url="https://github.com/test-bot",
            label="GitHub",
            platform="github",
        )
        self.assertIn("id", r)
        link_id = r["id"]
        p = self.ap.get(self.name)
        urls = [l["url"] for l in p["links"]]
        self.assertIn("https://github.com/test-bot", urls)
        self.ap.remove_link(self.name, self.api_key, link_id)
        p = self.ap.get(self.name)
        self.assertEqual(len(p["links"]), 0)

    def test_02_add_multiple_links(self):
        r1 = self.ap.add_link(self.name, self.api_key, url="https://a.com", label="A", platform="website")
        r2 = self.ap.add_link(self.name, self.api_key, url="https://b.com", label="B", platform="website")
        p = self.ap.get(self.name)
        self.assertEqual(len(p["links"]), 2)
        self.ap.remove_link(self.name, self.api_key, r1["id"])
        self.ap.remove_link(self.name, self.api_key, r2["id"])

    def test_03_add_and_remove_address(self):
        r = self.ap.add_address(
            self.name, self.api_key,
            network="bitcoin",
            address="1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
            label="Satoshi",
        )
        self.assertIn("id", r)
        addr_id = r["id"]
        p = self.ap.get(self.name)
        self.assertEqual(len(p["crypto_addresses"]), 1)
        self.assertEqual(p["crypto_addresses"][0]["network"], "bitcoin")
        self.ap.remove_address(self.name, self.api_key, addr_id)

    def test_04_add_update_remove_section(self):
        r = self.ap.add_section(
            self.name, self.api_key,
            title="About",
            content="I am a test agent.",
        )
        self.assertIn("id", r)
        section_id = r["id"]
        p = self.ap.get(self.name)
        self.assertEqual(len(p["sections"]), 1)
        self.assertEqual(p["sections"][0]["title"], "About")
        self.ap.update_section(
            self.name, self.api_key, section_id,
            content="Updated content.",
        )
        p = self.ap.get(self.name)
        self.assertEqual(p["sections"][0]["content"], "Updated content.")
        self.ap.remove_section(self.name, self.api_key, section_id)

    def test_05_add_and_remove_skill(self):
        r = self.ap.add_skill(self.name, self.api_key, skill="python")
        self.assertIn("id", r)
        skill_id = r["id"]
        p = self.ap.get(self.name)
        skill_names = [s["skill"] for s in p["skills"]]
        self.assertIn("python", skill_names)
        self.ap.remove_skill(self.name, self.api_key, skill_id)

    def test_06_skills_directory(self):
        r = self.ap.add_skill(self.name, self.api_key, skill="sdk-test-unique-skill")
        skills = self.ap.skills(q="sdk-test-unique")
        self.assertIn("skills", skills)
        found = [s for s in skills["skills"] if s["skill"] == "sdk-test-unique-skill"]
        self.assertEqual(len(found), 1)
        self.ap.remove_skill(self.name, self.api_key, r["id"])


class TestSearch(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.ap = AgentProfile(BASE_URL)
        cls.name = unique("search")
        r = throttled_register(cls.ap, cls.name)
        cls.api_key = r["api_key"]
        cls.ap.update(cls.name, cls.api_key, bio="unique-sdk-search-marker-42")
        s = cls.ap.add_skill(cls.name, cls.api_key, skill="sdk-search-skill")
        cls.skill_id = s["id"]

    @classmethod
    def tearDownClass(cls):
        try:
            cls.ap.delete(cls.name, cls.api_key)
        except AgentProfileError:
            pass

    def test_search_by_text(self):
        r = self.ap.search(q="unique-sdk-search-marker-42")
        self.assertIn("profiles", r)
        self.assertGreaterEqual(len(r["profiles"]), 1)
        usernames = [p["username"] for p in r["profiles"]]
        self.assertIn(self.name, usernames)

    def test_search_by_skill(self):
        r = self.ap.search(skill="sdk-search-skill")
        self.assertIn("profiles", r)
        usernames = [p["username"] for p in r["profiles"]]
        self.assertIn(self.name, usernames)

    def test_search_sort_popular(self):
        r = self.ap.search(sort="popular", limit=5)
        self.assertIn("profiles", r)
        self.assertLessEqual(len(r["profiles"]), 5)

    def test_search_sort_newest(self):
        r = self.ap.search(sort="newest", limit=5)
        self.assertIn("profiles", r)

    def test_search_pagination(self):
        r = self.ap.search(limit=2, offset=0)
        self.assertLessEqual(len(r["profiles"]), 2)
        self.assertEqual(r["limit"], 2)
        self.assertEqual(r["offset"], 0)


class TestEndorsements(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.ap = AgentProfile(BASE_URL)
        cls.endorser = unique("endorser")
        cls.endorsee = unique("endorsee")
        r1 = throttled_register(cls.ap, cls.endorser)
        r2 = throttled_register(cls.ap, cls.endorsee)
        cls.endorser_key = r1["api_key"]
        cls.endorsee_key = r2["api_key"]

    @classmethod
    def tearDownClass(cls):
        try:
            cls.ap.delete(cls.endorser, cls.endorser_key)
        except AgentProfileError:
            pass
        try:
            cls.ap.delete(cls.endorsee, cls.endorsee_key)
        except AgentProfileError:
            pass

    def test_01_endorse_and_list(self):
        r = self.ap.endorse(
            self.endorsee,
            from_user=self.endorser,
            api_key=self.endorser_key,
            message="Great agent!",
        )
        self.assertIn("id", r)
        endorsements = self.ap.endorsements(self.endorsee)
        self.assertGreaterEqual(len(endorsements), 1)
        msgs = [e["message"] for e in endorsements]
        self.assertIn("Great agent!", msgs)

    def test_02_endorse_upsert(self):
        self.ap.endorse(
            self.endorsee, from_user=self.endorser,
            api_key=self.endorser_key, message="Updated message",
        )
        endorsements = self.ap.endorsements(self.endorsee)
        msgs = [e["message"] for e in endorsements]
        self.assertIn("Updated message", msgs)

    def test_03_self_endorse_rejected(self):
        with self.assertRaises(AgentProfileError) as ctx:
            self.ap.endorse(
                self.endorser, from_user=self.endorser,
                api_key=self.endorser_key, message="I'm great",
            )
        self.assertEqual(ctx.exception.status, 422)

    def test_04_wrong_api_key(self):
        with self.assertRaises(AgentProfileError) as ctx:
            self.ap.endorse(
                self.endorsee, from_user=self.endorser,
                api_key=self.endorsee_key,
                message="Shouldn't work",
            )
        self.assertEqual(ctx.exception.status, 401)

    def test_05_remove_endorsement(self):
        self.ap.remove_endorsement(self.endorsee, self.endorser, self.endorser_key)
        endorsements = self.ap.endorsements(self.endorsee)
        self.assertEqual(len(endorsements), 0)


class TestWebFinger(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.ap = AgentProfile(BASE_URL)
        cls.name = unique("wf")
        r = throttled_register(cls.ap, cls.name)
        cls.api_key = r["api_key"]

    @classmethod
    def tearDownClass(cls):
        try:
            cls.ap.delete(cls.name, cls.api_key)
        except AgentProfileError:
            pass

    def test_webfinger(self):
        r = self.ap.webfinger(self.name)
        self.assertIn("subject", r)
        self.assertIn("links", r)


if __name__ == "__main__":
    unittest.main()
