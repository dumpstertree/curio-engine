use curio_core::{ASSET_UID_FONT_ASSET_DEFAULT, Assets, Color, FieldState, Matrix4x4, Quaternion, Vector2, Vector3};
use facet::facet;
use gameplay::traits::field_override::FieldOverride;
use rendering_static::ModelAsset;
use std::sync::Arc;

use crate::FontAsset;

#[facet]
pub struct RendererText {
    pub asset: Vec<(Arc<ModelAsset>, Vec<Matrix4x4>)>,
    font_asset: Option<Arc<FontAsset>>,
    contents: String,
    align_horizontal: AligmentHorizontal,
    align_vertical: AligmentVertical,
    font_size: f32,
    bounds: Vector2,
    is_dirty: bool,
    pub enabled: bool,
    pub tint: Color,
}
impl FieldOverride for RendererText {
    fn apply(&mut self, field: &str, value: &str) {
        match field {
            // "asset" => self.font_asset = Some(AssetLoader::load_font_asset(value)),
            "asset" => {
                self.font_asset = Some(Assets::load_asset::<FontAsset>(&Assets::try_lookup_key_for_name(value).unwrap()));
                self.is_dirty = true;
            }
            "contents" => {
                if self.contents != value {
                    self.contents = value.to_string();
                    self.is_dirty = true;
                }
            }
            "enabled" => self.enabled = value.parse().unwrap_or_default(),
            "font_size" => {
                let font_size = value.trim().parse().unwrap_or_default();
                if self.font_size != font_size {
                    self.font_size = font_size;
                    self.is_dirty = true;
                }
            }
            "bounds" => {
                let bounds = value.trim().parse().unwrap_or_default();
                if self.bounds != bounds {
                    self.bounds = bounds;
                    self.is_dirty = true;
                }
            }
            "tint" => self.tint = value.trim().parse().unwrap_or_default(),
            "align_horizontal" => {
                let align = match value.trim().to_ascii_lowercase().as_str() {
                    "left" => AligmentHorizontal::Left,
                    "right" => AligmentHorizontal::Right,
                    _ => AligmentHorizontal::Center,
                };
                if self.align_horizontal != align {
                    self.align_horizontal = align;
                    self.is_dirty = true;
                }
            }
            "align_vertical" => {
                let align = match value.trim().to_ascii_lowercase().as_str() {
                    "top" => AligmentVertical::Top,
                    "bottom" => AligmentVertical::Bottom,
                    _ => AligmentVertical::Center,
                };
                if self.align_vertical != align {
                    self.align_vertical = align;
                    self.is_dirty = true;
                }
            }
            _ => {}
        }
    }
    fn get_state(&self) -> Vec<FieldState> {
        // NOTE: font_asset is intentionally not serialized here. A placeholder value would be
        // fed back through apply() as a font name lookup and panic (see FormBuilder3D::facet).
        vec![
            FieldState::new("contents", self.contents.clone()),
            FieldState::new("enabled", self.enabled),
            FieldState::new("font_size", self.font_size),
            FieldState::new("bounds", self.bounds),
            FieldState::new("tint", self.tint),
            FieldState::new("align_horizontal", format!("{:?}", self.align_horizontal).to_lowercase()),
            FieldState::new("align_vertical", format!("{:?}", self.align_vertical).to_lowercase()),
        ]
    }
}

impl RendererText {
    pub fn default() -> RendererText {
        RendererText {
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
    /// Create a builder for a RendererText. Follows the same pattern as Camera::builder() and
    /// RendererStatic::builder(). A newly built RendererText is always rebuilt once by the
    /// rendering habit, so no manual dirty flagging is needed at construction time.
    pub fn builder() -> RendererTextBuilder {
        RendererTextBuilder {
            font_asset: None,
            contents: String::from("Lorum ipsum..."),
            align_horizontal: AligmentHorizontal::Center,
            align_vertical: AligmentVertical::Center,
            font_size: 0.05,
            bounds: Vector2::new(1.0, 1.0),
            enabled: true,
            tint: Color::white(),
        }
    }
}

pub struct RendererTextBuilder {
    font_asset: Option<Arc<FontAsset>>,
    contents: String,
    align_horizontal: AligmentHorizontal,
    align_vertical: AligmentVertical,
    font_size: f32,
    bounds: Vector2,
    enabled: bool,
    tint: Color,
}
impl RendererTextBuilder {
    pub fn font_asset(mut self, font_asset: Option<Arc<FontAsset>>) -> Self {
        self.font_asset = font_asset;
        self
    }
    pub fn contents(mut self, contents: &str) -> Self {
        self.contents = String::from(contents);
        self
    }
    pub fn horizontal_alignment(mut self, align: AligmentHorizontal) -> Self {
        self.align_horizontal = align;
        self
    }
    pub fn vertical_alignment(mut self, align: AligmentVertical) -> Self {
        self.align_vertical = align;
        self
    }
    pub fn font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }
    pub fn bounds(mut self, bounds: Vector2) -> Self {
        self.bounds = bounds;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn tint(mut self, tint: Color) -> Self {
        self.tint = tint;
        self
    }
    pub fn build(self) -> RendererText {
        RendererText {
            asset: Vec::new(),
            font_asset: self.font_asset,
            contents: self.contents,
            align_horizontal: self.align_horizontal,
            align_vertical: self.align_vertical,
            font_size: self.font_size,
            bounds: self.bounds,
            is_dirty: true,
            enabled: self.enabled,
            tint: self.tint,
            owner: None,
        }
    }
}

impl RendererText {
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
            self.font_asset = Some(Assets::load_asset::<FontAsset>(&ASSET_UID_FONT_ASSET_DEFAULT));
        }
        let Some(font_asset) = &self.font_asset else {
            println!("font asset is none");
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

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum AligmentHorizontal {
    Left,
    #[default]
    Center,
    Right,
}
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum AligmentVertical {
    Top,
    #[default]
    Center,
    Bottom,
}
