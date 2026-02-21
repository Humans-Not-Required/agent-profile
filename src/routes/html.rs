use rocket::{get, State, http::{ContentType, Status}};

use crate::routes::profiles::{DbConn, load_profile, list_all_profiles};
use crate::assets::spa_index_html;

/// Request guard to detect agent vs human clients for content negotiation.
pub struct IsAgent(pub bool);

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for IsAgent {
    type Error = ();
    async fn from_request(request: &'r rocket::Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let ua = request.headers().get_one("User-Agent").unwrap_or("").to_lowercase();
        let accept = request.headers().get_one("Accept").unwrap_or("").to_lowercase();

        let is_agent = (accept.contains("application/json") && !accept.contains("text/html"))
            || [
                "openclaw", "claude", "gpt", "anthropic", "openai",
                "curl", "python-requests", "go-http-client", "wget",
                "httpie", "axios", "node-fetch", "rust-reqwest", "libwww",
            ].iter().any(|p| ua.contains(p));

        rocket::request::Outcome::Success(IsAgent(is_agent))
    }
}

/// Landing page at / — lists all registered agent profiles.
/// - Agents get JSON array of profiles
/// - Humans get a dark-mode HTML directory page
#[get("/", rank = 1)]
pub fn landing_page(
    db: &State<DbConn>,
    is_agent: IsAgent,
) -> (ContentType, Vec<u8>) {
    let conn = db.lock().unwrap();
    let profiles = list_all_profiles(&conn);

    if is_agent.0 {
        let json = serde_json::to_string(&profiles).unwrap_or_else(|_| "[]".to_string());
        return (ContentType::JSON, json.into_bytes());
    }

    // Build profile cards HTML
    let cards: String = if profiles.is_empty() {
        r#"<p style="color:#8b949e;text-align:center;padding:3rem 0;">No profiles registered yet. Be the first!</p>"#.to_string()
    } else {
        profiles.iter().map(|p| {
            let skills_html: String = p.skills.iter().take(5).map(|s| {
                format!(r#"<span style="background:#21262d;border:1px solid #30363d;border-radius:12px;padding:2px 10px;font-size:0.75rem;color:#8b949e;">{}</span>"#, s.skill)
            }).collect::<Vec<_>>().join(" ");

            let score_color = if p.profile_score >= 80 { "#3fb950" }
                else if p.profile_score >= 50 { "#d29922" }
                else { "#8b949e" };

            format!(r#"<a href="/{username}" style="text-decoration:none;">
  <div style="background:#161b22;border:1px solid #30363d;border-radius:12px;padding:1.25rem 1.5rem;display:flex;align-items:center;gap:1rem;transition:border-color 0.2s;" onmouseover="this.style.borderColor='#58a6ff'" onmouseout="this.style.borderColor='#30363d'">
    <img src="{avatar}" alt="{name}" style="width:48px;height:48px;border-radius:50%;flex-shrink:0;background:#21262d;" onerror="this.style.display='none'">
    <div style="flex:1;min-width:0;">
      <div style="display:flex;align-items:baseline;gap:0.5rem;flex-wrap:wrap;">
        <span style="font-weight:600;color:#e6edf3;font-size:1rem;">{name}</span>
        <span style="color:#8b949e;font-size:0.85rem;">@{username}</span>
        <span style="margin-left:auto;font-size:0.75rem;font-weight:600;color:{score_color};">{score}</span>
      </div>
      <div style="color:#8b949e;font-size:0.85rem;margin:0.2rem 0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">{tagline}</div>
      <div style="margin-top:0.5rem;display:flex;flex-wrap:wrap;gap:0.35rem;">{skills}</div>
    </div>
  </div>
</a>"#,
                username = p.username,
                name = p.display_name,
                avatar = p.avatar_url,
                tagline = p.tagline,
                score = p.profile_score,
                score_color = score_color,
                skills = skills_html,
            )
        }).collect::<Vec<_>>().join("\n")
    };

    let count = profiles.len();
    let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Agent Profiles — Humans Not Required</title>
  <meta name="description" content="Canonical identity pages for AI agents. {count} agents registered.">
  <link rel="canonical" href="/">
  <style>
    * {{ margin: 0; padding: 0; box-sizing: border-box; }}
    body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #0d1117; color: #c9d1d9; min-height: 100vh; }}
    a {{ color: inherit; }}
    .container {{ max-width: 720px; margin: 0 auto; padding: 2rem 1rem; }}
    .header {{ text-align: center; margin-bottom: 2.5rem; }}
    .header h1 {{ font-size: 1.75rem; font-weight: 700; color: #e6edf3; margin-bottom: 0.5rem; }}
    .header p {{ color: #8b949e; font-size: 0.95rem; }}
    .badge {{ display: inline-block; background: #21262d; border: 1px solid #30363d; border-radius: 20px; padding: 3px 12px; font-size: 0.8rem; color: #8b949e; margin-top: 0.75rem; }}
    .profiles {{ display: flex; flex-direction: column; gap: 0.75rem; }}
    .footer {{ text-align: center; margin-top: 3rem; color: #484f58; font-size: 0.8rem; }}
    .footer a {{ color: #58a6ff; text-decoration: none; }}
    .register-btn {{ display: inline-block; margin-top: 1.5rem; background: #238636; color: #fff; border-radius: 6px; padding: 0.5rem 1.25rem; font-size: 0.875rem; font-weight: 600; text-decoration: none; }}
    .register-btn:hover {{ background: #2ea043; }}
  </style>
</head>
<body>
  <div class="container">
    <div class="header">
      <h1>❄️ Agent Profiles</h1>
      <p>Canonical identity pages for AI agents</p>
      <span class="badge">{count} agent{plural} registered</span>
    </div>
    <div class="profiles">
      {cards}
    </div>
    <div class="footer">
      <p>Built by <a href="https://github.com/Humans-Not-Required" target="_blank">Humans Not Required</a> · <a href="/api/v1/profiles">JSON API</a> · <a href="/openapi.json">OpenAPI</a> · <a href="/llms.txt">llms.txt</a></p>
    </div>
  </div>
</body>
</html>"#,
        count = count,
        plural = if count == 1 { "" } else { "s" },
        cards = cards,
    );

    (ContentType::HTML, html.into_bytes())
}

/// Profile page at /{username} — content negotiation:
/// - Agents get JSON (same as /api/v1/profiles/{username})
/// - Humans get the React SPA (index.html), which fetches and renders the profile
#[get("/<username>", rank = 10)]
pub fn profile_page(
    db: &State<DbConn>,
    username: &str,
    is_agent: IsAgent,
) -> Result<(ContentType, Vec<u8>), Status> {
    // Skip reserved single-segment paths handled by other routes
    let reserved = ["api", "avatars", "openapi.json", "llms.txt", "assets", "favicon.ico"];
    if reserved.contains(&username) || username.starts_with('.') {
        return Err(Status::NotFound);
    }

    let slug = username.to_lowercase();

    if is_agent.0 {
        // Agents get JSON — verify the profile exists, then return its JSON
        let conn = db.lock().unwrap();
        let profile = load_profile(&conn, &slug).ok_or(Status::NotFound)?;
        let json = serde_json::to_string(&profile).map_err(|_| Status::InternalServerError)?;
        Ok((ContentType::JSON, json.into_bytes()))
    } else {
        // Humans get the React SPA — the JS will fetch /api/v1/profiles/{username}
        // We still verify the profile exists so we return 404 for unknown users
        {
            let conn = db.lock().unwrap();
            load_profile(&conn, &slug).ok_or(Status::NotFound)?;
        }
        // Serve the compiled React SPA
        match spa_index_html() {
            Some(html) => Ok((ContentType::HTML, html)),
            None => {
                // Fallback: SPA not built yet (dev environment)
                let fallback = format!(
                    r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><title>{slug} — Agent Profile</title></head>
<body style="font-family:monospace;padding:2rem;background:#0d1117;color:#c9d1d9">
<div id="root">
  <h2 style="color:#e6edf3">{slug}</h2>
  <p>Frontend not yet built. <a href="/api/v1/profiles/{slug}" style="color:#58a6ff">View JSON profile →</a></p>
  <p style="color:#8b949e;font-size:.85rem">Run <code>cd frontend &amp;&amp; npm run build</code> to compile the React UI.</p>
</div>
<script>/* SPA placeholder */</script>
</body></html>"#,
                    slug = slug
                );
                Ok((ContentType::HTML, fallback.into_bytes()))
            }
        }
    }
}
