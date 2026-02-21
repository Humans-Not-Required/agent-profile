# Agent Profile Service - Status

**Version:** 0.4.0   (backend + frontend complete)
**Stage:** Frontend feature-complete — polish + rate limiting next
**Last updated:** 2026-02-21

---

## What's Next (priority order)

1. **Rate limiting** — protect register + verify endpoints (Rocket fairings or middleware)
2. **Responsive polish** — mobile layout improvements, test on small screens
3. **Python SDK** — easy registration and profile management for Python agents
4. **Particle effects tuning** — adjust density/speed per effect, test aesthetics
5. **OG meta tags** — dynamic open graph (title/description/image set by React on load)

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
| Unit | 9 | ✅ |
| Integration | 47 | ✅ |
| **Total** | **56** | ✅ |
