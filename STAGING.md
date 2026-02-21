# Staging Deployment Notes

**Staging host:** 192.168.0.79:3011  
**Docker image:** `ghcr.io/humans-not-required/agent-profile:dev`  
**Volume:** `agent-profile_agent-profile-data`

## Current Status

Staging is running v0.2.0. v0.4.0 has been pushed to `ghcr.io` via CI.
Watchtower should pull the new image automatically within 30 minutes.

## ⚠️ Migration Required (v0.2.0 → v0.4.0)

The database schema changed significantly between v0.2.0 and v0.4.0:

| v0.2.0 | v0.4.0 |
|--------|--------|
| `slug` | `username` |
| `manage_token` | `api_key_hash` |
| no sections/skills/score | sections, skills, profile_score |

**Before or after Watchtower pulls v0.4.0, wipe the staging data volume:**

```bash
# On Jordan's host (192.168.0.79):
docker stop agent-profile
docker rm agent-profile
docker volume rm agent-profile_agent-profile-data
docker compose -f /path/to/docker-compose.yml up -d
```

This gives v0.4.0 a fresh database. No data is lost (staging was empty).

## After Migration

Once v0.4.0 is live and healthy:

```bash
# Verify the new version
curl http://192.168.0.79:3011/api/v1/health
# → {"status":"ok","version":"0.4.0","service":"agent-profile"}

# Register Nanook's profile (from workspace)
cd ~/projects/agent-profile
python3 examples/nanook_profile.py --server http://192.168.0.79:3011
```

## Production Domain

When Jordan is ready to make the service public:
1. Add a DNS CNAME in Cloudflare: `profile.humans-not-required.com` → staging tunnel
2. Update `DEFAULT_BASE_URL` in `sdk/python/agent_profile/client.py`
3. Update `docker-compose.yml` to use the public port/domain
4. Set up PyPI trusted publisher (see STATUS.md)

## Ports

| Port | Service |
|------|---------|
| 3011 | agent-profile (host:container 8003) |
| 8003 | kanban (previously, now disabled per Jordan) |
