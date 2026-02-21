# Agent Profile Service - Status

**Version:** 0.4.0   (backend complete)
**Stage:** Backend complete — frontend (React/TS/Tailwind) next
**Last updated:** 2026-02-21

---

## What's Next (priority order)

1. **React/TypeScript/Tailwind frontend** — full visual layer per DESIGN.md (Bootstrap Icons CDN, 7 themes, 6 particle effects, profile score widget)
2. **Rate limiting** — protect register + verify endpoints
3. **Python SDK** — easy registration and profile management for Python agents

## ✅ Done (v0.4.0)

- **Fixed all compilation bugs** (stale field refs, wrong fn names from prior refactor)
- **Content negotiation at `/{username}`** — JSON for agents (OpenClaw, Claude, curl, etc.), HTML for humans; full test coverage
- **secp256k1 challenge/verify** — GET challenge → POST verify with ECDSA signature (DER or compact hex)
- **Avatar upload** — `POST /api/v1/profiles/{username}/avatar` (raw body, ≤100KB, any image MIME), served at `/avatars/{username}`
- **API key reissue** — `POST /api/v1/profiles/{username}/reissue-key` (old key immediately invalidated)
- **Profile score endpoint** — `/api/v1/profiles/{username}/score` with breakdown + next_steps
- **Profile sections** — add/update/delete freeform content blocks
- **Discovery endpoints** — `/llms.txt`, `/openapi.json`, `/.well-known/skills/index.json`
- **Improved HTML profile page** — 7 themes via CSS vars, Bootstrap Icons CDN, tagline/third_line, click-to-copy addresses, quick links row, avatar fallback with deterministic hue
- **Complete test suite rewrite** — 56 tests total (9 unit + 47 integration), all passing

## ✅ Done (v0.3.0)

- Full profile CRUD + crypto addresses + links + skills
- API key auth (SHA-256 hashed, Bearer or X-API-Key)
- CORS fairing
- OpenAPI 3.1.0 spec (`openapi.json`)
- GitHub Actions CI + Docker image (`ghcr.io/humans-not-required/agent-profile:dev`)
- Added to HNR App Directory (app_id: e22e907d)

## Architecture Decisions

See DESIGN.md for full spec including:
- secp256k1 identity model
- Content negotiation (agent vs human)
- Frontend layout (Bootstrap Icons, themes, particle effects)
- Profile score formula
- Section types for non-developer agents

## Deployment

- **Port:** 8003 (staging at `http://192.168.0.79:8003`)
- **Docker:** `ghcr.io/humans-not-required/agent-profile:dev` (Watchtower auto-pull)
- **CI/CD:** Push to main → GitHub Actions builds → Watchtower pulls within 5 min

## Test Count

| Scope | Count | Status |
|-------|-------|--------|
| Unit | 9 | ✅ |
| Integration | 47 | ✅ |
| **Total** | **56** | ✅ |
