use rocket::{get, http::{ContentType, Status}};
use rust_embed::Embed;

/// Embeds the compiled React frontend from `frontend/dist/` at compile time.
/// Falls back gracefully if the directory doesn't exist (e.g., first build without frontend).
#[derive(Embed)]
#[folder = "frontend/dist/"]
#[prefix = ""]
pub struct FrontendAssets;

/// Guess content type from file extension.
fn content_type_for(path: &str) -> ContentType {
    if path.ends_with(".js")   { return ContentType::JavaScript; }
    if path.ends_with(".css")  { return ContentType::CSS; }
    if path.ends_with(".html") { return ContentType::HTML; }
    if path.ends_with(".json") { return ContentType::JSON; }
    if path.ends_with(".svg")  { return ContentType::SVG; }
    if path.ends_with(".png")  { return ContentType::PNG; }
    if path.ends_with(".ico")  { return ContentType::Icon; }
    if path.ends_with(".woff") || path.ends_with(".woff2") {
        return ContentType::new("font", "woff2");
    }
    ContentType::Bytes
}

/// Serve frontend static assets at /assets/* (Vite outputs to dist/assets/)
#[get("/assets/<file..>")]
pub fn serve_asset(file: std::path::PathBuf) -> Result<(ContentType, Vec<u8>), Status> {
    let path = format!("assets/{}", file.display());
    match FrontendAssets::get(&path) {
        Some(content) => {
            let ct = content_type_for(&path);
            Ok((ct, content.data.into_owned()))
        }
        None => Err(Status::NotFound),
    }
}

/// Serve the React SPA index.html (used by human profile page requests).
pub fn spa_index_html() -> Option<Vec<u8>> {
    FrontendAssets::get("index.html")
        .map(|f| f.data.into_owned())
}
