use curio_core::{
    Color, FontAsset, Matrix4x4, ModelAsset, Quaternion, Vector2, Vector3,
    io::asset_loader::{ASSET_UID_FONT_ASSET_DEFAULT, AssetLoader},
};
use std::sync::Arc;

use crate::{
    built_in::facet::renderer_common::RendererCommon,
    form::Form,
    traits::{facet_common::FacetCommon, field_override::FieldOverride},
};

// pub fn update(world: &mut Context3D) {
//     let borrow = world.world.borrow_mut();
//     for w in borrow.iter() {
//         if let Some(mut x) = w.get::<&mut RendererText>() {
//             _ = x.update_enabled_in_heirarchy(&borrow);
//         }
//         if let Some(mut x) = w.get::<&mut RendererDynamic>() {
//             _ = x.update_enabled_in_heirarchy(&borrow);
//         }
//         if let Some(mut x) = w.get::<&mut RendererStatic>() {
//             _ = x.update_enabled_in_heirarchy(&borrow);
//         }
//     }
// }

unsafe impl Sync for RendererText {}
unsafe impl Send for RendererText {}

#[derive(Clone)]
pub struct RendererText {
    cached_enabled_in_hierachy: bool,
    cached_tint_in_hierachy: Color,

    pub asset: Vec<(Arc<ModelAsset>, Vec<Matrix4x4>)>,
    font_asset: Option<Arc<FontAsset>>,
    contents: String,
    align_horizontal: AligmentHorizontal,
    align_vertical: AligmentVertical,
    font_size: f32,
    bounds: Vector2,
    is_dirty: bool,
    enabled: bool,
    tint: Color,
    owner: Option<Form>,
}
impl FacetCommon for RendererText {
    fn set_ownership(&mut self, owner: Form) {
        self.owner = Some(owner);
    }
    fn form(&self) -> Form {
        self.owner.clone().unwrap()
    }
}
impl FieldOverride for RendererText {
    fn apply(&mut self, field: &str, value: &str) {
        match field {
            // "asset" => self.font_asset = Some(AssetLoader::load_font_asset(value)),
            "asset" => self.font_asset = Some(AssetLoader::load_asset::<FontAsset>(&AssetLoader::try_lookup_key_for_name(value).unwrap())),
            "contents" => self.contents = value.to_string(),
            "enabled" => self.enabled = value.parse().unwrap_or_default(),
            "font_size" => self.font_size = value.parse().unwrap_or_default(),
            "bounds" => self.bounds = value.parse().unwrap_or_default(),
            _ => {}
        }
    }
}

impl RendererCommon for RendererText {
    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn get_enabled(&self) -> bool {
        self.enabled
    }

    fn set_tint(&mut self, tint: Color) {
        self.tint = tint;
    }

    fn get_tint(&self) -> Color {
        self.tint
    }

    fn set_cached_enabled_in_hierarchy(&mut self, val: bool) {
        self.cached_enabled_in_hierachy = val;
    }

    fn get_cached_enabled_in_hierarchy(&self) -> bool {
        self.cached_enabled_in_hierachy
    }

    fn set_cached_tint_in_hierarchy(&mut self, val: Color) {
        self.cached_tint_in_hierachy = val;
    }

    fn get_cached_tint_in_hierarchy(&self) -> Color {
        self.cached_tint_in_hierachy
    }
}

impl Default for RendererText {
    fn default() -> RendererText {
        RendererText {
            cached_enabled_in_hierachy: false,
            cached_tint_in_hierachy: Color::white(),
            asset: Vec::new(),
            font_asset: None,
            contents: String::from("Lorum ipsum..."),
            align_horizontal: AligmentHorizontal::Center,
            align_vertical: AligmentVertical::Center,
            font_size: 0.05,
            bounds: Vector2::new(1.0, 1.0),
            is_dirty: true,
            enabled: true,
            tint: Color::white(),
            owner: None,
        }
    }
}
impl RendererText {
    pub fn set_enabled(&mut self, enabled: bool) -> &mut Self {
        if self.enabled == enabled {
            return self;
        }
        self.enabled = enabled;
        self
    }
    pub fn set_font_asset(&mut self, font_asset: Option<Arc<FontAsset>>) -> &mut Self {
        // if self.font_asset == font_asset {
        //     return self;
        // }

        self.font_asset = font_asset;
        self.is_dirty = true;
        self
    }
    pub fn set_contents(&mut self, contents: &str) -> &mut Self {
        if self.contents == contents {
            return self;
        }

        self.contents = String::from(contents);
        self.is_dirty = true;
        self
    }
    pub fn set_horizontal_alignment(&mut self, align: AligmentHorizontal) -> &mut Self {
        if self.align_horizontal == align {
            return self;
        }
        self.align_horizontal = align;
        self.is_dirty = true;
        self
    }
    pub fn set_vertical_alignment(&mut self, align: AligmentVertical) -> &mut Self {
        if self.align_vertical == align {
            return self;
        }
        self.align_vertical = align;
        self.is_dirty = true;
        self
    }
    pub fn set_font_size(&mut self, font_size: f32) -> &mut Self {
        if self.font_size == font_size {
            return self;
        }
        self.font_size = font_size;
        self.is_dirty = true;
        self
    }
    pub fn set_bounds(&mut self, bounds: Vector2) -> &mut Self {
        if self.bounds == bounds {
            return self;
        }

        self.bounds = bounds;
        self.is_dirty = true;
        self
    }

    pub fn rebuild(&mut self) {
        if !self.is_dirty {
            return;
        }

        self.is_dirty = false;
        // let font_asset = self
        //     .font_asset
        //     .clone()
        //     .unwrap_or_else(|| AssetLoader::load_font_asset("assets/default.font"));

        if self.font_asset.is_none() {
            self.font_asset = Some(AssetLoader::load_asset::<FontAsset>(&ASSET_UID_FONT_ASSET_DEFAULT));
        }
        let Some(font_asset) = &self.font_asset else {
            return;
        };

        let mut output: Vec<(Arc<ModelAsset>, Vec<Matrix4x4>)> = Vec::new();

        // let advance = self.font_size + (self.font_size * font_asset.char_spacing);
        // let line_height = self.font_size + (self.font_size * font_asset.line_spacing);
        let advance = self.font_size + (self.font_size * font_asset.glyph_width());
        let line_height = self.font_size + (self.font_size * font_asset.glyph_height());

        // === Step 1: Preprocess into wrapped lines ===
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0.0;
        let mut total_height = line_height;

        for word in self.contents.split_inclusive(' ') {
            let word_width = word.chars().filter(|c| *c != ' ').count() as f32 * advance;

            if current_width + word_width > self.bounds.x {
                // Word doesn't fit: push line and start a new one
                lines.push(current_line.trim_end().to_string());
                current_line = String::new();
                current_width = 0.0;
                total_height += line_height;

                // Truncate vertically if no space for new line
                if total_height > self.bounds.y {
                    break;
                }

                // Skip leading space if the next word starts with one
                if word.starts_with(' ') {
                    continue;
                }
            }

            current_line.push_str(word);
            current_width += word_width;
        }

        if !current_line.is_empty() && total_height <= self.bounds.y {
            lines.push(current_line.trim_end().to_string());
        }

        // === Step 2: Layout and create quads ===
        for (line_idx, line) in lines.iter().enumerate() {
            let mut cursor_x = 0.0;
            let cursor_y = -(line_idx as f32) * line_height; // going downward

            let trimmed = line.trim_end();
            let total_line_width = trimmed.chars().count() as f32 * advance;

            for (i, ch) in line.chars().enumerate() {
                if ch == ' ' && cursor_x == 0.0 {
                    continue; // skip leading spaces
                }
                if ch == ' ' && i == line.len() - 1 {
                    continue; // skip trailing spaces
                }
                if ch == ' ' {
                    cursor_x += advance;
                    continue;
                }

                // Create/reuse quad mesh
                let mesh_arc = font_asset.mesh_for_char(ch);
                let x_offset: f32 = match self.align_horizontal {
                    AligmentHorizontal::Center => -total_line_width * 0.5,
                    AligmentHorizontal::Left => -self.bounds.x * 0.5,
                    AligmentHorizontal::Right => self.bounds.x * 0.5 - total_line_width,
                };

                let y_offset: f32;
                let total_text_height = (lines.len() as f32) * line_height;

                match self.align_vertical {
                    AligmentVertical::Center => {
                        y_offset = total_text_height * 0.5 - line_height * 0.5;
                    }
                    AligmentVertical::Top => {
                        y_offset = self.bounds.y * 0.5;
                    }
                    AligmentVertical::Bottom => {
                        y_offset = self.bounds.y * -0.5 + total_text_height + line_height;
                    }
                }

                // build matrix taking into account offsets in each direection
                let transform = Matrix4x4::new(Vector3::new(x_offset + cursor_x, y_offset + cursor_y, 0.0), Quaternion::zero(), Vector3::new(self.font_size, self.font_size, 1.0));
                // Add transform to output
                if let Some((_mesh, matrices)) = output.iter_mut().find(|(m, _)| Arc::ptr_eq(m, &mesh_arc)) {
                    matrices.push(transform);
                } else {
                    output.push((mesh_arc, vec![transform]));
                }

                cursor_x += advance;
            }
        }

        // === Step 3: Upload results ===
        self.asset = output;
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum AligmentHorizontal {
    Left,
    #[default]
    Center,
    Right,
}
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum AligmentVertical {
    Top,
    #[default]
    Center,
    Bottom,
}
