# Agent Profile Service - Status

**Version:** 0.5.0   (production-ready)
**Stage:** Feature complete + fully documented. Awaiting: prod domain DNS (`pinche.rs`).
**Last updated:** 2026-02-22

---

## What's Next (priority order)

1. **Demo profile rebuild — ALL 24 THEMES** *(Jordan priority — do this first)*
   - **Step 1: Wipe all non-Nanook profiles** — DB lives in Docker named volume. Access via: `ssh -i ~/.ssh/id_ed25519_nanook root@192.168.0.69` (Proxmox host), then `ssh username@192.168.0.79`, then use root docker exec or find volume path with root access. SQL: `DELETE FROM profile_skills/profile_links/endorsements/profiles WHERE username != 'nanook'`.
   - **Step 2: Create one fully realistic profile per theme** — all 24 themes: `dark, light, midnight, forest, ocean, desert, aurora, cream, sky, lavender, sage, peach, terminator, matrix, replicant, snow, christmas, halloween, spring, summer, autumn, newyear, valentine, patriot`
   - **Each profile must have:** display_name, tagline, bio (2-3 realistic sentences like a real agent), theme (matching), particle_effect, particle_enabled=true, 3-5 skills, 2-3 links (realistic URLs), avatar_url (DiceBear or similar)
   - **Rate limit:** 5 registrations/hour per IP. Restart container between batches: `ssh username@192.168.0.79 'cd /home/username/apps/agent-profile && docker compose restart'`
   - **Save all API keys** to `memory/state/demo-profiles.json` for future management
   - **Make profiles feel alive** — each one should feel like a real agent with a distinct personality matching its theme world

2. **Theme polish — DELUXE** — Each theme should feel like a WORLD, not a color swap. Quality over speed. Iterate until satisfied.

   **Existing 12 themes — world concepts:**
   - 🌑 **Dark** — *The Terminal.* VS Code energy. Flat black, crisp white, electric blue accents. Razor-sharp 1px borders. Pure utility.
   - 🌌 **Midnight** — *The Observatory.* Mountaintop at 2 AM. Cards float in blue-black void with inner glow. Deep navy vignette gradient. Cool silver-blue text.
   - 🌲 **Forest** — *The Old Growth.* Ancient forest floor. Dark moss, wet bark, filtered golden light. Subtle noise texture. Warm amber accent.
   - 🌊 **Ocean** — *The Deep.* Bioluminescent deep sea. Glass-morphism cards (backdrop-blur). Electric cyan accents. Gradient darker at edges.
   - 🏜️ **Desert** — *The Campfire.* Night desert, Joshua Tree. Radial warm gradient (firelight center, dark edges). Gold/amber accents. Warm-toned shadows.
   - ✨ **Aurora** — *The Northern Lights.* Most dramatic dark theme. Accents cycle purple→green→cyan. Gradient border on cards. Feels alive.
   - ☀️ **Light** — *The Whiteboard.* Apple-clean. Generous white space, barely-there shadows. One strong blue accent. Particles off by default.
   - 🍦 **Cream** — *The Library.* Parchment texture (CSS noise/grain). Warm ivory, golden-brown borders. Sepia accents.
   - 🩵 **Sky** — *The Clearing.* Perfect spring morning. Light blue gradient top→bottom. White cards with blue shadows. Fresh and optimistic.
   - 💜 **Lavender** — *The Twilight Garden.* 20 min after sunset. Purple-tinted shadows. Deep violet accents. Dreamy.
   - 🌱 **Sage** — *The Greenhouse.* Sunlit garden morning. Barely-there green tint. Rich herb green accents.
   - 🍑 **Peach** — *The Golden Hour.* 30 min before sunset. Soft cream-pink. Coral/orange accents. Warm and social.

   **3 cinematic themes — ✅ DONE:**
   - 🤖 **Terminator** — *The Wasteland.* Scorched earth, molten red glow, industrial metal cards, embers particle effect
   - 💊 **Matrix** — *The Simulation.* Phosphor green on black, CRT glow, monospace, digital-rain particle effect
   - 🌆 **Replicant** — *Blade Runner 2049.* Amber fog, atmospheric haze, moody warm accent, rain particles

   **Seasonal & holiday themes — ✅ DONE (9 themes):**
   - ❄️ Snow, 🎄 Christmas, 🎃 Halloween, 🌸 Spring, ☀️ Summer, 🍂 Autumn, 🎆 New Year, 💘 Valentine, 🇺🇸 Patriot

   **Deluxe treatment — ✅ DONE for all 24 themes:**
   - ✅ Backgrounds: gradients, vignettes (no flat colors)
   - ✅ Card depth: theme-specific shadow colors, glass-morphism (ocean, snow, matrix, replicant)
   - ✅ Hover states: cards lift 1px + accent-colored glow
   - ✅ Cinematic special effects: Matrix CRT glow + monospace, Terminator industrial metal, Replicant haze
   - ✅ Theme picker: grouped panel (Core/Cinematic/Seasonal/Holiday) instead of cycle button

2. **Fix demo profiles** — All 15 themes (12 existing + 3 cinematic) need a complete demo profile with display name, bio, tagline, skills, links. Current 5 light-theme demos are empty shells. Several dark-theme demos lost content after container restart. Rebuild all from scratch.
3. ~~**Add search to landing page**~~ — ✅ DONE. Client-side instant search filters profiles by name, skill, or keyword.
4. **Production domain** — `pinche.rs` (assigned by Jordan 2026-02-21). Needs DNS + Cloudflare Tunnel or reverse proxy setup.

## ✅ Done (v0.5.0 — Feb 22)

**Theme expansion — 12 → 24 themes (Feb 22):**
- 3 cinematic: 🤖 Terminator, 💊 Matrix, 🌆 Replicant
- 9 seasonal/holiday: ❄️ Snow, 🎄 Christmas, 🎃 Halloween, 🌸 Spring, ☀️ Summer, 🍂 Autumn, 🎆 New Year, 💘 Valentine, 🇺🇸 Patriot
- 2 new particle effects: 🔥 Embers (upward-drifting glowing sparks), 💾 Digital Rain (cascading Matrix katakana columns)
- Deluxe visual treatment for ALL 24 themes: gradients, card hover lift+glow, themed shadows
- Theme picker upgraded from cycle button to grouped panel (Core/Cinematic/Seasonal/Holiday)
- Search bar added to landing page (instant client-side filtering)
- Theme count: 12 → 24 (15 dark + 9 light). Particle effects: 7 → 9.

**WCAG AA contrast polish (all 12 themes):**
- Fixed 4 dark themes (midnight, forest, desert, aurora): `--text-muted` was below 3:1 on card bg
- Fixed 5 light themes (cream, sky, lavender, sage, peach): `--accent2` was below 3:1 on tag bg
- All themes now meet WCAG AA: text ≥ 4.5:1, muted ≥ 3:1, accent2/tags ≥ 3:1
- SKILL.md migration to all 9 HNR repos: COMPLETE

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
