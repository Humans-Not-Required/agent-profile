# Agent Profile Service - Status

**Version:** 0.5.0   (production-ready)
**Stage:** Feature complete + fully documented. Awaiting: prod domain DNS (`pinche.rs`).
**Last updated:** 2026-02-22

---

## What's Next (priority order)

1. **Theme polish — DELUXE** — Each theme should feel like a WORLD, not a color swap. Quality over speed. Iterate until satisfied.

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

   **NEW seasonal & holiday themes to build:**
   - ❄️ **Snow** — *The Blizzard.* Pure winter. Deep blue-white palette, cards like frosted glass (icy blue tint, blur effect). Crisp silver-white accents. Snow particles always on. Could double as a winter base for holiday variants.
   - 🎄 **Christmas** — *The Fireplace.* Cozy holiday warmth. Deep forest green + rich crimson, gold accent. Cards feel like wrapped gifts — subtle warm glow. Snow particles. Could include subtle red/green gradient border.
   - 🎃 **Halloween** — *The Haunting.* Deep purple-black, pumpkin orange accents. Cards like tombstones — slightly rounded with eerie glow. Fireflies or custom "ember" particles. Creepy but beautiful.
   - 🌸 **Spring** — *The Blossom.* Fresh pinks, soft greens, light lavender. Cards feel airy. Sakura particles mandatory. Optimistic and alive.
   - ☀️ **Summer** — *The Solstice.* Bright, saturated. Warm golden yellows, turquoise accents. Cards feel sun-bleached. Leaves or firefly particles at dusk.
   - 🍂 **Autumn** — *The Harvest.* Burnt orange, amber, deep red, brown. Cards feel like fallen leaves — warm, textured. Falling leaves particles.
   - 🎆 **New Year** — *The Countdown.* Midnight black with gold and silver. Sparkle/star particles (like fireworks). Glamorous, celebratory.
   - 💘 **Valentine** — *The Love Letter.* Soft rose, deep red, cream. Cards feel like stationery. Sakura or heart-adjacent particles. Warm and romantic.
   - 🇺🇸 **Patriot** — *The Fourth.* Red, white, deep blue. Star particles like fireworks. Clean, bold, American.

   **Deluxe treatment (ALL themes):**
   - Backgrounds: gradients, vignettes, noise textures — no flat colors
   - Card depth: theme-specific shadow colors, glass-morphism where fitting
   - Hover states: cards lift + glow in accent color
   - Border personality: distinct per theme (sharp/soft/glowing/minimal)
   - Accent consistency: one signature color through links, tags, hover, particles

2. **Fix demo profiles** — All 15 themes (12 existing + 3 cinematic) need a complete demo profile with display name, bio, tagline, skills, links. Current 5 light-theme demos are empty shells. Several dark-theme demos lost content after container restart. Rebuild all from scratch.
3. **Add search to landing page** — The API supports `?q=` search but the frontend landing page (`/`) has no search UI. Add a search bar so visitors can find profiles by name, skill, or keyword.
4. **Production domain** — `pinche.rs` (assigned by Jordan 2026-02-21). Needs DNS + Cloudflare Tunnel or reverse proxy setup.

## ✅ Done (v0.5.0 — Feb 22)

**3 Cinematic themes + 2 particle effects (Feb 22):**
- 🤖 **Terminator** — Scorched earth palette, molten red accents, radial fire gradient bg, industrial metal card styling with inner glow
- 💊 **Matrix** — Phosphor green (#00ff41) on black, CRT glow text-shadows, monospace font override, glass-morphism cards
- 🌆 **Replicant** — Blade Runner 2049 amber fog over blue-gray, atmospheric backdrop-filter, muted warm palette
- 🔥 **Embers particle** — Glowing orange-red sparks drifting upward with flickering radial glow halos
- 💾 **Digital Rain particle** — Cascading Matrix-style katakana + hex character columns with bright head, fading green trail
- Theme count: 12 → 15 (9 dark + 6 light). Particle effects: 7 → 9.
- Updated: SKILL.md, openapi.json, ThemeToggle, ParticleEffect, models.rs

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
