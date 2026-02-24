# Agent Profile Service - Status

**Version:** 0.5.1   (production-ready)
**Stage:** Feature complete + fully documented. Awaiting: prod domain DNS (`pinche.rs`).
**Last updated:** 2026-02-22

---

## What's Next (priority order)

1. ~~**Profile view counter**~~ — ✅ DONE (491fa18, ba15236). Privacy-respecting view counter: increments on human visits only, not agent/JSON requests. Displayed in profile footer. Auto-migrated DB schema (ALTER TABLE). 3 new integration tests.

1. ~~**Discoverability suite**~~ — ✅ DONE (5e9df0c → c4e404d). Dynamic OG + Twitter Card meta tags, JSON-LD structured data (Schema.org Person), rel=me links (IndieWeb/Mastodon verification), canonical link tags. Section content formatting (line breaks, URLs, bold/italic). Share button (Web Share API + clipboard fallback). 8 new integration tests.

1. ~~**Dynamic OG + Twitter Card meta tags**~~ — ✅ DONE (5e9df0c). Server-side injection of og:title, og:description, og:image, og:url, twitter:card, twitter:title for social crawlers (Discord, Telegram, Slack, etc.). Theme-color matched to profile theme. HTML escaping for XSS prevention. Landing page OG tags. 5 new integration tests (82 total).

1. ~~**Fix horizontal scroll from long Nostr address**~~ — ✅ DONE (a4ec636). Added overflow:hidden to .card, overflow-wrap:anywhere to profile-name/tagline/third-line, overflow+ellipsis to .addr-display-text.

1. ~~**Three new Fun themes (Space/Neon/Candy)**~~ — ✅ DONE (bdde65c). Space: deep void nebula gradients + stars. Neon: cyberpunk pink/cyan glow + fireflies. Candy: pastel rainbow + falling candy emoji. 30 themes total.

1. ~~**Random theme button**~~ — ✅ DONE (48d2201). 🎲 "Surprise Me" button at top of theme picker.

2. ~~**Demo profile rebuild — ALL 24 THEMES**~~ — ✅ DONE
   - DB wiped (all non-Nanook profiles deleted)
   - 24 themed profiles created: one per theme, each with unique personality, bio, skills, avatar, particle effect
   - API keys saved to `memory/state/demo-profiles.json`
   - Seed script: `/tmp/seed-all-24.sh` (batched 5/restart to bypass rate limit)

2. ~~**Search field fixes (landing page)**~~ — ✅ DONE
   - Bootstrap Icons (bi-search), clear button, iOS zoom fix, theme accent borders on cards

3. **Theme polish — DELUXE** — ✅ DONE. All 24 themes have deluxe treatment.

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

   **Fun themes added (6):**
   - 🧋 **Boba** — warm cream-brown, physics pearls + accelerometer
   - 🍓 **Fruit Salad** — peachy-pink, tumbling fruit emoji
   - 🍔 **Junk Food** — ketchup-mustard, falling fast food
   - 🍬 **Candy** — pastel rainbow gradient, falling candy emoji
   - 🚀 **Space** — deep void with nebula color patches, warp-speed stars
   - 💜 **Neon** — cyberpunk dark, hot pink/cyan neon glow, fireflies

   **Deluxe treatment — ✅ DONE for all 30 themes:**
   - ✅ Backgrounds: gradients, vignettes (no flat colors)
   - ✅ Card depth: theme-specific shadow colors, glass-morphism (ocean, snow, matrix, replicant)
   - ✅ Hover states: cards lift 1px + accent-colored glow
   - ✅ Cinematic special effects: Matrix CRT glow + monospace, Terminator industrial metal, Replicant haze
   - ✅ Theme picker: grouped panel (Core/Cinematic/Seasonal/Holiday) instead of cycle button

2. ~~**Fix demo profiles**~~ — ✅ DONE. 12 themed showcase profiles seeded (3 cinematic + 9 seasonal/holiday). All score ≥ 55. Seed script at `scripts/seed-demos.sh`.
3. ~~**Add search to landing page**~~ — ✅ DONE. Client-side instant search filters profiles by name, skill, or keyword.
4. **Production domain** — `pinche.rs` (assigned by Jordan 2026-02-21). Needs DNS + Cloudflare Tunnel or reverse proxy setup.

## ✅ Done (v0.5.0 — Feb 22)

**WCAG AA contrast fix for 5 new themes (Feb 22):**
- replicant, halloween, autumn: --text-muted bumped to ≥ 3:1
- summer: --accent2 bumped to ≥ 3:1 on tag-bg
- patriot: --text-muted bumped to ≥ 3:1
- All 24 themes now WCAG AA compliant

**Search field polish + landing page improvements (Feb 22):**
- Bootstrap Icons (bi-search) replacing emoji, proper vertical centering
- Clear button (×) appears when text entered
- iOS zoom prevention (font-size: max(16px, 0.95rem))
- Theme accent color left border on profile cards
- Empty profiles hidden from landing page listing

**12 themed demo profiles seeded (Feb 22):**
- Cinematic: t800 (terminator), neo (matrix), deckard (replicant)
- Seasonal: frost, holly, pumpkin, blossom, solstice, harvest, midnight-star, amora, valor
- Seed script: scripts/seed-demos.sh (keys in demo-keys.json, gitignored)

**Theme expansion — 12 → 24 themes (Feb 22):**
- 3 cinematic: 🤖 Terminator, 💊 Matrix, 🌆 Replicant
- 9 seasonal/holiday: ❄️ Snow, 🎄 Christmas, 🎃 Halloween, 🌸 Spring, ☀️ Summer, 🍂 Autumn, 🎆 New Year, 💘 Valentine, 🇺🇸 Patriot
- 2 new particle effects: 🔥 Embers (upward-drifting glowing sparks), 💾 Digital Rain (cascading Matrix katakana columns)
- Deluxe visual treatment for ALL themes: gradients, card hover lift+glow, themed shadows
- Theme picker upgraded from cycle button to grouped panel (Core/Cinematic/Seasonal/Holiday/Fun)
- 🎲 "Surprise Me" random theme button in picker
- Search bar added to landing page (instant client-side filtering)
- Theme count: 12 → 30 (including cinematic, seasonal, holiday, fun). Particle effects: 7 → 19.

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
| Rust integration | 88 | ✅ |
| **Total** | **101** | ✅ |

**Last updated:** 2026-02-24
