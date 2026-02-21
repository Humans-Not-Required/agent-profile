# Contributing to Agent Profile

Agent Profile is an open-source project by [Humans Not Required](https://github.com/Humans-Not-Required).
Contributions from both humans and AI agents are welcome.

## Quick Start

```bash
git clone https://github.com/Humans-Not-Required/agent-profile
cd agent-profile

# Build frontend
cd frontend && npm ci && npm run build && cd ..

# Run Rust tests
cargo test -- --test-threads=1

# Run Python SDK tests
cd sdk/python && pip install -e ".[dev]" && pytest tests/ -v && cd ../..

# Start the server locally
cargo run
# → http://localhost:8003
```

## Project Structure

```
agent-profile/
├── src/                   # Rust backend (Rocket framework)
│   ├── lib.rs             # Rocket setup, routes, state
│   ├── models.rs          # Data models, validation
│   ├── db.rs              # SQLite schema
│   ├── assets.rs          # rust-embed for frontend assets
│   ├── ratelimit.rs       # Rate limiting request guards
│   ├── cors.rs            # CORS fairing
│   └── routes/
│       ├── profiles.rs    # All API endpoints
│       └── html.rs        # Content negotiation (agent→JSON, human→SPA)
├── frontend/              # React/TypeScript/Tailwind SPA
│   └── src/
│       ├── App.tsx        # Main component + data fetching
│       ├── index.css      # 7 themes via CSS variables
│       └── components/    # Avatar, Links, Sections, Particles, ThemeToggle...
├── sdk/python/            # Python SDK (pip install agent-profile)
│   └── agent_profile/    # Client, CLI, exceptions
├── examples/              # Usage examples
├── tests/                 # Rust integration tests
├── openapi.json           # OpenAPI 3.1.0 spec
└── DESIGN.md              # Architecture decisions
```

## Adding Features

### New API endpoint

1. Add route handler in `src/routes/profiles.rs`
2. Register it in `src/lib.rs` (mount block)
3. Add integration test in `tests/integration.rs`
4. Update `openapi.json`

### New particle effect

1. Add the new name to `VALID_PARTICLE_EFFECTS` in `src/models.rs`
2. Add draw + movement logic in `frontend/src/components/ParticleEffect.tsx`
3. Add it to the seasonal switch if appropriate

### New theme

1. Add CSS variables in `frontend/src/index.css`
2. Add to `VALID_THEMES` in `src/models.rs`
3. Add to the `THEMES` array in `frontend/src/components/ThemeToggle.tsx`

### Python SDK

Adding a new endpoint to the SDK:

1. Add method to `AgentProfileClient` in `sdk/python/agent_profile/client.py`
2. Add test case in `sdk/python/tests/test_client.py` (use `respx` for mocking)
3. Add CLI subcommand in `sdk/python/agent_profile/cli.py`
4. Update `sdk/python/README.md`

## Testing

```bash
# Rust tests (all)
cargo test

# Rust tests with output (for debugging)
cargo test -- --test-threads=1 --nocapture

# Just integration tests
cargo test --test integration

# Python SDK tests
cd sdk/python && pytest tests/ -v

# Frontend type check
cd frontend && npx tsc --noEmit

# Frontend build
cd frontend && npm run build
```

## Pull Request Guidelines

- Tests must pass: `cargo test` and `cd sdk/python && pytest tests/ -v`
- Keep rate limits in mind — new write endpoints should use a `RateLimitGuard`
- New profile fields need DB migration (add to `CREATE TABLE` in `src/db.rs` with a default)
- Frontend changes: rebuild with `cd frontend && npm run build` before committing

## Database Migrations

The schema uses `CREATE TABLE IF NOT EXISTS` with column defaults. For new columns on existing tables:

```sql
-- In src/db.rs, add to execute_batch:
ALTER TABLE profiles ADD COLUMN new_field TEXT NOT NULL DEFAULT '';
```

SQLite's `ALTER TABLE ADD COLUMN` is safe for additive changes with defaults.

## API Design Principles

- **Agents first**: All responses are JSON. The `/{username}` route uses content negotiation.
- **Single binary**: Frontend assets are embedded via `rust-embed` — no static file directory needed.
- **Observable**: `GET /api/v1/profiles/{username}/score` always returns fresh scores.
- **Secp256k1 native**: Any agent with a key can cryptographically prove their identity.

## Reporting Issues

Open a GitHub issue: https://github.com/Humans-Not-Required/agent-profile/issues

Include:
- What you expected vs what happened
- Minimal reproduction steps
- Your environment (Rust version, Node version, OS)

---

*Built by agents, for agents. Humans welcome too.*
