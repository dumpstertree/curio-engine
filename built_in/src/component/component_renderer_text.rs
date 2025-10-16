use core::{
    collections::{
        color::Color,
        material::Material,
        matrix4x4::Matrix4x4,
        mesh::{Mesh, Vertex},
        quaternion::Quaternion,
        vector2::Vector2,
        vector3::Vector3,
    },
    io::{asset_loader::AssetLoader, file::File, font_asset::FontAsset, model_asset::ModelAsset},
};
use std::{collections::HashMap, sync::Arc};

use hecs::{Entity, World};

use crate::component::{component_renderer_animated::RendererAnimated, component_renderer_static::Renderer};

pub trait RendererCommon {
    // hierachy
    fn set_parent(&mut self, parent: Option<Entity>);
    fn get_parent(&self) -> Option<Entity>;
    // tint
    fn set_tint(&mut self, tint: Color);
    fn get_tint(&self) -> Color;
    // enabled
    fn set_enabled(&mut self, enabled: bool);
    fn get_enabled(&self) -> bool;
    //
    fn tint_in_hierachy(&self, world: &World) -> Color {
        let mut tint = self.get_tint();
        let mut parent_entity = self.get_parent();
        while parent_entity.is_some() {
            if let Ok(parent_renderer) = world.get::<&ComponentRendererText>(parent_entity.unwrap()) {
                tint = tint * parent_renderer.get_tint();
                parent_entity = parent_renderer.get_parent();
            } else if let Ok(parent_renderer) = world.get::<&Renderer>(parent_entity.unwrap()) {
                tint = tint * parent_renderer.get_tint();
                parent_entity = parent_renderer.get_parent();
            } else if let Ok(parent_renderer) = world.get::<&RendererAnimated>(parent_entity.unwrap()) {
                tint = tint * parent_renderer.get_tint();
                parent_entity = parent_renderer.get_parent();
            }
        }

        return tint;
    }
    fn enabled_in_hierarchy(&self, world: &World) -> bool {
        if !self.get_enabled() {
            return false;
        }

        let mut parent_entity = self.get_parent();
        while parent_entity.is_some() {
            if let Ok(parent_renderer) = world.get::<&ComponentRendererText>(parent_entity.unwrap()) {
                if !parent_renderer.get_enabled() {
                    return false;
                } else {
                    parent_entity = parent_renderer.get_parent();
                }
            } else if let Ok(parent_renderer) = world.get::<&Renderer>(parent_entity.unwrap()) {
                if !parent_renderer.get_enabled() {
                    return false;
                } else {
                    parent_entity = parent_renderer.get_parent();
                }
            } else if let Ok(parent_renderer) = world.get::<&RendererAnimated>(parent_entity.unwrap()) {
                if !parent_renderer.get_enabled() {
                    return false;
                } else {
                    parent_entity = parent_renderer.get_parent();
                }
            }
        }

        return true;
    }
}
pub struct ComponentRendererText {
    pub asset: Vec<(Arc<ModelAsset>, Vec<Matrix4x4>)>,
    font_asset: Option<FontAsset>,
    contents: String,
    align_horizontal: AligmentHorizontal,
    align_vertical: AligmentVertical,
    font_size: f32,
    bounds: Vector2,
    is_dirty: bool,
    enabled: bool,
    parent: Option<Entity>,
    tint: Color,
}
impl RendererCommon for ComponentRendererText {
    fn set_parent(&mut self, parent: Option<Entity>) {
        self.parent = parent;
    }

    fn get_parent(&self) -> Option<Entity> {
        self.parent
    }

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
}

impl ComponentRendererText {
    pub fn default() -> ComponentRendererText {
        ComponentRendererText {
            asset: Vec::new(),
            font_asset: None,
            contents: String::from("Lorum ipsum..."),
            align_horizontal: AligmentHorizontal::Center,
            align_vertical: AligmentVertical::Center,
            font_size: 0.05,
            bounds: Vector2::new(1.0, 1.0),
            is_dirty: true,
            enabled: true,
            parent: None,
            tint: Color::white(),
        }
    }
    pub fn set_enabled(&mut self, enabled: bool) -> &mut Self {
        self.enabled = enabled;
        self
    }
    pub fn set_font_asset(&mut self, font_asset: Option<FontAsset>) -> &mut Self {
        self.font_asset = font_asset;
        self.is_dirty = true;
        self
    }
    pub fn set_contents(&mut self, contents: &str) -> &mut Self {
        self.contents = String::from(contents);
        self.is_dirty = true;
        self
    }
    pub fn set_horizontal_alignment(&mut self, align: AligmentHorizontal) -> &mut Self {
        self.align_horizontal = align;
        self.is_dirty = true;
        self
    }
    pub fn set_vertical_alignment(&mut self, align: AligmentVertical) -> &mut Self {
        self.align_vertical = align;
        self.is_dirty = true;
        self
    }
    pub fn set_font_size(&mut self, font_size: f32) -> &mut Self {
        self.font_size = font_size;
        self.is_dirty = true;
        self
    }
    pub fn set_bounds(&mut self, bounds: Vector2) -> &mut Self {
        self.bounds = bounds;
        self.is_dirty = true;
        self
    }

    pub fn rebuild(&mut self) {
        if !self.is_dirty {
            return;
        }
        self.is_dirty = false;
        let font_asset: FontAsset = self
            .font_asset
            .clone()
            .unwrap_or_else(|| AssetLoader::load_font_asset_from_path("assets/default.font"));

        let texture = AssetLoader::load_texture_from_path(&File::join_path(&File::get_built_in_asset_path(), &font_asset.texture_path));
        let shader = AssetLoader::load_shader_desc(&File::join_path(&File::get_built_in_asset_path(), &font_asset.shader_path));

        let w = texture.texture.width() as f32;
        let h = texture.texture.height() as f32;

        let mut material = Material::new(shader);
        material.set_texture_with_label(Some(texture), "diffuse");
        let material = Arc::new(material);

        // Padding → normalized
        let padding_left_01 = font_asset.padding_left as f32 / w;
        let padding_right_01 = font_asset.padding_right as f32 / w;
        let padding_top_01 = font_asset.padding_top as f32 / h;
        let padding_bottom_01 = font_asset.padding_bottom as f32 / h;

        // Glyph UV layout
        let glyph_width = (1.0 - padding_left_01 - padding_right_01) / font_asset.columns as f32;
        let glyph_height = (1.0 - padding_top_01 - padding_bottom_01) / font_asset.rows as f32;

        let mut quad_cache: HashMap<char, Arc<Mesh>> = HashMap::new();
        let mut output: Vec<(Arc<Mesh>, Vec<Matrix4x4>)> = Vec::new();

        let advance = self.font_size + (self.font_size * font_asset.char_spacing);
        let line_height = self.font_size + (self.font_size * font_asset.line_spacing);

        // === Step 1: Preprocess into wrapped lines ===
        let mut lines: Vec<String> = Vec::new();
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

                // Find glyph index in atlas
                let index = font_asset.char_order.find(|c| c == ch);
                let Some(index) = index else {
                    cursor_x += advance;
                    continue;
                };

                let col = index as i32 % font_asset.columns;
                let row = index as i32 / font_asset.columns;

                let u_min = padding_left_01 + col as f32 * glyph_width;
                let v_min = padding_top_01 + row as f32 * glyph_height;
                let u_max = u_min + glyph_width;
                let v_max = v_min + glyph_height;

                // Create/reuse quad mesh
                let mesh_arc = quad_cache.entry(ch).or_insert_with(|| {
                    let vertices = vec![
                        Vertex {
                            position: [0.0, 0.0, 0.0],
                            normal: [0.0, 0.0, 1.0],
                            color: [1.0, 1.0, 1.0, 1.0],
                            uv0: [u_min, v_max],
                            uv1: [0.0, 0.0],
                        },
                        Vertex {
                            position: [self.font_size, 0.0, 0.0],
                            normal: [0.0, 0.0, 1.0],
                            color: [1.0, 1.0, 1.0, 1.0],
                            uv0: [u_max, v_max],
                            uv1: [0.0, 0.0],
                        },
                        Vertex {
                            position: [self.font_size, self.font_size, 0.0],
                            normal: [0.0, 0.0, 1.0],
                            color: [1.0, 1.0, 1.0, 1.0],
                            uv0: [u_max, v_min],
                            uv1: [0.0, 0.0],
                        },
                        Vertex {
                            position: [0.0, self.font_size, 0.0],
                            normal: [0.0, 0.0, 1.0],
                            color: [1.0, 1.0, 1.0, 1.0],
                            uv0: [u_min, v_min],
                            uv1: [0.0, 0.0],
                        },
                    ];
                    Arc::new(Mesh::new(format!("glyph_{}", ch), vertices, vec![0, 1, 2, 0, 2, 3], Matrix4x4::default()))
                });
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
                let transform = Matrix4x4::new(Vector3::new(x_offset + cursor_x, y_offset + cursor_y, 0.0), Quaternion::zero(), Vector3::new(1.0, 1.0, 1.0));
                // Add transform to output
                if let Some((_mesh, matrices)) = output.iter_mut().find(|(m, _)| Arc::ptr_eq(m, mesh_arc)) {
                    matrices.push(transform);
                } else {
                    output.push((mesh_arc.clone(), vec![transform]));
                }

                cursor_x += advance;
            }
        }

        // === Step 3: Upload results ===
        self.asset.clear();
        for x in &output {
            let mesh = vec![x.0.clone()];
            let material = vec![material.clone()];
            let asset = Arc::new(ModelAsset::new(mesh, material));
            self.asset.push((asset, x.1.clone()));
        }
    }
}

pub enum AligmentHorizontal {
    Left,
    Center,
    Right,
}
pub enum AligmentVertical {
    Top,
    Center,
    Bottom,
}
