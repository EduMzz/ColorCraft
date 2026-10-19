pub mod capture;
pub mod catalog;
pub mod color;
pub mod composition;
pub mod diagnostics;
pub mod engine;
pub mod export;
pub mod model;
pub mod solver;
pub use model::*;
pub type Result<T> = std::result::Result<T, String>;
pub fn profile_name(bytes: &[u8]) -> Result<String> {
    craft_cms::Profile::parse(bytes)
        .map(|p| p.description)
        .map_err(|e| e.to_string())
}
pub fn profile_space(bytes: &[u8]) -> Result<String> {
    let p = craft_cms::Profile::parse(bytes).map_err(|e| e.to_string())?;
    Ok(match p.color_space {
        craft_cms::ColorSpace::Rgb => "srgb",
        craft_cms::ColorSpace::Cmyk => "cmyk",
        craft_cms::ColorSpace::Lab => "lab",
        craft_cms::ColorSpace::Gray => "gray",
        _ => {
            return Err(
                "Este modelo de perfil não pode representar uma cor editável nesta versão.".into(),
            );
        }
    }
    .into())
}
