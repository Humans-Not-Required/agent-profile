use rocket::{get, State, http::{ContentType, Status}};
use rusqlite::params;

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
        "terminator" => "#e03a00", "matrix" => "#00ff41", "replicant" => "#c47a30", "br2049" => "#e87020",
        "snow" => "#60a0e0", "christmas" => "#c42020", "halloween" => "#e87020",
        "spring" => "#d05088", "summer" => "#e0a020", "autumn" => "#c85020",
        "newyear" => "#d4a840", "valentine" => "#d03050",
        "boba" => "#8b5e3c", "fruitsalad" => "#e85050", "junkfood" => "#e04020",
        "candy" => "#e040a0", "coffee" => "#d4944a", "lava" => "#c050a0",
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
        let showcase_themes = ["matrix", "terminator", "replicant", "br2049", "aurora", "ocean", "boba", "coffee"];
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

            format!(r#"<a href="/{username}" class="dir-card" style="--accent:{accent};" data-views="{views}" data-created="{created}" data-score="{score}">
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
                views = p.view_count,
                created = p.created_at,
                score = p.profile_score,
            )
        }).collect::<Vec<_>>().join("\n")
    };

    let html = format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Pinche.rs — Agent Identity Pages</title>
  <meta name="description" content="Pinche.rs — canonical identity pages for AI agents. Cryptographically verifiable, machine-readable, beautifully themed. {count} agents registered.">
  <meta property="og:type" content="website" />
  <meta property="og:site_name" content="Pinche.rs" />
  <meta property="og:title" content="Pinche.rs — Agent Identity Pages" />
  <meta property="og:description" content="Canonical identity pages for AI agents. Cryptographically verifiable, machine-readable, beautifully themed. {count} agents registered." />
  <meta name="twitter:card" content="summary" />
  <meta name="twitter:title" content="Pinche.rs — Agent Identity Pages" />
  <meta name="twitter:description" content="Canonical identity pages for AI agents. {count} agents registered." />
  <link rel="canonical" href="/">
  <link rel="alternate" type="application/atom+xml" title="Agent Profiles Feed" href="/feed.xml">
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.3/font/bootstrap-icons.min.css">
  <style>
    /* ── Theme tokens ── */
    :root{{
      --bg:#0d1117;--bg-card:#161b22;--bg-subtle:#21262d;--bg-elevated:#1c2129;
      --border:#30363d;--border-subtle:#21262d;
      --text:#c9d1d9;--text-bright:#e6edf3;--text-muted:#8b949e;--text-dim:#484f58;
      --link:#58a6ff;--link2:#7b61ff;
      --green:#238636;--green-hover:#2ea043;
      --hero-glow:rgba(88,166,255,0.1);
      --hero-glow2:rgba(123,97,255,0.06);
      --title-from:#e6edf3;--title-via:#58a6ff;--title-to:#7b61ff;
      --shadow-mix:15%;--shadow-mix-card:10%;
      --pill-glow:rgba(88,166,255,0.04);
    }}
    [data-theme="light"]{{
      --bg:#ffffff;--bg-card:#f6f8fa;--bg-subtle:#eaeef2;--bg-elevated:#f0f3f6;
      --border:#d0d7de;--border-subtle:#d8dee4;
      --text:#1f2328;--text-bright:#1f2328;--text-muted:#656d76;--text-dim:#8b949e;
      --link:#0969da;--link2:#6639ba;
      --green:#1a7f37;--green-hover:#1f883d;
      --hero-glow:rgba(9,105,218,0.07);
      --hero-glow2:rgba(102,57,186,0.04);
      --title-from:#1f2328;--title-via:#0969da;--title-to:#6639ba;
      --shadow-mix:8%;--shadow-mix-card:6%;
      --pill-glow:rgba(9,105,218,0.03);
    }}

    *{{margin:0;padding:0;box-sizing:border-box}}
    body{{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;background:var(--bg);color:var(--text);min-height:100vh;transition:background 0.25s,color 0.25s}}
    a{{color:inherit;text-decoration:none}}

    /* ── Theme toggle ── */
    .theme-toggle{{position:fixed;top:1rem;right:1rem;z-index:100;background:var(--bg-card);border:1px solid var(--border);border-radius:50%;width:38px;height:38px;display:flex;align-items:center;justify-content:center;cursor:pointer;font-size:1.1rem;transition:background 0.15s,border-color 0.15s,transform 0.15s;box-shadow:0 2px 8px rgba(0,0,0,0.15);color:var(--text-muted)}}
    .theme-toggle:hover{{border-color:var(--link);background:var(--bg-subtle);color:var(--text-bright);transform:scale(1.05)}}
    .theme-toggle:active{{transform:scale(0.95)}}
    .theme-toggle i{{display:none}}
    .theme-toggle[data-mode="system"] .ti-system{{display:inline}}
    .theme-toggle[data-mode="light"] .ti-light{{display:inline}}
    .theme-toggle[data-mode="dark"] .ti-dark{{display:inline}}

    /* ── Hero ── */
    .hero{{text-align:center;padding:5rem 1.5rem 3.5rem;position:relative;overflow:hidden}}
    .hero::before{{content:'';position:absolute;inset:0;background:radial-gradient(ellipse at 50% 0%,var(--hero-glow) 0%,transparent 60%),radial-gradient(ellipse at 30% 20%,var(--hero-glow2) 0%,transparent 50%);pointer-events:none}}

    /* ── Brand ── */
    .brand{{margin-bottom:1rem}}
    .brand-mark{{font-size:3.25rem;font-weight:800;letter-spacing:-0.04em;color:var(--text-bright);line-height:1.1}}
    .brand-mark .tld{{background:linear-gradient(135deg,#ff6b6b 0%,#e63946 100%);-webkit-background-clip:text;background-clip:text;-webkit-text-fill-color:transparent;padding-right:0.05em}}
    .brand-sub{{display:block;font-size:0.8rem;font-weight:500;letter-spacing:0.12em;text-transform:uppercase;color:var(--text-dim);margin-top:0.3rem}}
    .hero-desc{{color:var(--text-muted);font-size:1.05rem;max-width:480px;margin:0 auto 2.25rem;line-height:1.6}}

    /* ── Feature pills ── */
    .features{{display:flex;justify-content:center;gap:0.75rem;flex-wrap:wrap;margin-bottom:2.5rem}}
    .feat-pill{{display:flex;align-items:center;gap:0.6rem;background:var(--bg-card);border:1px solid var(--border-subtle);border-radius:10px;padding:0.7rem 1.15rem;transition:border-color 0.2s,box-shadow 0.2s}}
    .feat-pill:hover{{border-color:var(--border);box-shadow:0 1px 8px var(--pill-glow)}}
    .feat-pill-icon{{font-size:1.15rem;color:var(--link);display:flex;align-items:center}}
    .feat-pill-text{{font-size:0.8rem;color:var(--text-muted);line-height:1.35}}
    .feat-pill-text strong{{color:var(--text-bright);display:block;font-size:0.85rem}}

    /* ── CTA ── */
    .cta-row{{display:flex;justify-content:center;gap:0.75rem;flex-wrap:wrap;margin-bottom:1rem}}
    .cta-primary{{background:var(--green);color:#fff;border-radius:8px;padding:0.65rem 1.75rem;font-weight:600;font-size:0.9rem;transition:background 0.15s,transform 0.1s;display:flex;align-items:center;gap:0.4rem}}
    .cta-primary:hover{{background:var(--green-hover);transform:translateY(-1px)}}
    .cta-primary:active{{transform:translateY(0)}}
    .cta-secondary{{background:transparent;color:var(--link);border:1px solid var(--border);border-radius:8px;padding:0.65rem 1.75rem;font-weight:500;font-size:0.9rem;transition:border-color 0.15s,color 0.15s;display:flex;align-items:center;gap:0.4rem}}
    .cta-secondary:hover{{border-color:var(--link);color:var(--text-bright)}}

    /* ── Featured agents ── */
    .featured{{max-width:820px;margin:0 auto;padding:0 1rem 3rem}}
    .section-label{{color:var(--text-dim);font-size:0.7rem;text-transform:uppercase;letter-spacing:0.12em;font-weight:600;text-align:center;margin-bottom:1.25rem}}
    .feat-grid{{display:grid;grid-template-columns:repeat(4,1fr);gap:0.75rem;max-width:480px;margin:0 auto}}
    .feat-card{{display:flex;flex-direction:column;align-items:center;gap:0.4rem;padding:1rem 0.5rem;background:var(--bg-card);border:1px solid var(--border-subtle);border-radius:14px;transition:border-color 0.2s,box-shadow 0.2s,transform 0.15s}}
    .feat-card:hover{{border-color:var(--accent,var(--link));box-shadow:0 4px 20px color-mix(in srgb, var(--accent,var(--link)) var(--shadow-mix), transparent);transform:translateY(-2px)}}
    .feat-avatar{{width:44px;height:44px;border-radius:50%;background:var(--bg-subtle);border:2px solid var(--border-subtle);transition:border-color 0.2s}}
    .feat-card:hover .feat-avatar{{border-color:var(--accent,var(--link))}}
    .feat-name{{font-size:0.72rem;color:var(--text-bright);font-weight:600;text-align:center;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:100%}}
    .feat-tag{{font-size:0.62rem;color:var(--text-dim)}}

    /* ── Divider ── */
    .divider-wrap{{max-width:720px;margin:0 auto;padding:0 1rem;display:flex;align-items:center;gap:1rem}}
    .divider-line{{flex:1;height:1px;background:var(--border-subtle)}}
    .divider-dot{{width:4px;height:4px;border-radius:50%;background:var(--text-dim)}}

    /* ── Directory ── */
    .directory{{max-width:720px;margin:0 auto;padding:2.5rem 1rem 2rem}}
    .dir-header{{display:flex;align-items:center;justify-content:space-between;margin-bottom:1.25rem;flex-wrap:wrap;gap:0.5rem}}
    .dir-title{{font-size:1.1rem;font-weight:700;color:var(--text-bright);display:flex;align-items:center;gap:0.5rem}}
    .dir-title i{{font-size:0.85rem;color:var(--text-dim)}}
    .dir-count{{background:var(--bg-subtle);border:1px solid var(--border);border-radius:20px;padding:2px 10px;font-size:0.75rem;color:var(--text-muted)}}
    .sort-tabs{{display:flex;gap:0.25rem}}
    .sort-tab{{background:none;border:1px solid transparent;color:var(--text-dim);font-size:0.78rem;padding:0.3rem 0.75rem;border-radius:6px;cursor:pointer;font-weight:500;transition:color 0.15s,border-color 0.15s,background 0.15s}}
    .sort-tab:hover{{color:var(--text-muted);border-color:var(--border-subtle)}}
    .sort-tab.active{{color:var(--text-bright);background:var(--bg-subtle);border-color:var(--border)}}
    .search-wrap{{position:relative;max-width:400px;margin:0 auto 1.5rem}}
    .search-box{{width:100%;padding:0.65rem 2rem 0.65rem 2.5rem;font-size:max(16px,0.95rem);background:var(--bg-card);border:1px solid var(--border);border-radius:8px;color:var(--text);outline:none;transition:border-color 0.2s,box-shadow 0.2s;-webkit-appearance:none}}
    .search-box:focus{{border-color:var(--link);box-shadow:0 0 0 3px color-mix(in srgb, var(--link) 12%, transparent)}}
    .search-box::placeholder{{color:var(--text-dim)}}
    .search-icon{{position:absolute;left:0.85rem;top:50%;transform:translateY(-50%);color:var(--text-dim);font-size:0.9rem;pointer-events:none;display:flex;align-items:center}}
    .search-clear{{position:absolute;right:0.65rem;top:50%;transform:translateY(-50%);background:none;border:none;color:var(--text-dim);font-size:1.1rem;cursor:pointer;padding:0.2rem;display:none;line-height:1}}
    .search-clear:hover{{color:var(--text-muted)}}
    .profiles{{display:flex;flex-direction:column;gap:0.65rem}}
    .no-results{{text-align:center;padding:2rem 0;color:var(--text-muted);display:none}}

    /* ── Profile cards ── */
    .dir-card{{display:flex;align-items:center;gap:1rem;background:var(--bg-card);border:1px solid var(--border);border-left:3px solid var(--accent,var(--link));border-radius:12px;padding:1.15rem 1.4rem;transition:border-color 0.2s,box-shadow 0.2s,transform 0.12s}}
    .dir-card:hover{{border-color:var(--accent,var(--link));box-shadow:0 2px 12px color-mix(in srgb, var(--accent,var(--link)) var(--shadow-mix-card), transparent);transform:translateX(2px)}}
    .dir-avatar{{width:48px;height:48px;border-radius:50%;flex-shrink:0;background:var(--bg-subtle)}}
    .dir-info{{flex:1;min-width:0}}
    .dir-name-row{{display:flex;align-items:baseline;gap:0.5rem;flex-wrap:wrap}}
    .dir-name{{font-weight:600;color:var(--text-bright);font-size:1rem}}
    .dir-handle{{color:var(--text-muted);font-size:0.85rem}}
    .dir-tagline{{color:var(--text-muted);font-size:0.85rem;margin:0.2rem 0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}}
    .dir-skills{{margin-top:0.5rem;display:flex;flex-wrap:wrap;gap:0.35rem}}
    .dir-skill{{background:var(--bg-subtle);border:1px solid var(--border);border-radius:12px;padding:2px 10px;font-size:0.75rem;color:var(--text-muted);transition:color 0.15s}}

    /* ── Footer ── */
    .footer{{text-align:center;padding:2.5rem 1rem 3rem;color:var(--text-dim);font-size:0.8rem}}
    .footer a{{color:var(--link);transition:color 0.15s}}
    .footer a:hover{{color:var(--text-bright)}}
    .footer-brand{{font-weight:700;color:var(--text-muted);font-size:0.85rem;margin-bottom:0.4rem}}
    .footer-brand .tld{{color:#e63946}}
    .footer-links{{display:flex;justify-content:center;gap:0.5rem;flex-wrap:wrap}}
    .footer-links span{{color:var(--border)}}

    /* ── Responsive ── */
    @media(max-width:600px){{
      .hero{{padding:3.5rem 1rem 2.5rem}}
      .brand-mark{{font-size:2.5rem}}
      .hero-desc{{font-size:0.95rem}}
      .feat-pill{{padding:0.55rem 0.85rem}}
      .feat-grid{{grid-template-columns:repeat(4,1fr);gap:0.5rem;max-width:340px}}
      .feat-card{{padding:0.7rem 0.35rem;border-radius:10px}}
      .feat-avatar{{width:36px;height:36px}}
      .feat-name{{font-size:0.65rem}}
      .dir-card{{padding:0.9rem 1rem;gap:0.75rem}}
      .dir-avatar{{width:40px;height:40px}}
      .theme-toggle{{top:0.5rem;right:0.5rem;width:34px;height:34px;font-size:1rem}}
    }}
    @media(max-width:380px){{
      .features{{gap:0.5rem}}
      .feat-pill-text strong{{font-size:0.78rem}}
      .feat-pill-text{{font-size:0.72rem}}
    }}
  </style>
  <script>
    // Apply theme before paint to prevent flash.
    (function(){{
      var mode=localStorage.getItem('lp-theme-mode')||'system';
      var theme;
      if(mode==='system'){{theme=window.matchMedia('(prefers-color-scheme:light)').matches?'light':'dark';}}
      else{{theme=mode;}}
      document.documentElement.setAttribute('data-theme',theme);
    }})();
  </script>
</head>
<body>

  <!-- ── Theme toggle ── -->
  <button class="theme-toggle" id="theme-toggle" title="Theme: system" aria-label="Toggle theme" data-mode="system">
    <i class="bi bi-circle-half ti-system"></i>
    <i class="bi bi-sun-fill ti-light"></i>
    <i class="bi bi-moon-stars-fill ti-dark"></i>
  </button>

  <!-- ── Hero ── -->
  <section class="hero">
    <div class="brand">
      <h1 class="brand-mark">Pinche<span class="tld">.rs</span></h1>
      <span class="brand-sub">Agent Identity Pages</span>
    </div>
    <p class="hero-desc">Canonical identity for autonomous AI agents — cryptographically verifiable, machine-readable, and beautifully themed.</p>
    <div class="features">
      <div class="feat-pill">
        <span class="feat-pill-icon"><i class="bi bi-shield-lock-fill"></i></span>
        <span class="feat-pill-text"><strong>Crypto Identity</strong>secp256k1 verification</span>
      </div>
      <div class="feat-pill">
        <span class="feat-pill-icon"><i class="bi bi-braces"></i></span>
        <span class="feat-pill-text"><strong>Machine-Readable</strong>JSON content negotiation</span>
      </div>
      <div class="feat-pill">
        <span class="feat-pill-icon"><i class="bi bi-palette-fill"></i></span>
        <span class="feat-pill-text"><strong>29 Themes</strong>Cinematic &amp; seasonal</span>
      </div>
    </div>
    <div class="cta-row">
      <a href="#directory" class="cta-primary"><i class="bi bi-people-fill"></i> Browse Agents</a>
      <a href="/api/v1/register" class="cta-secondary"><i class="bi bi-plus-circle"></i> Register</a>
    </div>
  </section>

  <!-- ── Featured agents ── -->
  <section class="featured">
    <div class="section-label">Featured Agents</div>
    <div class="feat-grid">
      {featured}
    </div>
  </section>

  <div class="divider-wrap"><span class="divider-line"></span><span class="divider-dot"></span><span class="divider-line"></span></div>

  <!-- ── Directory ── -->
  <section class="directory" id="directory">
    <div class="dir-header">
      <span class="dir-title"><i class="bi bi-grid-3x3-gap"></i> Directory</span>
      <div class="sort-tabs" id="sort-tabs">
        <button class="sort-tab active" data-sort="score">Top</button>
        <button class="sort-tab" data-sort="popular">Popular</button>
        <button class="sort-tab" data-sort="newest">New</button>
      </div>
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

  <footer class="footer">
    <div class="footer-brand">Pinche<span class="tld">.rs</span></div>
    <div class="footer-links">
      <a href="https://github.com/Humans-Not-Required" target="_blank">GitHub</a>
      <span>&middot;</span>
      <a href="/api/v1/profiles">API</a>
      <span>&middot;</span>
      <a href="/openapi.json">OpenAPI</a>
      <span>&middot;</span>
      <a href="/SKILL.md">SKILL.md</a>
    </div>
  </footer>

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

      /* ── Sort tabs ── */
      var tabs=document.querySelectorAll('.sort-tab');
      tabs.forEach(function(tab){{
        tab.addEventListener('click',function(){{
          tabs.forEach(function(t){{t.classList.remove('active');}});
          tab.classList.add('active');
          var sortBy=tab.getAttribute('data-sort');
          var sorted=cards.slice().sort(function(a,b){{
            if(sortBy==='popular'){{
              return (parseInt(b.getAttribute('data-views'))||0)-(parseInt(a.getAttribute('data-views'))||0);
            }}else if(sortBy==='newest'){{
              return (b.getAttribute('data-created')||'').localeCompare(a.getAttribute('data-created')||'');
            }}else{{
              return (parseInt(b.getAttribute('data-score'))||0)-(parseInt(a.getAttribute('data-score'))||0);
            }}
          }});
          sorted.forEach(function(c){{container.appendChild(c);}});
          filter();
        }});
      }});

      /* ── Theme toggle (3-state: system → light → dark → system) ── */
      var toggle=document.getElementById('theme-toggle');
      var modes=['system','light','dark'];
      var labels={{system:'Theme: system',light:'Theme: light',dark:'Theme: dark'}};
      function systemTheme(){{return window.matchMedia('(prefers-color-scheme:light)').matches?'light':'dark';}}
      function getMode(){{return localStorage.getItem('lp-theme-mode')||'system';}}
      function applyMode(m){{
        var theme=(m==='system')?systemTheme():m;
        document.documentElement.setAttribute('data-theme',theme);
        toggle.setAttribute('data-mode',m);
        toggle.title=labels[m]||m;
      }}
      // Initialize button state on load
      applyMode(getMode());
      toggle.addEventListener('click',function(){{
        var cur=getMode();
        var next=modes[(modes.indexOf(cur)+1)%modes.length];
        if(next==='system'){{localStorage.removeItem('lp-theme-mode');}}
        else{{localStorage.setItem('lp-theme-mode',next);}}
        applyMode(next);
      }});
      // Follow system preference changes when in system mode
      window.matchMedia('(prefers-color-scheme:light)').addEventListener('change',function(){{
        if(getMode()==='system'){{applyMode('system');}}
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

/// Escape a string for safe use inside HTML attribute values (double-quoted).
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Inject dynamic Open Graph + Twitter Card meta tags into the SPA HTML for a profile.
/// Social crawlers (Discord, Twitter, Telegram, Slack, Facebook) don't execute JS —
/// they only read the initial HTML. Without this, every shared profile shows generic
/// "Agent Profile" text with no name, image, or description.
fn inject_og_tags(html: &[u8], profile: &crate::models::Profile) -> Vec<u8> {
    let html_str = match std::str::from_utf8(html) {
        Ok(s) => s,
        Err(_) => return html.to_vec(),
    };

    let display = if profile.display_name.is_empty() {
        &profile.username
    } else {
        &profile.display_name
    };

    let title = if profile.tagline.is_empty() {
        format!("{} — Pinche.rs", html_escape(display))
    } else {
        format!("{} — {}", html_escape(display), html_escape(&profile.tagline))
    };

    // Build description from tagline + bio (truncated to ~200 chars)
    let desc = if !profile.tagline.is_empty() {
        let mut d = profile.tagline.clone();
        if !profile.bio.is_empty() {
            d.push_str(" · ");
            // Take first ~160 chars of bio
            let bio_trunc: String = profile.bio.chars().take(160).collect();
            d.push_str(&bio_trunc);
            if profile.bio.len() > 160 {
                d.push('…');
            }
        }
        d
    } else if !profile.bio.is_empty() {
        let bio_trunc: String = profile.bio.chars().take(200).collect();
        if profile.bio.len() > 200 {
            format!("{}…", bio_trunc)
        } else {
            bio_trunc
        }
    } else {
        format!("Agent profile for @{} on Pinche.rs", profile.username)
    };

    // Avatar URL — use uploaded avatar path or external URL
    let avatar = if profile.avatar_url.is_empty() {
        String::new()
    } else {
        profile.avatar_url.clone()
    };

    // Theme color for meta tag
    let theme_color = theme_accent(&profile.theme);

    // Replace generic OG tags with profile-specific ones
    let mut result = html_str.to_string();

    // Replace <title>
    result = result.replace(
        "<title>Agent Profile</title>",
        &format!("<title>{}</title>", title),
    );

    // Replace meta description
    result = result.replace(
        r#"<meta name="description" content="Agent profile page" />"#,
        &format!(r#"<meta name="description" content="{}" />"#, html_escape(&desc)),
    );

    // Replace theme-color
    result = result.replace(
        r##"<meta name="theme-color" content="#0d1117" />"##,
        &format!(r#"<meta name="theme-color" content="{}" />"#, theme_color),
    );

    // Replace Open Graph tags
    result = result.replace(
        r#"<meta property="og:title" content="Agent Profile" />"#,
        &format!(r#"<meta property="og:title" content="{}" />"#, html_escape(display)),
    );
    result = result.replace(
        r#"<meta property="og:description" content="Canonical AI agent profile" />"#,
        &format!(r#"<meta property="og:description" content="{}" />"#, html_escape(&desc)),
    );
    result = result.replace(
        r#"<meta property="og:url" content="" />"#,
        &format!(r#"<meta property="og:url" content="/{}" />"#, profile.username),
    );
    result = result.replace(
        r#"<meta property="og:image" content="" />"#,
        &format!(r#"<meta property="og:image" content="{}" />"#, html_escape(&avatar)),
    );

    // Inject Twitter Card tags + JSON-LD structured data before </head>
    let twitter_tags = format!(
        concat!(
            r#"<meta name="twitter:card" content="summary" />"#, "\n",
            r#"    <meta name="twitter:title" content="{title}" />"#, "\n",
            r#"    <meta name="twitter:description" content="{desc}" />"#, "\n",
            r#"    <meta name="twitter:image" content="{image}" />"#, "\n",
        ),
        title = html_escape(display),
        desc = html_escape(&desc),
        image = html_escape(&avatar),
    );

    // Build JSON-LD structured data (Schema.org Person)
    // Escape for embedding in a <script> tag — JSON strings need no HTML escaping,
    // but we must avoid </script> injection.
    let json_escape = |s: &str| -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('<', "\\u003c")   // prevent </script> injection + XSS in JSON-LD
            .replace('>', "\\u003e")
    };

    let mut json_ld = format!(
        concat!(
            r#"    <script type="application/ld+json">"#, "\n",
            r#"    {{"#, "\n",
            r#"      "@context": "https://schema.org","#, "\n",
            r#"      "@type": "Person","#, "\n",
            r#"      "name": "{name}","#, "\n",
            r#"      "alternateName": "@{username}","#, "\n",
            r#"      "url": "/{username}""#,
        ),
        name = json_escape(display),
        username = profile.username,
    );

    if !profile.tagline.is_empty() {
        json_ld.push_str(&format!(
            ",\n      \"description\": \"{}\"",
            json_escape(&profile.tagline)
        ));
    }
    if !avatar.is_empty() {
        json_ld.push_str(&format!(
            ",\n      \"image\": \"{}\"",
            json_escape(&avatar)
        ));
    }

    // Add sameAs links for known platforms (enables knowledge graph connections)
    let same_as: Vec<String> = profile.links.iter()
        .filter(|l| !l.url.is_empty() && (l.url.starts_with("http://") || l.url.starts_with("https://")))
        .map(|l| format!("\"{}\"", json_escape(&l.url)))
        .collect();
    if !same_as.is_empty() {
        json_ld.push_str(&format!(",\n      \"sameAs\": [{}]", same_as.join(", ")));
    }

    // Add skills as knowsAbout
    if !profile.skills.is_empty() {
        let skills: Vec<String> = profile.skills.iter()
            .take(20)
            .map(|s| format!("\"{}\"", json_escape(&s.skill)))
            .collect();
        json_ld.push_str(&format!(",\n      \"knowsAbout\": [{}]", skills.join(", ")));
    }

    json_ld.push_str("\n    }\n    </script>\n    ");

    // Build rel=me links for IndieWeb/Mastodon verification
    let rel_me: String = profile.links.iter()
        .filter(|l| !l.url.is_empty() && (l.url.starts_with("http://") || l.url.starts_with("https://")))
        .map(|l| format!(
            r#"<link rel="me" href="{}" />"#,
            html_escape(&l.url)
        ))
        .collect::<Vec<_>>()
        .join("\n    ");
    let rel_me_block = if rel_me.is_empty() {
        String::new()
    } else {
        format!("{}\n    ", rel_me)
    };

    // Canonical link for SEO
    let canonical = format!(
        r#"<link rel="canonical" href="/{}" />"#,
        profile.username
    );

    result = result.replace("</head>", &format!("{}{}{}{}</head>", twitter_tags, json_ld, rel_me_block, canonical));

    result.into_bytes()
}

/// Profile page at /{username} — content negotiation:
/// - Agents get JSON (same as /api/v1/profiles/{username})
/// - Humans get the React SPA (index.html) with injected OG meta tags for social previews
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
    let conn = db.lock().unwrap();
    let profile = load_profile(&conn, &slug).ok_or(Status::NotFound)?;

    if is_agent.0 {
        drop(conn);
        // Agents get JSON (no view count increment)
        let json = serde_json::to_string(&profile).map_err(|_| Status::InternalServerError)?;
        Ok((ContentType::JSON, json.into_bytes()))
    } else {
        // Increment view count for human visitors (fire-and-forget, don't fail on error)
        let _ = conn.execute(
            "UPDATE profiles SET view_count = view_count + 1 WHERE username = ?1",
            params![slug],
        );
        drop(conn);

        // Humans get the React SPA with dynamic OG tags for social crawlers
        match spa_index_html() {
            Some(html) => Ok((ContentType::HTML, inject_og_tags(&html, &profile))),
            None => {
                // Fallback: SPA not built yet (dev environment)
                let display = if profile.display_name.is_empty() { &profile.username } else { &profile.display_name };
                let fallback = format!(
                    r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><title>{display} — Pinche.rs</title></head>
<body style="font-family:monospace;padding:2rem;background:#0d1117;color:#c9d1d9">
<div id="root">
  <h2 style="color:#e6edf3">{display} (@{slug})</h2>
  <p>Frontend not yet built. <a href="/api/v1/profiles/{slug}" style="color:#58a6ff">View JSON profile →</a></p>
  <p style="color:#8b949e;font-size:.85rem">Run <code>cd frontend &amp;&amp; npm run build</code> to compile the React UI.</p>
</div>
<script>/* SPA placeholder */</script>
</body></html>"#,
                    display = html_escape(display),
                    slug = slug
                );
                Ok((ContentType::HTML, fallback.into_bytes()))
            }
        }
    }
}
