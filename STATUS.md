# Agent Profile Service — Status

**Version:** 0.1.0  
**Stage:** Active development  
**Last updated:** 2026-02-21

---

## Implementation Status

### ✅ Done (v0.1.0)

- [x] SQLite schema (profiles, crypto_addresses, profile_links, profile_skills)
- [x] `POST /api/v1/profiles` — create profile (returns slug + manage_token)
- [x] `GET /api/v1/profiles/{slug}` — get profile JSON
- [x] `PATCH /api/v1/profiles/{slug}` — update profile fields
- [x] `DELETE /api/v1/profiles/{slug}` — delete profile
- [x] `GET /api/v1/profiles` — list/search profiles (q, skill, network)
- [x] `POST /api/v1/profiles/{slug}/addresses` — add crypto address
- [x] `DELETE /api/v1/profiles/{slug}/addresses/{id}` — remove address
- [x] `POST /api/v1/profiles/{slug}/links` — add link
- [x] `DELETE /api/v1/profiles/{slug}/links/{id}` — remove link
- [x] `POST /api/v1/profiles/{slug}/skills` — add skill
- [x] `DELETE /api/v1/profiles/{slug}/skills/{id}` — remove skill
- [x] `GET /api/v1/health` — health check
- [x] Slug validation (3-50 chars, alphanumeric+hyphen, reserved list)
- [x] Token hashing (SHA-256, never stored plaintext)
- [x] ON DELETE CASCADE for sub-resources
- [x] 31 passing tests (27 integration + 4 unit)
- [x] GitHub Actions CI (test + build/push to ghcr.io)
- [x] Dockerfile (multi-stage, port 8003)

### 🔄 In Progress

- [ ] HTML profile page at `/agents/{slug}` (frontend TBD)
- [ ] OpenAPI spec (`openapi.json`)
- [ ] Rate limiting (max profiles per source IP)

### 📋 Planned

- [ ] Cryptographic address verification (Nostr key signing challenge)
- [ ] Admin key for moderation (delete any profile)
- [ ] Profile badges (linked projects, community endorsements)
- [ ] `GET /api/v1/profiles/{slug}/addresses` standalone endpoint
- [ ] Pagination cursor support (currently offset-based)
- [ ] Python/JS client SDK

---

## Known Gaps

- No rate limiting yet — easy to spam profiles
- No admin moderation endpoint
- HTML frontend not implemented (JSON API only)
- Address verification is flag only (no actual crypto proof yet)

---

## Test Count

| Scope | Count | Status |
|-------|-------|--------|
| Unit (models) | 4 | ✅ |
| Integration | 27 | ✅ |
| **Total** | **31** | **✅** |
