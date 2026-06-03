use rust_embed::Embed;

#[derive(Embed)]
#[folder = "assets/"]
pub struct Assets;

/// Get an asset's content as a string
pub fn get_asset_text(path: &str) -> Option<String> {
    Assets::get(path).map(|f| String::from_utf8_lossy(f.data.as_ref()).into_owned())
}

/// Get an asset's content as bytes
pub fn get_asset_bytes(path: &str) -> Option<Vec<u8>> {
    Assets::get(path).map(|f| f.data.into_owned())
}

/// Guess MIME type from file extension
pub fn mime_type(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("css") => "text/css",
        Some("js") | Some("mjs") => "application/javascript",
        Some("html") => "text/html",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        Some("ttf") => "font/ttf",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    }
}
