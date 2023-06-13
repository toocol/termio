use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../resources/"]
#[include = "*.properties"]
#[include = "*.svg"]
#[include = "*.css"]
pub struct Asset;
