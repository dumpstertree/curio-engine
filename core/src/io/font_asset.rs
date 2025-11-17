use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct FontAsset {
    pub texture_path: String,
    pub shader_path: String,
    // spacing
    pub char_spacing: f32,
    pub line_spacing: f32,
    pub padding_left: i32,
    pub padding_right: i32,
    pub padding_top: i32,
    pub padding_bottom: i32,
    // spacing
    pub columns: i32,
    pub rows: i32,
    // chars
    pub char_order: String,
}
