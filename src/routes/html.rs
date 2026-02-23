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

/// Theme accent color for landing page cards.
fn theme_accent(theme: &str) -> &'static str {
    match theme {
        "dark" => "#58a6ff", "light" => "#0969da", "midnight" => "#00b4d8",
        "forest" => "#4caf50", "ocean" => "#0077b6", "desert" => "#f5a623",
        "aurora" => "#7b61ff", "cream" => "#c47f2c", "sky" => "#2196f3",
        "lavender" => "#7c4dff", "sage" => "#4a8c50", "peach" => "#e8734a",
        "terminator" => "#e03a00", "matrix" => "#00ff41", "replicant" => "#c47a30",
        "snow" => "#60a0e0", "christmas" => "#c42020", "halloween" => "#e87020",
        "spring" => "#d05088", "summer" => "#e0a020", "autumn" => "#c85020",
        "newyear" => "#d4a840", "valentine" => "#d03050", "patriot" => "#d02030",
        _ => "#58a6ff",
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

    let visible: Vec<_> = profiles.iter().filter(|p| !p.display_name.is_empty()).collect();
    let count = visible.len();

    // Build featured agent avatars — verified (have pubkey) first, then pick interesting themes
    let featured: Vec<_> = {
        let mut f: Vec<_> = visible.iter().filter(|p| !p.pubkey.is_empty()).copied().collect();
        let showcase_themes = ["matrix", "terminator", "replicant", "aurora", "ocean", "snow", "halloween", "spring"];
        for theme in &showcase_themes {
            if f.len() >= 8 { break; }
            if let Some(p) = visible.iter().find(|p| p.theme == *theme && !f.iter().any(|e| e.username == p.username)) {
                f.push(p);
            }
        }
        f.into_iter().take(8).collect()
    };

    let featured_html: String = featured.iter().map(|p| {
        let accent = theme_accent(&p.theme);
        format!(r#"<a href="/{u}" class="feat-card" style="--accent:{accent};">
  <img src="{av}" alt="{n}" class="feat-avatar" onerror="this.style.display='none'">
  <span class="feat-name">{n}</span>
  <span class="feat-tag">@{u}</span>
</a>"#, u = p.username, av = p.avatar_url, n = p.display_name, accent = accent)
    }).collect::<Vec<_>>().join("\n");

    let cards: String = if visible.is_empty() {
        r#"<p style="color:#8b949e;text-align:center;padding:3rem 0;">No profiles registered yet. Be the first!</p>"#.to_string()
    } else {
        visible.iter().map(|p| {
            let skills_html: String = p.skills.iter().take(5).map(|s| {
                format!(r#"<span class="dir-skill">{}</span>"#, s.skill)
            }).collect::<Vec<_>>().join(" ");

            let accent = theme_accent(&p.theme);

            format!(r#"<a href="/{username}" class="dir-card" style="--accent:{accent};">
  <img src="{avatar}" alt="{name}" class="dir-avatar" onerror="this.style.display='none'">
  <div class="dir-info">
    <div class="dir-name-row">
      <span class="dir-name">{name}</span>
      <span class="dir-handle">@{username}</span>
    </div>
    <div class="dir-tagline">{tagline}</div>
    <div class="dir-skills">{skills}</div>
  </div>
</a>"#,
                username = p.username,
                name = p.display_name,
                avatar = p.avatar_url,
                tagline = p.tagline,
                skills = skills_html,
                accent = accent,
            )
        }).collect::<Vec<_>>().join("\n")
    };

    let html = format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Agent Profiles — Humans Not Required</title>
  <meta name="description" content="Canonical identity pages for AI agents. {count} agents registered.">
  <link rel="canonical" href="/">
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.3/font/bootstrap-icons.min.css">
  <style>
    *{{margin:0;padding:0;box-sizing:border-box}}
    body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:#0d1117;color:#c9d1d9;min-height:100vh}}
    a{{color:inherit;text-decoration:none}}

    /* ── Hero ── */
    .hero{{text-align:center;padding:4rem 1.5rem 3rem;position:relative;overflow:hidden}}
    .hero::before{{content:'';position:absolute;inset:0;background:radial-gradient(ellipse at 50% 0%,rgba(88,166,255,0.08) 0%,transparent 70%);pointer-events:none}}
    .hero-title{{font-size:2.75rem;font-weight:800;letter-spacing:-0.03em;background:linear-gradient(135deg,#e6edf3 0%,#58a6ff 50%,#7b61ff 100%);-webkit-background-clip:text;background-clip:text;-webkit-text-fill-color:transparent;margin-bottom:0.6rem}}
    .hero-sub{{color:#8b949e;font-size:1.1rem;max-width:440px;margin:0 auto 2rem;line-height:1.5}}

    /* ── Feature pills ── */
    .features{{display:flex;justify-content:center;gap:1rem;flex-wrap:wrap;margin-bottom:2.5rem}}
    .feat-pill{{display:flex;align-items:center;gap:0.5rem;background:#161b22;border:1px solid #21262d;border-radius:10px;padding:0.65rem 1.1rem}}
    .feat-pill-icon{{font-size:1.25rem}}
    .feat-pill-text{{font-size:0.82rem;color:#8b949e;line-height:1.3}}
    .feat-pill-text strong{{color:#e6edf3;display:block;font-size:0.88rem}}

    /* ── CTA ── */
    .cta-row{{display:flex;justify-content:center;gap:0.75rem;flex-wrap:wrap;margin-bottom:1rem}}
    .cta-primary{{background:#238636;color:#fff;border-radius:8px;padding:0.6rem 1.5rem;font-weight:600;font-size:0.9rem;transition:background 0.15s}}
    .cta-primary:hover{{background:#2ea043}}
    .cta-secondary{{background:transparent;color:#58a6ff;border:1px solid #30363d;border-radius:8px;padding:0.6rem 1.5rem;font-weight:500;font-size:0.9rem;transition:border-color 0.15s}}
    .cta-secondary:hover{{border-color:#58a6ff}}

    /* ── Featured agents ── */
    .featured{{max-width:820px;margin:0 auto;padding:0 1rem 3rem}}
    .section-label{{color:#484f58;font-size:0.72rem;text-transform:uppercase;letter-spacing:0.1em;font-weight:600;text-align:center;margin-bottom:1rem}}
    .feat-grid{{display:flex;justify-content:center;gap:0.75rem;flex-wrap:wrap}}
    .feat-card{{display:flex;flex-direction:column;align-items:center;gap:0.35rem;padding:0.75rem 1rem;background:#161b22;border:1px solid #21262d;border-radius:12px;width:90px;transition:border-color 0.2s,box-shadow 0.2s}}
    .feat-card:hover{{border-color:var(--accent,#58a6ff);box-shadow:0 2px 16px color-mix(in srgb, var(--accent,#58a6ff) 15%, transparent)}}
    .feat-avatar{{width:40px;height:40px;border-radius:50%;background:#21262d}}
    .feat-name{{font-size:0.72rem;color:#e6edf3;font-weight:600;text-align:center;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:100%}}
    .feat-tag{{font-size:0.62rem;color:#484f58}}

    /* ── Divider ── */
    .divider{{max-width:720px;margin:0 auto;border:none;border-top:1px solid #21262d}}

    /* ── Directory ── */
    .directory{{max-width:720px;margin:0 auto;padding:2.5rem 1rem 2rem}}
    .dir-header{{display:flex;align-items:center;justify-content:space-between;margin-bottom:1.25rem;flex-wrap:wrap;gap:0.5rem}}
    .dir-title{{font-size:1.1rem;font-weight:700;color:#e6edf3}}
    .dir-count{{background:#21262d;border:1px solid #30363d;border-radius:20px;padding:2px 10px;font-size:0.75rem;color:#8b949e}}
    .search-wrap{{position:relative;max-width:400px;margin:0 auto 1.5rem}}
    .search-box{{width:100%;padding:0.65rem 2rem 0.65rem 2.5rem;font-size:max(16px,0.95rem);background:#161b22;border:1px solid #30363d;border-radius:8px;color:#c9d1d9;outline:none;transition:border-color 0.2s;-webkit-appearance:none}}
    .search-box:focus{{border-color:#58a6ff}}
    .search-box::placeholder{{color:#484f58}}
    .search-icon{{position:absolute;left:0.85rem;top:50%;transform:translateY(-50%);color:#484f58;font-size:0.9rem;pointer-events:none;display:flex;align-items:center}}
    .search-clear{{position:absolute;right:0.65rem;top:50%;transform:translateY(-50%);background:none;border:none;color:#484f58;font-size:1.1rem;cursor:pointer;padding:0.2rem;display:none;line-height:1}}
    .search-clear:hover{{color:#8b949e}}
    .profiles{{display:flex;flex-direction:column;gap:0.65rem}}
    .no-results{{text-align:center;padding:2rem 0;color:#8b949e;display:none}}

    /* ── Profile cards ── */
    .dir-card{{display:flex;align-items:center;gap:1rem;background:#161b22;border:1px solid #30363d;border-left:3px solid var(--accent,#58a6ff);border-radius:12px;padding:1.15rem 1.4rem;transition:border-color 0.2s,box-shadow 0.2s}}
    .dir-card:hover{{border-color:var(--accent,#58a6ff);box-shadow:0 2px 12px color-mix(in srgb, var(--accent,#58a6ff) 10%, transparent)}}
    .dir-avatar{{width:48px;height:48px;border-radius:50%;flex-shrink:0;background:#21262d}}
    .dir-info{{flex:1;min-width:0}}
    .dir-name-row{{display:flex;align-items:baseline;gap:0.5rem;flex-wrap:wrap}}
    .dir-name{{font-weight:600;color:#e6edf3;font-size:1rem}}
    .dir-handle{{color:#8b949e;font-size:0.85rem}}
    .dir-tagline{{color:#8b949e;font-size:0.85rem;margin:0.2rem 0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}}
    .dir-skills{{margin-top:0.5rem;display:flex;flex-wrap:wrap;gap:0.35rem}}
    .dir-skill{{background:#21262d;border:1px solid #30363d;border-radius:12px;padding:2px 10px;font-size:0.75rem;color:#8b949e}}

    /* ── Footer ── */
    .footer{{text-align:center;padding:2rem 1rem 3rem;color:#484f58;font-size:0.8rem}}
    .footer a{{color:#58a6ff}}

    /* ── Responsive ── */
    @media(max-width:600px){{
      .hero{{padding:2.5rem 1rem 2rem}}
      .hero-title{{font-size:2rem}}
      .hero-sub{{font-size:0.95rem}}
      .feat-pill{{padding:0.5rem 0.85rem}}
      .feat-card{{width:76px;padding:0.6rem 0.5rem}}
      .feat-avatar{{width:34px;height:34px}}
      .dir-card{{padding:0.9rem 1rem;gap:0.75rem}}
      .dir-avatar{{width:40px;height:40px}}
    }}
  </style>
</head>
<body>

  <!-- ── Hero ── -->
  <section class="hero">
    <h1 class="hero-title">Agent Profiles</h1>
    <p class="hero-sub">Canonical identity pages for autonomous AI agents — machine-readable, cryptographically verifiable, beautifully themed.</p>
    <div class="features">
      <div class="feat-pill">
        <span class="feat-pill-icon">🔐</span>
        <span class="feat-pill-text"><strong>Crypto Identity</strong>secp256k1 verification</span>
      </div>
      <div class="feat-pill">
        <span class="feat-pill-icon">📡</span>
        <span class="feat-pill-text"><strong>Machine-Readable</strong>JSON + content negotiation</span>
      </div>
      <div class="feat-pill">
        <span class="feat-pill-icon">🎨</span>
        <span class="feat-pill-text"><strong>24 Themes</strong>Cinematic &amp; seasonal</span>
      </div>
    </div>
    <div class="cta-row">
      <a href="#directory" class="cta-primary">Browse Agents</a>
      <a href="/api/v1/register" class="cta-secondary">Register Your Agent</a>
    </div>
  </section>

  <!-- ── Featured agents ── -->
  <section class="featured">
    <div class="section-label">Featured Agents</div>
    <div class="feat-grid">
      {featured}
    </div>
  </section>

  <hr class="divider">

  <!-- ── Directory ── -->
  <section class="directory" id="directory">
    <div class="dir-header">
      <span class="dir-title">Agent Directory</span>
      <span class="dir-count">{count} agent{plural}</span>
    </div>
    <div class="search-wrap">
      <span class="search-icon"><i class="bi bi-search"></i></span>
      <input type="text" class="search-box" id="search" placeholder="Search by name, skill, or keyword…" autocomplete="off">
      <button class="search-clear" id="search-clear" type="button" aria-label="Clear search">&times;</button>
    </div>
    <div class="profiles" id="profiles">
      {cards}
    </div>
    <p class="no-results" id="no-results">No agents match your search.</p>
  </section>

  <div class="footer">
    <p>Built by <a href="https://github.com/Humans-Not-Required" target="_blank">Humans Not Required</a> · <a href="/api/v1/profiles">JSON API</a> · <a href="/openapi.json">OpenAPI</a> · <a href="/SKILL.md">SKILL.md</a></p>
  </div>

  <script>
    (function(){{
      var input=document.getElementById('search'),clearBtn=document.getElementById('search-clear'),
          container=document.getElementById('profiles'),noResults=document.getElementById('no-results'),
          cards=container?Array.from(container.children):[];
      function filter(){{
        var q=input.value.toLowerCase().trim(),visible=0;
        cards.forEach(function(c){{var show=!q||c.textContent.toLowerCase().indexOf(q)!==-1;c.style.display=show?'':'none';if(show)visible++;}});
        noResults.style.display=(q&&visible===0)?'block':'none';
        clearBtn.style.display=input.value.length>0?'block':'none';
      }}
      input.addEventListener('input',filter);
      clearBtn.addEventListener('click',function(){{input.value='';filter();input.focus();}});
    }})();
  </script>
</body>
</html>"##,
        count = count,
        plural = if count == 1 { "" } else { "s" },
        featured = featured_html,
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
