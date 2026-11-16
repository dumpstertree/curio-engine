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

/// the atlas grid: where each glyph's cell lives in the texture.
/// v grows downward in texture space, the same way y grows downward in UI
/// pixels — so a cell's top edge is its v_min
pub struct UiFontGrid {
    pub columns: i32,
    pub rows: i32,
    pub u_left: f32,
    pub v_top: f32,
    pub cell_width: f32,
    pub cell_height: f32,
    pub char_order: String,
}
impl UiFontGrid {
    pub fn new(desc: UiFontDesc, width: u32, height: u32) -> UiFontGrid {
        let w = width as f32;
        let h = height as f32;
        UiFontGrid {
            columns: desc.columns,
            rows: desc.rows,
            u_left: desc.padding_left as f32 / w,
            v_top: desc.padding_top as f32 / h,
            cell_width:
                (1.0 - desc.padding_left as f32 / w - desc.padding_right as f32 / w) / desc.columns as f32,
            cell_height:
                (1.0 - desc.padding_top as f32 / h - desc.padding_bottom as f32 / h) / desc.rows as f32,
            char_order: desc.char_order,
        }
    }
    // cell rect [u_min, v_top, u_max, v_bottom] — None for a char the
    // atlas has no cell for
    pub fn glyph_uv(&self, ch: char) -> Option<[f32; 4]> {
        let mut index: i32 = -1;
        for (position, found) in self.char_order.chars().enumerate() {
            if found == ch {
                index = position as i32;
                break;
            }
        }
        if index < 0 {
            return None;
        }
        let col = (index % self.columns) as f32;
        let row = (index / self.columns) as f32;
        Some([
            self.u_left + col * self.cell_width,
            self.v_top + row * self.cell_height,
            self.u_left + (col + 1.0) * self.cell_width,
            self.v_top + (row + 1.0) * self.cell_height,
        ])
    }
}
