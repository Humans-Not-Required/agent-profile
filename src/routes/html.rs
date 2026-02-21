use rocket::{get, State, http::{ContentType, Status}};

use crate::routes::profiles::{DbConn, load_profile};
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
