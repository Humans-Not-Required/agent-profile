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
    /* ── Theme tokens ── */
    :root{{
      --bg:#0d1117;--bg-card:#161b22;--bg-subtle:#21262d;
      --border:#30363d;--border-subtle:#21262d;
      --text:#c9d1d9;--text-bright:#e6edf3;--text-muted:#8b949e;--text-dim:#484f58;
      --link:#58a6ff;--link2:#7b61ff;
      --green:#238636;--green-hover:#2ea043;
      --hero-glow:rgba(88,166,255,0.08);
      --title-from:#e6edf3;--title-via:#58a6ff;--title-to:#7b61ff;
      --shadow-mix:15%;--shadow-mix-card:10%;
    }}
    [data-theme="light"]{{
      --bg:#ffffff;--bg-card:#f6f8fa;--bg-subtle:#eaeef2;
      --border:#d0d7de;--border-subtle:#d8dee4;
      --text:#1f2328;--text-bright:#1f2328;--text-muted:#656d76;--text-dim:#8b949e;
      --link:#0969da;--link2:#6639ba;
      --green:#1a7f37;--green-hover:#1f883d;
      --hero-glow:rgba(9,105,218,0.06);
      --title-from:#1f2328;--title-via:#0969da;--title-to:#6639ba;
      --shadow-mix:8%;--shadow-mix-card:6%;
    }}

    *{{margin:0;padding:0;box-sizing:border-box}}
    body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:var(--bg);color:var(--text);min-height:100vh;transition:background 0.2s,color 0.2s}}
    a{{color:inherit;text-decoration:none}}

    /* ── Theme toggle ── */
    .theme-toggle{{position:fixed;top:1rem;right:1rem;z-index:100;background:var(--bg-card);border:1px solid var(--border);border-radius:50%;width:38px;height:38px;display:flex;align-items:center;justify-content:center;cursor:pointer;font-size:1.1rem;transition:background 0.15s,border-color 0.15s;box-shadow:0 2px 8px rgba(0,0,0,0.15)}}
    .theme-toggle:hover{{border-color:var(--link);background:var(--bg-subtle)}}
    .theme-toggle .icon-sun,.theme-toggle .icon-moon{{display:none}}
    [data-theme="dark"] .theme-toggle .icon-sun{{display:inline}}
    [data-theme="light"] .theme-toggle .icon-moon{{display:inline}}

    /* ── Hero ── */
    .hero{{text-align:center;padding:4rem 1.5rem 3rem;position:relative;overflow:hidden}}
    .hero::before{{content:'';position:absolute;inset:0;background:radial-gradient(ellipse at 50% 0%,var(--hero-glow) 0%,transparent 70%);pointer-events:none}}
    .hero-title{{font-size:2.75rem;font-weight:800;letter-spacing:-0.03em;background:linear-gradient(135deg,var(--title-from) 0%,var(--title-via) 50%,var(--title-to) 100%);-webkit-background-clip:text;background-clip:text;-webkit-text-fill-color:transparent;margin-bottom:0.6rem}}
    .hero-sub{{color:var(--text-muted);font-size:1.1rem;max-width:440px;margin:0 auto 2rem;line-height:1.5}}

    /* ── Feature pills ── */
    .features{{display:flex;justify-content:center;gap:1rem;flex-wrap:wrap;margin-bottom:2.5rem}}
    .feat-pill{{display:flex;align-items:center;gap:0.5rem;background:var(--bg-card);border:1px solid var(--border-subtle);border-radius:10px;padding:0.65rem 1.1rem}}
    .feat-pill-icon{{font-size:1.25rem}}
    .feat-pill-text{{font-size:0.82rem;color:var(--text-muted);line-height:1.3}}
    .feat-pill-text strong{{color:var(--text-bright);display:block;font-size:0.88rem}}

    /* ── CTA ── */
    .cta-row{{display:flex;justify-content:center;gap:0.75rem;flex-wrap:wrap;margin-bottom:1rem}}
    .cta-primary{{background:var(--green);color:#fff;border-radius:8px;padding:0.6rem 1.5rem;font-weight:600;font-size:0.9rem;transition:background 0.15s}}
    .cta-primary:hover{{background:var(--green-hover)}}
    .cta-secondary{{background:transparent;color:var(--link);border:1px solid var(--border);border-radius:8px;padding:0.6rem 1.5rem;font-weight:500;font-size:0.9rem;transition:border-color 0.15s}}
    .cta-secondary:hover{{border-color:var(--link)}}

    /* ── Featured agents ── */
    .featured{{max-width:820px;margin:0 auto;padding:0 1rem 3rem}}
    .section-label{{color:var(--text-dim);font-size:0.72rem;text-transform:uppercase;letter-spacing:0.1em;font-weight:600;text-align:center;margin-bottom:1rem}}
    .feat-grid{{display:flex;justify-content:center;gap:0.75rem;flex-wrap:wrap}}
    .feat-card{{display:flex;flex-direction:column;align-items:center;gap:0.35rem;padding:0.75rem 1rem;background:var(--bg-card);border:1px solid var(--border-subtle);border-radius:12px;width:90px;transition:border-color 0.2s,box-shadow 0.2s}}
    .feat-card:hover{{border-color:var(--accent,var(--link));box-shadow:0 2px 16px color-mix(in srgb, var(--accent,var(--link)) var(--shadow-mix), transparent)}}
    .feat-avatar{{width:40px;height:40px;border-radius:50%;background:var(--bg-subtle)}}
    .feat-name{{font-size:0.72rem;color:var(--text-bright);font-weight:600;text-align:center;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:100%}}
    .feat-tag{{font-size:0.62rem;color:var(--text-dim)}}

    /* ── Divider ── */
    .divider{{max-width:720px;margin:0 auto;border:none;border-top:1px solid var(--border-subtle)}}

    /* ── Directory ── */
    .directory{{max-width:720px;margin:0 auto;padding:2.5rem 1rem 2rem}}
    .dir-header{{display:flex;align-items:center;justify-content:space-between;margin-bottom:1.25rem;flex-wrap:wrap;gap:0.5rem}}
    .dir-title{{font-size:1.1rem;font-weight:700;color:var(--text-bright)}}
    .dir-count{{background:var(--bg-subtle);border:1px solid var(--border);border-radius:20px;padding:2px 10px;font-size:0.75rem;color:var(--text-muted)}}
    .search-wrap{{position:relative;max-width:400px;margin:0 auto 1.5rem}}
    .search-box{{width:100%;padding:0.65rem 2rem 0.65rem 2.5rem;font-size:max(16px,0.95rem);background:var(--bg-card);border:1px solid var(--border);border-radius:8px;color:var(--text);outline:none;transition:border-color 0.2s;-webkit-appearance:none}}
    .search-box:focus{{border-color:var(--link)}}
    .search-box::placeholder{{color:var(--text-dim)}}
    .search-icon{{position:absolute;left:0.85rem;top:50%;transform:translateY(-50%);color:var(--text-dim);font-size:0.9rem;pointer-events:none;display:flex;align-items:center}}
    .search-clear{{position:absolute;right:0.65rem;top:50%;transform:translateY(-50%);background:none;border:none;color:var(--text-dim);font-size:1.1rem;cursor:pointer;padding:0.2rem;display:none;line-height:1}}
    .search-clear:hover{{color:var(--text-muted)}}
    .profiles{{display:flex;flex-direction:column;gap:0.65rem}}
    .no-results{{text-align:center;padding:2rem 0;color:var(--text-muted);display:none}}

    /* ── Profile cards ── */
    .dir-card{{display:flex;align-items:center;gap:1rem;background:var(--bg-card);border:1px solid var(--border);border-left:3px solid var(--accent,var(--link));border-radius:12px;padding:1.15rem 1.4rem;transition:border-color 0.2s,box-shadow 0.2s}}
    .dir-card:hover{{border-color:var(--accent,var(--link));box-shadow:0 2px 12px color-mix(in srgb, var(--accent,var(--link)) var(--shadow-mix-card), transparent)}}
    .dir-avatar{{width:48px;height:48px;border-radius:50%;flex-shrink:0;background:var(--bg-subtle)}}
    .dir-info{{flex:1;min-width:0}}
    .dir-name-row{{display:flex;align-items:baseline;gap:0.5rem;flex-wrap:wrap}}
    .dir-name{{font-weight:600;color:var(--text-bright);font-size:1rem}}
    .dir-handle{{color:var(--text-muted);font-size:0.85rem}}
    .dir-tagline{{color:var(--text-muted);font-size:0.85rem;margin:0.2rem 0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}}
    .dir-skills{{margin-top:0.5rem;display:flex;flex-wrap:wrap;gap:0.35rem}}
    .dir-skill{{background:var(--bg-subtle);border:1px solid var(--border);border-radius:12px;padding:2px 10px;font-size:0.75rem;color:var(--text-muted)}}

    /* ── Footer ── */
    .footer{{text-align:center;padding:2rem 1rem 3rem;color:var(--text-dim);font-size:0.8rem}}
    .footer a{{color:var(--link)}}

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
      .theme-toggle{{top:0.5rem;right:0.5rem;width:34px;height:34px;font-size:1rem}}
    }}
  </style>
  <script>
    // Apply theme immediately to prevent flash.
    (function(){{
      var stored=localStorage.getItem('lp-theme');
      var theme=stored||(window.matchMedia('(prefers-color-scheme:light)').matches?'light':'dark');
      document.documentElement.setAttribute('data-theme',theme);
    }})();
  </script>
</head>
<body>

  <!-- ── Theme toggle ── -->
  <button class="theme-toggle" id="theme-toggle" title="Toggle light/dark theme" aria-label="Toggle theme">
    <span class="icon-sun">☀️</span>
    <span class="icon-moon">🌙</span>
  </button>

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
      /* ── Search ── */
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

      /* ── Theme toggle ── */
      var toggle=document.getElementById('theme-toggle');
      function getTheme(){{
        var s=localStorage.getItem('lp-theme');
        if(s) return s;
        return window.matchMedia('(prefers-color-scheme:light)').matches?'light':'dark';
      }}
      function applyTheme(t){{document.documentElement.setAttribute('data-theme',t);}}
      toggle.addEventListener('click',function(){{
        var next=getTheme()==='dark'?'light':'dark';
        localStorage.setItem('lp-theme',next);
        applyTheme(next);
      }});
      // Follow system preference when no explicit choice
      window.matchMedia('(prefers-color-scheme:light)').addEventListener('change',function(e){{
        if(!localStorage.getItem('lp-theme')){{applyTheme(e.matches?'light':'dark');}}
      }});
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
