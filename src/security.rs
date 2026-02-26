use rocket::{
    fairing::{Fairing, Info, Kind},
    Request, Response,
    http::Header,
};

/// Adds security headers to all responses.
///
/// - Content-Security-Policy: restricts resource loading
/// - X-Content-Type-Options: prevents MIME sniffing
/// - X-Frame-Options: prevents clickjacking
/// - Referrer-Policy: limits referrer leakage
/// - Permissions-Policy: disables unnecessary browser APIs
/// - X-XSS-Protection: legacy XSS filter (for older browsers)
pub struct SecurityHeaders;

#[rocket::async_trait]
impl Fairing for SecurityHeaders {
    fn info(&self) -> Info {
        Info {
            name: "Security Headers",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        let path = request.uri().path().as_str();

        // Content-Security-Policy — tailored for React SPA
        // Allow inline styles (React CSS-in-JS + our inline style attributes)
        // Allow blob: for Web Share API
        // Allow data: for SVG avatar fallbacks
        // Allow specific CDN for Bootstrap Icons font
        let csp = "default-src 'self'; \
                    script-src 'self'; \
                    style-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net; \
                    img-src 'self' data: https:; \
                    font-src 'self' https://cdn.jsdelivr.net; \
                    connect-src 'self'; \
                    frame-src 'none'; \
                    object-src 'none'; \
                    base-uri 'self'; \
                    form-action 'self'";
        response.set_header(Header::new("Content-Security-Policy", csp));

        // Prevent MIME type sniffing
        response.set_header(Header::new("X-Content-Type-Options", "nosniff"));

        // Prevent embedding in iframes (clickjacking protection)
        response.set_header(Header::new("X-Frame-Options", "DENY"));

        // Limit referrer information leakage
        response.set_header(Header::new("Referrer-Policy", "strict-origin-when-cross-origin"));

        // Disable unnecessary browser features
        response.set_header(Header::new(
            "Permissions-Policy",
            "camera=(), microphone=(), geolocation=(), payment=()",
        ));

        // Legacy XSS filter for older browsers
        response.set_header(Header::new("X-XSS-Protection", "1; mode=block"));

        // Cache-Control based on response type
        add_cache_headers(path, response);
    }
}

/// Set Cache-Control headers based on path.
///
/// Strategy:
/// - /assets/* (hashed filenames from Vite) → immutable, 1 year
/// - /avatars/* → 1 hour (can change when user uploads new)
/// - /SKILL.md, /llms.txt, /openapi.json → 1 hour
/// - /feed.xml → 15 minutes
/// - /robots.txt, /sitemap.xml → 1 day
/// - /api/v1/health → no-cache
/// - /api/v1/* → 60 seconds (short cache for API)
/// - / and profile pages → 5 minutes
fn add_cache_headers(path: &str, response: &mut Response<'_>) {
    let cache_value = if path.starts_with("/assets/") {
        // Vite uses content hashes in filenames — safe to cache forever
        "public, max-age=31536000, immutable"
    } else if path.starts_with("/avatars/")
           || path == "/SKILL.md" || path == "/llms.txt" || path == "/openapi.json"
           || path == "/.well-known/skills/index.json" {
        "public, max-age=3600"
    } else if path == "/feed.xml" {
        "public, max-age=900"
    } else if path == "/robots.txt" || path == "/sitemap.xml" {
        "public, max-age=86400"
    } else if path == "/api/v1/health" {
        "no-cache"
    } else if path.starts_with("/api/v1/") {
        "public, max-age=60"
    } else {
        // Landing page, profile pages
        "public, max-age=300"
    };

    response.set_header(Header::new("Cache-Control", cache_value));
}
