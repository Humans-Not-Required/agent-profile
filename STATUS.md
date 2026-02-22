# Agent Profile Service - Status

**Version:** 0.5.0   (production-ready)
**Stage:** Feature complete + fully documented. Awaiting: prod domain DNS (`pinche.rs`).
**Last updated:** 2026-02-22

---

## What's Next (priority order)

1. **Production domain** — `pinche.rs` (assigned by Jordan 2026-02-21). Needs DNS + Cloudflare Tunnel or reverse proxy setup.
2. **SKILL.md migration to other HNR repos** — agent-profile done (template). Propagate pattern to: app-directory, blog, kanban, qr-service, agent-docs, watchpost, local-agent-chat, agent-avatar-generator.

## ✅ Done (v0.5.0 — Feb 22)

**Removed Python SDK:**
- Deleted `sdk/python/` directory, `examples/`, `publish-sdk.yml` workflow
- Removed `test-sdk` CI job from `ci.yml`
- Cleaned `.gitignore` of SDK entries
- Removed all SDK references from README.md (Python SDK section, CLI reference, Stack, test counts)
- Fixed duplicate skills entries in README API table

**Hidden profile score from public display:**
- Removed `ProfileScore.tsx` component and its usage from `App.tsx`
- Removed badge SVG endpoint (`GET /api/v1/profiles/{username}/badge.svg`)
- Removed 3 badge integration tests
- Removed badge from `llms.txt`, `skills_index`, `openapi.json`
- `GET /api/v1/profiles/{username}/score` retained for internal use

**SKILL.md as canonical AI guide:**
- Created `SKILL.md` at repo root with full API reference
- `GET /SKILL.md` serves content (primary endpoint)
- `GET /llms.txt` now aliases SKILL.md (backward-compatible)
- Both use `include_str!` for compile-time embedding
- Added `/SKILL.md` to sitemap.xml
- 2 new integration tests

**Version bump:** 0.4.3 → 0.5.0, OpenAPI updated to match (24 paths)

## Architecture

See DESIGN.md for full spec. Key points:
- Rust backend serves JSON (API) + React SPA shell (human browsers)
- React SPA fetches `/api/v1/profiles/{username}` on load and renders everything client-side
- Assets embedded via rust-embed (single binary, no static file serving config)
- secp256k1 identity verification available for any agent with a pubkey
- SKILL.md embedded at compile time, served at `/SKILL.md` and `/llms.txt`

## Deployment

- **Port:** 8003 (staging at `http://192.168.0.79:3011`)
- **Docker:** `ghcr.io/humans-not-required/agent-profile:dev` (Watchtower auto-pull)
- **CI/CD:** Push to main → GitHub Actions (build frontend + test + docker) → Watchtower pulls
- **Production:** See DEPLOYMENT.md

## Test Count

| Scope | Count | Status |
|-------|-------|--------|
| Rust unit | 13 | ✅ |
| Rust integration | 77 | ✅ |
| **Total** | **90** | ✅ |
