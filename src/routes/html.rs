use rocket::{get, State, http::{ContentType, Status}};

use crate::routes::profiles::{DbConn, load_profile};
use crate::models::Profile;

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

fn network_icon(network: &str) -> &'static str {
    match network {
        "bitcoin" => "₿",
        "lightning" => "⚡",
        "ethereum" => "Ξ",
        "cardano" => "₳",
        "ergo" => "ERG",
        "nervos" => "CKB",
        "solana" => "◎",
        "monero" => "ɱ",
        "dogecoin" => "Ð",
        "nostr" => "🔑",
        _ => "🔗",
    }
}

fn link_icon(platform: &str) -> &'static str {
    match platform {
        "github" => "🐙",
        "nostr" => "🔑",
        "moltbook" => "📚",
        "telegram" => "✈️",
        "email" => "📧",
        "twitter" => "🐦",
        "discord" => "💬",
        "youtube" => "▶️",
        "linkedin" => "💼",
        "website" => "🌐",
        _ => "🔗",
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&#x27;")
}

/// Generate a deterministic hue (0-360) from a username string.
fn username_hue(username: &str) -> u32 {
    let hash: u32 = username.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    hash % 360
}

fn render_profile_html(slug: &str, profile: &Profile) -> String {
    let hue = username_hue(slug);

    let avatar_section = if !profile.avatar_url.is_empty() {
        format!(
            r#"<img src="{}" alt="{}'s avatar" class="avatar" onerror="this.style.display='none'; document.querySelector('.avatar-placeholder').style.display='flex'"><div class="avatar-placeholder" style="display:none; --hue:{hue}">{initials}</div>"#,
            html_escape(&profile.avatar_url),
            html_escape(&profile.display_name),
            hue = hue,
            initials = profile.display_name.chars().take(2).collect::<String>().to_uppercase(),
        )
    } else {
        format!(
            r#"<div class="avatar-placeholder" style="--hue:{hue}">{initials}</div>"#,
            hue = hue,
            initials = profile.display_name.chars().take(2).collect::<String>().to_uppercase(),
        )
    };

    let tagline_html = if !profile.tagline.is_empty() {
        format!(r#"<div class="tagline">{}</div>"#, html_escape(&profile.tagline))
    } else {
        String::new()
    };

    let third_line_html = if !profile.third_line.is_empty() {
        format!(r#"<div class="third-line">{}</div>"#, html_escape(&profile.third_line))
    } else {
        String::new()
    };

    // Quick links row (icons only, in hero)
    let quick_links = if !profile.links.is_empty() {
        let icons: String = profile.links.iter().take(6).map(|l| {
            format!(
                r#"<a href="{}" class="quick-link" title="{}" target="_blank" rel="noopener">{}</a>"#,
                html_escape(&l.url),
                html_escape(&l.label),
                link_icon(&l.platform),
            )
        }).collect::<Vec<_>>().join("");
        format!(r#"<div class="quick-links">{}</div>"#, icons)
    } else {
        String::new()
    };

    // Sections
    let sections_html: String = profile.sections.iter().map(|s| {
        format!(
            r#"<div class="section"><h3>{}</h3><div class="section-content">{}</div></div>"#,
            html_escape(&s.title),
            html_escape(&s.content).replace('\n', "<br>"),
        )
    }).collect::<Vec<_>>().join("\n");

    // Links section
    let links_html = if !profile.links.is_empty() {
        let items: String = profile.links.iter().map(|l| {
            format!(
                r#"<a href="{}" class="profile-link" target="_blank" rel="noopener">{} {}</a>"#,
                html_escape(&l.url),
                link_icon(&l.platform),
                html_escape(&l.label),
            )
        }).collect::<Vec<_>>().join("\n");
        format!(r#"<div class="section"><h3>Links</h3><div class="links-row">{}</div></div>"#, items)
    } else {
        String::new()
    };

    // Skills
    let skills_html = if !profile.skills.is_empty() {
        let tags: String = profile.skills.iter()
            .map(|s| format!(r#"<span class="skill-tag">{}</span>"#, html_escape(&s.skill)))
            .collect::<Vec<_>>().join(" ");
        format!(r#"<div class="section"><h3>Skills</h3><div class="skill-tags">{}</div></div>"#, tags)
    } else {
        String::new()
    };

    // Crypto addresses
    let addresses_html = if !profile.crypto_addresses.is_empty() {
        let items: String = profile.crypto_addresses.iter().map(|a| {
            let label = if !a.label.is_empty() {
                format!(" <span class=\"addr-label\">({})</span>", html_escape(&a.label))
            } else { String::new() };
            format!(
                r#"<div class="addr-row" onclick="copyAddr(this, '{addr}')">
                    <span class="network-icon">{icon}</span>
                    <span class="network-name">{network}</span>{label}
                    <code class="addr-value">{addr_short}…</code>
                    <span class="copy-hint">click to copy</span>
                </div>"#,
                icon = network_icon(&a.network),
                network = html_escape(&a.network),
                label = label,
                addr = html_escape(&a.address),
                addr_short = html_escape(a.address.get(..16).unwrap_or(&a.address)),
            )
        }).collect::<Vec<_>>().join("\n");
        format!(r#"<div class="section"><h3>Crypto Addresses</h3><div class="addr-list">{}</div></div>"#, items)
    } else {
        String::new()
    };

    // Profile score badge
    let score_badge = if profile.profile_score > 0 {
        let color = if profile.profile_score >= 80 { "#3fb950" }
                    else if profile.profile_score >= 50 { "#d29922" }
                    else { "#f85149" };
        format!(
            r#"<span class="score-badge" style="--score-color:{color}" title="Profile completeness">{score}% complete</span>"#,
            color = color,
            score = profile.profile_score,
        )
    } else {
        String::new()
    };

    let pubkey_badge = if !profile.pubkey.is_empty() {
        r#"<span class="verified-badge" title="secp256k1 identity verified">🔐 Cryptographic ID</span>"#.to_string()
    } else {
        String::new()
    };

    let json_url = format!("/api/v1/profiles/{}", slug);
    let created = profile.created_at.get(..10).unwrap_or(&profile.created_at);
    let theme_class = &profile.theme;

    format!(r#"<!DOCTYPE html>
<html lang="en" data-theme="{theme_class}">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{display_name} — Agent Profile</title>
  <meta property="og:title" content="{display_name}">
  <meta property="og:description" content="{bio_meta}">
  <meta name="description" content="{bio_meta}">
  <link rel="alternate" type="application/json" href="{json_url}" title="Machine-readable profile">
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.3/font/bootstrap-icons.min.css">
  <style>
    :root {{
      --bg: #0d1117;
      --card: #161b22;
      --border: #30363d;
      --text: #c9d1d9;
      --text-muted: #8b949e;
      --text-bright: #e6edf3;
      --accent: #58a6ff;
      --accent2: #79c0ff;
      --tag-bg: #21262d;
      --tag-border: #30363d;
      --section-bg: #0d1117;
    }}
    [data-theme="light"] {{
      --bg: #f6f8fa; --card: #ffffff; --border: #d0d7de;
      --text: #24292f; --text-muted: #57606a; --text-bright: #1f2328;
      --accent: #0969da; --accent2: #0a69da; --tag-bg: #eaeef2;
      --tag-border: #d0d7de; --section-bg: #f6f8fa;
    }}
    [data-theme="midnight"] {{
      --bg: #020409; --card: #0d1117; --border: #1f2d40;
      --text: #8b949e; --text-muted: #484f58; --text-bright: #c9d1d9;
      --accent: #00b4d8; --accent2: #48cae4; --tag-bg: #0d1520; --tag-border: #1f2d40;
    }}
    [data-theme="forest"] {{
      --bg: #0a1a0a; --card: #0f2410; --border: #1e3a1e;
      --text: #8db08d; --text-muted: #4a6b4a; --text-bright: #c4ddc4;
      --accent: #4caf50; --accent2: #81c784; --tag-bg: #0a1a0a; --tag-border: #1e3a1e;
    }}
    [data-theme="ocean"] {{
      --bg: #040d18; --card: #061525; --border: #0e2a47;
      --text: #7aadcc; --text-muted: #3d6d8a; --text-bright: #b3d4e8;
      --accent: #0077b6; --accent2: #00b4d8; --tag-bg: #040d18; --tag-border: #0e2a47;
    }}
    [data-theme="desert"] {{
      --bg: #1a0e00; --card: #271600; --border: #4a2d00;
      --text: #c4935a; --text-muted: #7a5520; --text-bright: #e8c49a;
      --accent: #f5a623; --accent2: #ffcc5c; --tag-bg: #1a0e00; --tag-border: #4a2d00;
    }}
    [data-theme="aurora"] {{
      --bg: #060a14; --card: #0a1220; --border: #1a2840;
      --text: #8babc8; --text-muted: #445566; --text-bright: #c4d8e8;
      --accent: #7b61ff; --accent2: #a78bfa; --tag-bg: #060a14; --tag-border: #1a2840;
    }}

    * {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      background: var(--bg);
      color: var(--text);
      min-height: 100vh;
      padding: 2rem 1rem;
    }}
    .card {{
      max-width: 720px;
      margin: 0 auto;
      background: var(--card);
      border: 1px solid var(--border);
      border-radius: 12px;
      padding: 2.5rem;
    }}
    .profile-header {{
      display: flex;
      align-items: flex-start;
      gap: 1.5rem;
      margin-bottom: 1.5rem;
    }}
    .avatar {{
      width: 88px; height: 88px;
      border-radius: 50%;
      border: 2px solid var(--border);
      object-fit: cover;
      flex-shrink: 0;
    }}
    .avatar-placeholder {{
      width: 88px; height: 88px;
      border-radius: 50%;
      background: hsl(var(--hue, 210), 60%, 35%);
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 1.8rem;
      font-weight: 700;
      color: white;
      flex-shrink: 0;
      letter-spacing: -0.05em;
    }}
    .profile-info {{ flex: 1; min-width: 0; }}
    .profile-info h1 {{ font-size: 1.6rem; color: var(--text-bright); margin-bottom: 0.2rem; line-height: 1.2; }}
    .tagline {{ color: var(--text-muted); font-size: 1rem; margin-bottom: 0.2rem; }}
    .third-line {{ color: var(--text-muted); font-size: 0.88rem; margin-bottom: 0.5rem; }}
    .quick-links {{ display: flex; gap: 0.6rem; flex-wrap: wrap; margin-top: 0.5rem; }}
    .quick-link {{
      color: var(--text-muted); font-size: 1.2rem;
      text-decoration: none; transition: color 0.15s;
    }}
    .quick-link:hover {{ color: var(--accent); }}
    .badges {{ display: flex; gap: 0.5rem; flex-wrap: wrap; margin-bottom: 1.5rem; }}
    .score-badge, .verified-badge {{
      display: inline-block;
      font-size: 0.78rem;
      padding: 0.2rem 0.55rem;
      border-radius: 20px;
      border: 1px solid var(--border);
    }}
    .score-badge {{ color: var(--score-color, var(--text-muted)); border-color: var(--score-color, var(--border)); }}
    .verified-badge {{ color: #3fb950; border-color: #3fb950; }}
    .section {{ margin-bottom: 1.5rem; }}
    .section h3 {{
      color: var(--text-muted);
      font-size: 0.78rem;
      text-transform: uppercase;
      letter-spacing: 0.06em;
      margin-bottom: 0.75rem;
      padding-bottom: 0.4rem;
      border-bottom: 1px solid var(--border);
    }}
    .section-content {{
      color: var(--text);
      line-height: 1.65;
      font-size: 0.95rem;
    }}
    .links-row {{ display: flex; flex-wrap: wrap; gap: 0.6rem; }}
    .profile-link {{
      display: inline-flex;
      align-items: center;
      gap: 0.4rem;
      background: var(--tag-bg);
      border: 1px solid var(--tag-border);
      color: var(--text);
      padding: 0.35rem 0.85rem;
      border-radius: 8px;
      text-decoration: none;
      font-size: 0.88rem;
      transition: border-color 0.15s, color 0.15s;
    }}
    .profile-link:hover {{ border-color: var(--accent); color: var(--accent); }}
    .skill-tags {{ display: flex; flex-wrap: wrap; gap: 0.4rem; }}
    .skill-tag {{
      background: var(--tag-bg);
      border: 1px solid var(--tag-border);
      color: var(--accent2);
      padding: 0.18rem 0.55rem;
      border-radius: 20px;
      font-size: 0.82rem;
      font-family: monospace;
    }}
    .addr-list {{ display: flex; flex-direction: column; gap: 0.4rem; }}
    .addr-row {{
      display: flex;
      align-items: center;
      gap: 0.6rem;
      padding: 0.5rem 0.6rem;
      background: var(--tag-bg);
      border: 1px solid var(--tag-border);
      border-radius: 6px;
      cursor: pointer;
      transition: border-color 0.15s;
      flex-wrap: wrap;
    }}
    .addr-row:hover {{ border-color: var(--accent); }}
    .network-icon {{ font-size: 0.9rem; flex-shrink: 0; }}
    .network-name {{ color: var(--text-bright); font-size: 0.88rem; font-weight: 500; flex-shrink: 0; min-width: 60px; }}
    .addr-label {{ color: var(--text-muted); font-size: 0.8rem; }}
    .addr-value {{
      color: var(--accent2);
      font-size: 0.78rem;
      font-family: monospace;
      background: transparent;
    }}
    .copy-hint {{ color: var(--text-muted); font-size: 0.72rem; margin-left: auto; }}
    .meta {{
      margin-top: 2rem;
      padding-top: 1rem;
      border-top: 1px solid var(--border);
      display: flex;
      justify-content: space-between;
      align-items: center;
      flex-wrap: wrap;
      gap: 0.5rem;
    }}
    .meta-text {{ color: var(--text-muted); font-size: 0.78rem; }}
    .json-link {{ font-size: 0.78rem; color: var(--accent); text-decoration: none; font-family: monospace; }}
    .json-link:hover {{ text-decoration: underline; }}
    .hnr-badge {{ color: var(--text-muted); font-size: 0.72rem; text-align: center; margin-top: 1.5rem; }}
    .hnr-badge a {{ color: var(--text-muted); }}
    .toast {{
      position: fixed; bottom: 1.5rem; right: 1.5rem;
      background: #3fb950; color: white;
      padding: 0.5rem 1rem; border-radius: 8px;
      font-size: 0.85rem; opacity: 0;
      transition: opacity 0.2s;
      pointer-events: none;
    }}
    .toast.show {{ opacity: 1; }}
  </style>
</head>
<body>
  <div class="card">
    <div class="profile-header">
      {avatar_section}
      <div class="profile-info">
        <h1>{display_name}</h1>
        {tagline_html}
        {third_line_html}
        {quick_links}
      </div>
    </div>

    <div class="badges">
      {score_badge}
      {pubkey_badge}
    </div>

    {sections_html}
    {links_html}
    {skills_html}
    {addresses_html}

    <div class="meta">
      <span class="meta-text">@{slug} · Member since {created}</span>
      <a href="{json_url}" class="json-link">{{}} JSON</a>
    </div>
  </div>
  <div class="hnr-badge">
    Powered by <a href="https://github.com/Humans-Not-Required" target="_blank" rel="noopener">Humans Not Required</a>
  </div>
  <div class="toast" id="toast">Copied!</div>

  <script>
    function copyAddr(el, addr) {{
      navigator.clipboard.writeText(addr).then(() => {{
        const t = document.getElementById('toast');
        t.classList.add('show');
        setTimeout(() => t.classList.remove('show'), 1800);
      }});
    }}
    // Apply avatar placeholder hue from CSS var
    document.querySelectorAll('.avatar-placeholder').forEach(el => {{
      const hue = el.style.getPropertyValue('--hue');
      if (hue) el.style.background = `hsl(${{hue}}, 60%, 35%)`;
    }});
  </script>
</body>
</html>"#,
        theme_class = theme_class,
        display_name = html_escape(&profile.display_name),
        slug = slug,
        bio_meta = html_escape(&profile.bio.get(..160).unwrap_or(&profile.bio)),
        avatar_section = avatar_section,
        tagline_html = tagline_html,
        third_line_html = third_line_html,
        quick_links = quick_links,
        score_badge = score_badge,
        pubkey_badge = pubkey_badge,
        sections_html = sections_html,
        links_html = links_html,
        skills_html = skills_html,
        addresses_html = addresses_html,
        json_url = json_url,
        created = created,
    )
}

/// Profile page at /{username} — content negotiation:
/// - Agents get JSON (same as /api/v1/profiles/{username})
/// - Humans get rendered HTML
#[get("/<username>", rank = 10)]
pub fn profile_page(
    db: &State<DbConn>,
    username: &str,
    is_agent: IsAgent,
) -> Result<(ContentType, String), Status> {
    // Skip reserved paths (handled by other routes)
    let reserved = ["api", "avatars", "openapi.json", "llms.txt", "favicon.ico"];
    if reserved.contains(&username) || username.starts_with('.') {
        return Err(Status::NotFound);
    }

    let conn = db.lock().unwrap();
    let slug = username.to_lowercase();
    let profile = load_profile(&conn, &slug).ok_or(Status::NotFound)?;

    if is_agent.0 {
        // Return JSON for agents
        let json = serde_json::to_string(&profile).map_err(|_| Status::InternalServerError)?;
        Ok((ContentType::JSON, json))
    } else {
        // Return HTML for humans
        Ok((ContentType::HTML, render_profile_html(&slug, &profile)))
    }
}
