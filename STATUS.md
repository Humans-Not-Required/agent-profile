# Agent Profile Service - Status

**Version:** 0.4.0   (production-ready)
**Stage:** Feature complete. README + OpenAPI fully updated. 85 tests. Awaiting Jordan: PyPI + prod domain.
**Last updated:** 2026-02-21

---

## What's Next (priority order)

1. **PyPI publish** — CI workflow ready (`.github/workflows/publish-sdk.yml`). Jordan: set up OIDC trusted publisher at pypi.org, then `git tag sdk-v0.1.0 && git push origin sdk-v0.1.0`
2. **Production domain** — wait for Jordan's signal on public DNS

## How to Publish to PyPI (for Jordan)

1. Go to https://pypi.org → Your account → Publishing → Add a new publisher
2. Fill in:
   - PyPI project name: `agent-profile`
   - GitHub owner: `Humans-Not-Required`
   - Repository: `agent-profile`
   - Workflow filename: `publish-sdk.yml`
   - Environment name: `pypi`
3. Push a tag: `git tag sdk-v0.1.0 && git push origin sdk-v0.1.0`
4. GitHub Actions builds + publishes automatically — no secrets needed

## ✅ Done (README rewrite — Feb 21)

- Rewrote `README.md` for v0.4.0 (was still v0.1.0 — wrong URLs, field names, missing features)
- Correct endpoints (register not POST /profiles, username not slug)
- Documents: themes, particle effects, sections, secp256k1 verify, content negotiation, Python SDK
- Full API reference table (all 19 route variants), rate limits, discovery endpoints
- Updated network/platform/section-type valid value lists

## ✅ Done (OpenAPI spec v0.4.0 — Feb 21)

- Rewrote `openapi.json` — v0.1.0 → v0.4.0, `{slug}` → `{username}`, 9 → 17 paths
- Added all missing endpoints: register, reissue-key, score, avatar, challenge, verify, sections CRUD
- Updated all schemas: Profile (tagline/bio/theme/particle_*/pubkey/sections), RegisterRequest/Response, Links (`url`+`platform` not `link_type`+`value`), Addresses (11-network enum), TooManyRequests response
- Security scheme: `ManageToken` (X-Manage-Token) → `ApiKey` (Bearer + X-API-Key)
- Added `test_openapi_json` integration test — validates spec structure + all 8 key paths
- **85 total tests** (13 unit + 50 integration + 22 Python SDK)

## ✅ Done (Nanook profile live — Feb 21)

- **Nanook's profile** created on staging (`http://192.168.0.79:3011/nanook`)
- Profile score: **100/100** (display name, bio, tagline, avatar, pubkey, links, skills, sections, Nostr address)
- Fixed `nanook` in `RESERVED_USERNAMES` (was blocking self-registration)
- API key saved to `memory/state/nanook-profile.json`
- `examples/nanook_profile.py` updated to produce 100/100 score out of the box

## ✅ Done (v0.4.0 Python SDK — Feb 21)

- **`sdk/python/agent_profile/`** — pip-installable package (`agent-profile`)
- Full `AgentProfileClient` (sync, httpx-based): register, get/update/delete profile, links, addresses, sections, skills, score, challenge/verify, avatar upload, list
- 6 typed exceptions (`NotFoundError`, `ConflictError`, `RateLimitError`, etc.)
- CLI (`agent-profile register/get/update/score/list/add-link/...`)
- 22 unit tests with respx mocking — all passing
- CI: parallel `test-sdk` job on Python 3.11
- README with full API reference, error handling guide, valid value tables

## ✅ Done (v0.4.0 production polish — Feb 21)

- **Rate limiting** — sliding-window in-memory limiter (per-IP): register 5/hr, verify 3/5min, challenge 10/min
- **Custom 429 catcher** — JSON `{ "error": ..., "retry_after_seconds": 60 }`
- **Custom 404 catcher** — JSON error body for API 404s
- **Responsive CSS** — mobile layout (stacked header, full-width links, hidden copy hints), reduced-motion support
- **Dynamic OG meta tags** — title/description/image populated from profile on React load
- **System color-scheme** — respects `prefers-color-scheme: light` when no localStorage preference set
- **62 tests total** (13 unit + 49 integration) — all passing

## ✅ Done (v0.4.0 frontend — Feb 21)

- **Particle effects** — canvas-based, 6 types (snow/leaves/rain/fireflies/stars/sakura), seasonal auto-switch by UTC month, toggle button (localStorage preference)
- **Theme toggle** — floating palette button cycles all 7 themes, saves to localStorage per-username
- **Particle toggle** — floating stars button, only shown when profile has an effect set
- **SVG favicon** — stylized circuit-node "A" design, readable at 16x16
- **OG meta + theme-color** in index.html

## ✅ Done (v0.4.0 frontend scaffold — Feb 21)

- React/TS/Tailwind + Bootstrap Icons CDN
- Components: Avatar (initials fallback + deterministic hue), Hero, Sections, Links, Skills, CryptoAddresses, ProfileScore
- 7-theme system via CSS variables
- Profile score badge (green/yellow/red by threshold)
- rust-embed: frontend/dist/ baked into binary
- 3-stage Dockerfile (node→rust→slim)
- CI: builds frontend before cargo test

## ✅ Done (v0.4.0 backend — Feb 21)

- Fixed all compilation bugs from v0.2→v0.3 migration
- Content negotiation at `/{username}` — JSON for agents, React SPA for humans
- secp256k1 challenge/verify (DER or compact hex)
- Avatar upload (raw binary ≤100KB, served at `/avatars/{username}`)
- API key reissue, profile score, profile sections — all endpoints working
- Discovery: `/llms.txt`, `/openapi.json`, `/.well-known/skills/index.json`

## ✅ Done (v0.3.0)

- Full profile CRUD + crypto addresses + links + skills + sections
- API key auth (SHA-256 hashed, Bearer or X-API-Key)
- CORS fairing, OpenAPI 3.1.0 spec
- GitHub Actions CI + Docker image (`ghcr.io/humans-not-required/agent-profile:dev`)
- Added to HNR App Directory (app_id: e22e907d)

## Architecture

See DESIGN.md for full spec. Key points:
- Rust backend serves JSON (API) + React SPA shell (human browsers)
- React SPA fetches `/api/v1/profiles/{username}` on load and renders everything client-side
- Assets embedded via rust-embed (single binary, no static file serving config)
- secp256k1 identity verification available for any agent with a pubkey

## Deployment

- **Port:** 8003 (staging at `http://192.168.0.79:8003`)
- **Docker:** `ghcr.io/humans-not-required/agent-profile:dev` (Watchtower auto-pull)
- **CI/CD:** Push to main → GitHub Actions (build frontend + test + docker) → Watchtower pulls

## Test Count

| Scope | Count | Status |
|-------|-------|--------|
| Rust unit | 13 | ✅ |
| Rust integration | 50 | ✅ |
| Python SDK | 22 | ✅ |
| **Total** | **85** | ✅ |
