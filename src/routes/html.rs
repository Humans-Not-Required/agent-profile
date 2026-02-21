use rocket::{get, State, http::{ContentType, Status}};
use rusqlite::params;
use std::sync::Mutex;

use crate::routes::profiles::{DbConn, load_profile};

fn network_icon(network: &str) -> &str {
    match network {
        "bitcoin" => "₿",
        "lightning" => "⚡",
        "ethereum" => "Ξ",
        "solana" => "◎",
        "nostr" => "🔑",
        _ => "🔗",
    }
}

fn link_icon(link_type: &str) -> &str {
    match link_type {
        "github" => "🐙",
        "nostr" => "🔑",
        "moltbook" => "📚",
        "telegram" => "✈️",
        "email" => "📧",
        "twitter" => "🐦",
        "website" => "🌐",
        _ => "🔗",
    }
}

fn render_profile_html(slug: &str, db: &State<DbConn>) -> Option<String> {
    let conn = db.lock().unwrap();
    let profile = load_profile(&conn, slug)?;

    let avatar_section = if !profile.avatar_url.is_empty() {
        format!(
            r#"<img src="{}" alt="{}'s avatar" class="avatar" onerror="this.style.display='none'">"#,
            html_escape(&profile.avatar_url),
            html_escape(&profile.display_name)
        )
    } else {
        format!(
            r#"<div class="avatar-placeholder">{}</div>"#,
            profile.display_name.chars().next().unwrap_or('?').to_uppercase().next().unwrap_or('?')
        )
    };

    let bio_section = if !profile.bio.is_empty() {
        format!(r#"<p class="bio">{}</p>"#, html_escape(&profile.bio))
    } else {
        String::new()
    };

    let skills_section = if !profile.skills.is_empty() {
        let tags: String = profile.skills.iter()
            .map(|s| format!(r#"<span class="skill-tag">{}</span>"#, html_escape(&s.skill)))
            .collect::<Vec<_>>()
            .join(" ");
        format!(r#"<div class="section"><h3>Skills</h3><div class="skill-tags">{}</div></div>"#, tags)
    } else {
        String::new()
    };

    let addresses_section = if !profile.crypto_addresses.is_empty() {
        let items: String = profile.crypto_addresses.iter()
            .map(|a| {
                let label = if !a.label.is_empty() {
                    format!(" <span class=\"addr-label\">({})</span>", html_escape(&a.label))
                } else {
                    String::new()
                };
                let verified = if a.verified { " ✅" } else { "" };
                format!(
                    r#"<div class="addr-row">
                        <span class="network-icon">{icon}</span>
                        <span class="network-name">{network}{verified}</span>{label}
                        <code class="addr-value">{address}</code>
                    </div>"#,
                    icon = network_icon(&a.network),
                    network = html_escape(&a.network),
                    verified = verified,
                    label = label,
                    address = html_escape(&a.address),
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!(r#"<div class="section"><h3>Crypto Addresses</h3>{}</div>"#, items)
    } else {
        String::new()
    };

    let links_section = if !profile.links.is_empty() {
        let items: String = profile.links.iter()
            .map(|l| format!(
                r#"<a href="{}" class="profile-link" target="_blank" rel="noopener">{} {}</a>"#,
                html_escape(&l.value),
                link_icon(&l.link_type),
                html_escape(&l.label),
            ))
            .collect::<Vec<_>>()
            .join("\n");
        format!(r#"<div class="section links-section">{}</div>"#, items)
    } else {
        String::new()
    };

    let json_url = format!("/api/v1/profiles/{}", slug);
    let created = profile.created_at.get(..10).unwrap_or(&profile.created_at);

    Some(format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{display_name} — Agent Profile</title>
  <meta property="og:title" content="{display_name}">
  <meta property="og:description" content="{bio_meta}">
  <meta name="description" content="{bio_meta}">
  <link rel="alternate" type="application/json" href="{json_url}" title="Machine-readable profile">
  <style>
    * {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      background: #0d1117;
      color: #c9d1d9;
      min-height: 100vh;
      padding: 2rem 1rem;
    }}
    .card {{
      max-width: 680px;
      margin: 0 auto;
      background: #161b22;
      border: 1px solid #30363d;
      border-radius: 12px;
      padding: 2.5rem;
    }}
    .profile-header {{
      display: flex;
      align-items: center;
      gap: 1.5rem;
      margin-bottom: 1.5rem;
    }}
    .avatar {{
      width: 80px;
      height: 80px;
      border-radius: 50%;
      border: 2px solid #30363d;
      object-fit: cover;
      flex-shrink: 0;
    }}
    .avatar-placeholder {{
      width: 80px;
      height: 80px;
      border-radius: 50%;
      background: linear-gradient(135deg, #1f6feb, #388bfd);
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 2rem;
      font-weight: 700;
      color: white;
      flex-shrink: 0;
    }}
    .profile-info h1 {{ font-size: 1.5rem; color: #e6edf3; margin-bottom: 0.25rem; }}
    .profile-info .slug {{ color: #8b949e; font-size: 0.9rem; font-family: monospace; }}
    .bio {{
      color: #c9d1d9;
      line-height: 1.6;
      margin-bottom: 1.5rem;
      white-space: pre-wrap;
    }}
    .section {{ margin-bottom: 1.5rem; }}
    .section h3 {{ color: #8b949e; font-size: 0.8rem; text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 0.75rem; }}
    .skill-tags {{ display: flex; flex-wrap: wrap; gap: 0.5rem; }}
    .skill-tag {{
      background: #21262d;
      border: 1px solid #30363d;
      color: #79c0ff;
      padding: 0.2rem 0.6rem;
      border-radius: 20px;
      font-size: 0.85rem;
      font-family: monospace;
    }}
    .addr-row {{
      display: flex;
      align-items: baseline;
      gap: 0.5rem;
      padding: 0.5rem 0;
      border-bottom: 1px solid #21262d;
      flex-wrap: wrap;
    }}
    .addr-row:last-child {{ border-bottom: none; }}
    .network-icon {{ font-size: 1rem; flex-shrink: 0; }}
    .network-name {{ color: #e6edf3; font-size: 0.9rem; font-weight: 500; flex-shrink: 0; }}
    .addr-label {{ color: #8b949e; font-size: 0.8rem; }}
    .addr-value {{
      color: #79c0ff;
      font-size: 0.75rem;
      word-break: break-all;
      background: #21262d;
      padding: 0.15rem 0.4rem;
      border-radius: 4px;
    }}
    .links-section {{ display: flex; flex-wrap: wrap; gap: 0.75rem; }}
    .profile-link {{
      display: inline-flex;
      align-items: center;
      gap: 0.4rem;
      background: #21262d;
      border: 1px solid #30363d;
      color: #c9d1d9;
      padding: 0.4rem 0.9rem;
      border-radius: 8px;
      text-decoration: none;
      font-size: 0.9rem;
      transition: border-color 0.2s;
    }}
    .profile-link:hover {{ border-color: #58a6ff; color: #58a6ff; }}
    .meta {{
      margin-top: 2rem;
      padding-top: 1rem;
      border-top: 1px solid #21262d;
      display: flex;
      justify-content: space-between;
      align-items: center;
      flex-wrap: wrap;
      gap: 0.5rem;
    }}
    .meta-text {{ color: #484f58; font-size: 0.8rem; }}
    .json-link {{
      font-size: 0.8rem;
      color: #58a6ff;
      text-decoration: none;
      font-family: monospace;
    }}
    .json-link:hover {{ text-decoration: underline; }}
    .hnr-badge {{
      color: #484f58;
      font-size: 0.75rem;
      text-align: center;
      margin-top: 1.5rem;
    }}
    .hnr-badge a {{ color: #484f58; }}
  </style>
</head>
<body>
  <div class="card">
    <div class="profile-header">
      {avatar_section}
      <div class="profile-info">
        <h1>{display_name}</h1>
        <div class="slug">@{slug}</div>
      </div>
    </div>

    {bio_section}
    {links_section}
    {skills_section}
    {addresses_section}

    <div class="meta">
      <span class="meta-text">Member since {created}</span>
      <a href="{json_url}" class="json-link">{{}} JSON</a>
    </div>
  </div>
  <div class="hnr-badge">
    <a href="https://github.com/Humans-Not-Required" target="_blank" rel="noopener">Humans Not Required</a>
  </div>
</body>
</html>"#,
        display_name = html_escape(&profile.display_name),
        slug = slug,
        bio_meta = html_escape(&profile.bio.get(..160).unwrap_or(&profile.bio)),
        avatar_section = avatar_section,
        bio_section = bio_section,
        links_section = links_section,
        skills_section = skills_section,
        addresses_section = addresses_section,
        json_url = json_url,
        created = created,
    ))
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&#x27;")
}

#[get("/agents/<slug>")]
pub fn profile_page(
    db: &State<DbConn>,
    slug: &str,
) -> Result<(ContentType, String), Status> {
    match render_profile_html(&slug.to_lowercase(), db) {
        Some(html) => Ok((ContentType::HTML, html)),
        None => Err(Status::NotFound),
    }
}
