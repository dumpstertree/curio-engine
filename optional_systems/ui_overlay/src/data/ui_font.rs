// the font atlas is shared: 3D text samples the built-in
// ASSET_UID_TEXTURE_FONT_ATLAS texture through curio_core, and lays glyphs
// out against the grid in default.font. ui_overlay binds that same texture
// and parses that same JSON — embedding the file at compile time keeps this
// crate independent of the 3D rendering crates, so only the convention is
// shared, not the draw-call plumbing

use serde::Deserialize;
use serde_json::{self, from_str};

// the exact file the 3D FontAsset parses
pub const DEFAULT_FONT: &[u8] = include_bytes!("../../../../assets/built_in/font/default.font");

#[derive(Clone, Debug, Deserialize)]
pub struct UiFontDesc {
    pub texture_path: String,
    pub shader_path: String,
    pub char_spacing: f32,
    pub line_spacing: f32,
    pub padding_left: i32,
    pub padding_right: i32,
    pub padding_top: i32,
    pub padding_bottom: i32,
    pub columns: i32,
    pub rows: i32,
    pub char_order: String,
}

impl UiFontDesc {
    pub fn parse(bits: &[u8]) -> Option<UiFontDesc> {
        let contents = match String::from_utf8(bits.to_vec()) {
            Ok(found) => found,
            Err(_) => return None,
        };
        match from_str::<UiFontDesc>(&contents) {
            Ok(found) => Some(found),
            Err(_) => None,
        }
    }
}
