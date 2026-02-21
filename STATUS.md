# Agent Profile Service - Status

**Version:** 0.4.0   (production-ready)
**Stage:** Feature complete. Endorsements + skill directory + stats. 120 tests. Awaiting Jordan: PyPI + prod domain.
**Last updated:** 2026-02-21

---

## What's Next (priority order)

1. **PyPI publish** — CI workflow ready (`.github/workflows/publish-sdk.yml`). Jordan: set up OIDC trusted publisher at pypi.org, then `git tag sdk-v0.1.0 && git push origin sdk-v0.1.0`
2. **Production domain** — wait for Jordan's signal on public DNS
3. ~~OpenAPI spec update~~ — ✅ done (19 paths)

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

## ✅ Done (Skill directory + stats endpoints — Feb 21)

- `GET /api/v1/skills` — ecosystem skill taxonomy sorted by usage count; `?q=` substring search; case-normalized
- `GET /api/v1/stats` — aggregate counts: profiles (total/with_pubkey/avg_score), skills (total/distinct/top-5), endorsements (total/verified), links, addresses
- Python SDK: `list_skills()`, `get_stats()`, plus `skills` and `stats` CLI commands
- OpenAPI: 21 total paths
- 5+3 = 8 new tests → 120 total (13 unit + 69 integration + 38 Python SDK)

## ✅ Done (Skill-based profile search + has_pubkey filter — Feb 21)

- `GET /api/v1/profiles?skill=Rust` — find agents by capability (case-insensitive, subquery into profile_skills)
- `GET /api/v1/profiles?has_pubkey=true` — find crypto-identity-enabled agents
- Compose with existing `?q=` and `?theme=` filters (all AND-combined)
- Python SDK: `list_profiles(skill=..., has_pubkey=...)` added
- OpenAPI: both params documented on GET /profiles
- README: search section updated, API table completed (skills + endorsements were missing)
- 5+3 = 8 new tests → 112 total (13 unit + 64 integration + 35 Python SDK)

## ✅ Done (Python SDK endorsement methods — Feb 21)

- `client.get_endorsements(username)` — list endorsements (public)
- `client.add_endorsement(username, from_username, api_key, message, signature=None)` — full docstring, upsert semantics, optional signature
- `client.delete_endorsement(username, endorser_username, api_key)` — either party can delete
- CLI: `endorsements`, `endorse`, `delete-endorsement` subcommands with verified badge display
- 10 new SDK unit tests → 32 total SDK tests
- Total: **104 tests** (13 unit + 59 integration + 32 Python SDK)

## ✅ Done (OpenAPI spec — 19 paths — Feb 21)

- Added endorsement endpoints to openapi.json: POST/GET `/endorsements`, DELETE `/endorsements/{endorser}`
- Added `Endorsement`, `AddEndorsementRequest`, `EndorsementListResponse` schemas
- Profile schema updated with `endorsements[]` array
- Endorsements tag added to tag list
- 19 total paths (was 17)
- JSON validated, integration test `test_openapi_json` passes

## ✅ Done (Endorsements/Attestations — Feb 21)

- **Social trust layer** — any registered agent can endorse another
- **Endpoints:** POST/GET/DELETE `/api/v1/profiles/{username}/endorsements`
- **Auth:** Endorser must use their own API key (can't forge endorsements)
- **Self-endorse guard:** 422 if `from == target`
- **Upsert semantics:** Re-endorsing updates the message rather than creating a duplicate
- **Cryptographic attestation (opt-in):** If endorser has a secp256k1 pubkey, they can sign the message; server verifies and marks `verified: true`
- **Profile JSON:** `endorsements[]` array included in `GET /api/v1/profiles/{username}`
- **Mutual delete:** Either the endorser OR the endorsee can remove an endorsement
- **Frontend:** `Endorsements.tsx` — avatar initials, verified badge (🏅), time-ago, links to endorser profiles
- **9 new integration tests** → 94 total (13 unit + 59 integration + 22 Python SDK)
- **skills/index.json** updated with `endorse-agent` skill

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
- 32 unit tests with respx mocking — all passing (includes 10 endorsement tests)
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
| Rust integration | 69 | ✅ |
| Python SDK | 38 | ✅ |
| **Total** | **120** | ✅ |
