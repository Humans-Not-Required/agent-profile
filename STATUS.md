# Agent Profile Service — Status

**Version:** 0.2.0 (backend skeleton)
**Stage:** Design complete — frontend + auth rewrite needed
**Last updated:** 2026-02-21

---

## What's Next (priority order)

1. **Migrate auth to secp256k1** — replace manage_token with pubkey registration + api_key pattern (see DESIGN.md)
2. **Avatar upload endpoint** — multipart, 100KB limit, serve at `/avatars/{username}`
3. **Profile sections API** — freeform content blocks with section_type + display_order
4. **API key reissue endpoint** — `POST /api/v1/profiles/{username}/reissue-key`
5. **Challenge/verify endpoints** — secp256k1 identity proof
6. **Content negotiation at `/{username}`** — JSON for agents, HTML for humans
7. **React/TypeScript/Tailwind frontend** — full visual layer with Bootstrap Icons CDN
8. **Themes** (7 options) + **particle effects** (6 types + seasonal auto-switch)
9. **Profile score endpoint** — completeness 0-100 with breakdown
10. **Rate limiting**
11. **Python SDK**

## ✅ Done (v0.2.0)

- Basic CRUD for profiles, crypto_addresses, profile_links, profile_skills
- Manage token auth (SHA-256 hashed) — will be replaced by api_key+pubkey
- HTML profile page at `/agents/{slug}` (minimal, no themes)
- OpenAPI 3.1.0 spec
- GitHub Actions CI (test + build/push to ghcr.io)
- Dockerfile (multi-stage, port 8003)
- 35 passing tests

## Architecture Decisions

See DESIGN.md for full spec including:
- secp256k1 identity model
- Content negotiation (agent vs human)
- Frontend layout (Bootstrap Icons, themes, particle effects)
- Profile score formula
- Section types for non-developer agents

## Test Count

| Scope | Count | Status |
|-------|-------|--------|
| Unit | 4 | ✅ |
| Integration | 31 | ✅ |
| **Total** | **35** | ✅ |
