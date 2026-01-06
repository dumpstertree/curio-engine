use core::{
    collections::{
        color::Color,
        matrix4x4::Matrix4x4,
        quaternion::Quaternion,
        vector2::Vector2,
        vector3::Vector3,
    },
    io::{
        asset_loader::{AssetLoader, FontAsset},
        model_asset::ModelAsset,
    },
};
use std::{
    cell::RefMut,
    sync::Arc,
};

use hecs::World;

use crate::{
    component::{component_renderer_animated::RendererAnimated, component_renderer_static::Renderer},
    field_override::FieldDeserialize,
    world_context::{GameObject, WorldContext},
};

pub fn update(world: &mut WorldContext) {
    let borrow = world.world.borrow_mut();
    for w in borrow.iter() {
        if let Some(mut x) = w.get::<&mut ComponentRendererText>() {
            _ = x.update_enabled_in_heirarchy(&borrow);
        }
        if let Some(mut x) = w.get::<&mut RendererAnimated>() {
            _ = x.update_enabled_in_heirarchy(&borrow);
        }
        if let Some(mut x) = w.get::<&mut Renderer>() {
            _ = x.update_enabled_in_heirarchy(&borrow);
        }
    }
}

pub trait RendererCommon {
    fn set_cached_enabled_in_hierarchy(&mut self, val: bool);
    fn get_cached_enabled_in_hierarchy(&self) -> bool;

    fn set_cached_tint_in_hierarchy(&mut self, val: Color);
    fn get_cached_tint_in_hierarchy(&self) -> Color;

    // hierachy
    fn set_parent(&mut self, parent: Option<GameObject>);
    fn get_parent(&self) -> Option<GameObject>;
    // tint
    fn set_tint(&mut self, tint: Color);
    fn get_tint(&self) -> Color;
    // enabled
    fn set_enabled(&mut self, enabled: bool);
    fn get_enabled(&self) -> bool;
    //
    // fn tint_in_hierachy(&self, world: &World) -> Color {
    //     let mut tint = self.get_tint();
    //     let mut current = self.get_parent();
    //     while let Some(parent_entity) = &current {
    //         // if let Some(parent_renderer) = parent_entity.get_component::<&ComponentRendererText>() {
    //         //     tint = tint * parent_renderer.get_tint();
    //         //     current = parent_renderer.get_parent();
    //         // } else if let Some(parent_renderer) = parent_entity.get_component::<&Renderer>() {
    //         //     tint = tint * parent_renderer.get_tint();
    //         //     current = parent_renderer.get_parent();
    //         // } else if let Some(parent_renderer) = parent_entity.get_component::<&RendererAnimated>() {
    //         //     tint = tint * parent_renderer.get_tint();
    //         //     current = parent_renderer.get_parent();
    //         // }
    //         if let Some(parent_renderer) = parent_entity.get_component::<ComponentRendererText>() {
    //             tint = tint * parent_renderer.get_tint();
    //             current = parent_renderer.get_parent();
    //         } else if let Some(parent_renderer) = parent_entity.get_component::<Renderer>() {
    //             tint = tint * parent_renderer.get_tint();
    //             current = parent_renderer.get_parent();
    //         } else if let Some(parent_renderer) = parent_entity.get_component::<RendererAnimated>() {
    //             tint = tint * parent_renderer.get_tint();
    //             current = parent_renderer.get_parent();
    //         }
    //     }

    //     return tint;
    // }

    fn update_tint_in_heirarchy(&self, w: WorldContext) {
        // let b = w.world.borrow();

        // let x = b.get::<&ComponentRendererText>(self.get_parent().unwrap().entity);

        // let mut tint = self.get_tint();
        // let mut current = self.get_parent();
        // while let Some(parent_entity) = &current {
        //     if let Some(parent_renderer) = parent_entity.get_component::<ComponentRendererText>() {
        //         tint = tint * parent_renderer.get_cached_tint_in_hierarchy();
        //         current = parent_renderer.get_parent();
        //     } else if let Some(parent_renderer) = parent_entity.get_component::<Renderer>() {
        //         tint = tint * parent_renderer.get_cached_tint_in_hierarchy();
        //         current = parent_renderer.get_parent();
        //     } else if let Some(parent_renderer) = parent_entity.get_component::<RendererAnimated>() {
        //         tint = tint * parent_renderer.get_cached_tint_in_hierarchy();
        //         current = parent_renderer.get_parent();
        //     }
        // }

        // self.set_cached_tint_in_hierarchy(tint);
    }
    fn update(world: &mut WorldContext) {
        let borrow = world.world.borrow_mut();
        for w in borrow.iter() {
            if let Some(mut x) = w.get::<&mut ComponentRendererText>() {
                _ = x.update_enabled_in_heirarchy(&borrow);
            }
            if let Some(mut x) = w.get::<&mut RendererAnimated>() {
                _ = x.update_enabled_in_heirarchy(&borrow);
            }
            if let Some(mut x) = w.get::<&mut Renderer>() {
                _ = x.update_enabled_in_heirarchy(&borrow);
            }
        }
    }
    fn update_enabled_in_heirarchy(&mut self, world: &RefMut<'_, World>) -> bool {
        let is_enabled = self.get_enabled();

        if let Some(parent_entity) = &self.get_parent() {
            let mut parent_is_enabled = false;
            if let Ok(mut parent_renderer) = world.get::<&mut ComponentRendererText>(parent_entity.entity) {
                parent_is_enabled = parent_renderer.update_enabled_in_heirarchy(world);
            }
            if let Ok(mut parent_renderer) = world.get::<&mut Renderer>(parent_entity.entity) {
                parent_is_enabled = parent_renderer.update_enabled_in_heirarchy(world);
            }
            if let Ok(mut parent_renderer) = world.get::<&mut RendererAnimated>(parent_entity.entity) {
                parent_is_enabled = parent_renderer.update_enabled_in_heirarchy(world);
            }

            self.set_cached_enabled_in_hierarchy(is_enabled && parent_is_enabled);
            return is_enabled && parent_is_enabled;
        } else {
            self.set_cached_enabled_in_hierarchy(is_enabled);
            return is_enabled;
        }
    }
    // fn enabled_in_hierarchy(&self, world: &WorldContext) -> bool {
    //     if !self.get_enabled() {
    //         return false;
    //     }

    //     let mut current = self.get_parent();
    //     while let Some(parent_entity) = &current {
    //         if let Some(parent_renderer) = parent_entity.get_component::<ComponentRendererText>() {
    //             if !parent_renderer.get_enabled() {
    //                 return false;
    //             } else {
    //                 current = parent_renderer.get_parent();
    //             }
    //         } else if let Some(parent_renderer) = parent_entity.get_component::<Renderer>() {
    //             if !parent_renderer.get_enabled() {
    //                 return false;
    //             } else {
    //                 current = parent_renderer.get_parent();
    //             }
    //         } else if let Some(parent_renderer) = parent_entity.get_component::<RendererAnimated>() {
    //             if !parent_renderer.get_enabled() {
    //                 return false;
    //             } else {
    //                 current = parent_renderer.get_parent();
    //             }
    //         } else {
    //             current = None;
    //         }
    //     }

    //     return true;
    // }
}

unsafe impl Sync for ComponentRendererText {}
unsafe impl Send for ComponentRendererText {}

#[derive(Clone)]
pub struct ComponentRendererText {
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
    parent: Option<GameObject>,
    tint: Color,
}
impl FieldDeserialize for ComponentRendererText {
    fn override_field(&mut self, field: &str, value: &str) {
        match field {
            "asset" => self.font_asset = Some(AssetLoader::load_font_asset(value)),
            "contents" => self.contents = value.to_string(),
            "enabled" => self.enabled = value.parse().unwrap_or_default(),
            "font_size" => self.font_size = value.parse().unwrap_or_default(),
            "bounds" => self.bounds = value.parse().unwrap_or_default(),
            _ => {}
        }
    }
}

impl RendererCommon for ComponentRendererText {
    fn set_parent(&mut self, parent: Option<GameObject>) {
        self.parent = parent;
    }

    fn get_parent(&self) -> Option<GameObject> {
        self.parent.clone()
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

impl Default for ComponentRendererText {
    fn default() -> ComponentRendererText {
        ComponentRendererText {
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
            parent: None,
            tint: Color::white(),
        }
    }
}
impl ComponentRendererText {
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
            self.font_asset = Some(AssetLoader::load_font_asset("assets/default.font"));
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
