use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "resources/"]
#[include = "*.svg"]
#[include = "*.json"]
pub struct Asset;