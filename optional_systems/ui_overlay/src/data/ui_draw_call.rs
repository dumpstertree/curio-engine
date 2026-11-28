use crate::UiKind;
use crate::UiRect;
use curio_core::{Color, Vector2};

/// One painted box in the UI overlay, produced every frame by the
/// `update_ui_layout` habit and consumed by `UiOverlayPass`.
#[derive(Clone, PartialEq, Debug)]
pub struct UiDrawCall {
    /// the upright box in pixels; its centre is where the turned box sits on screen
    pub rect: UiRect,
    /// radians, counter-clockwise on screen, about the rect's centre
    pub rotation: f32,
    /// applied about the rect's centre before the rotation
    pub scale: Vector2,
    /// hover/click feedback is already folded in
    pub color: Color,
    pub kind: UiKind,
    /// painter's order within the screen, from `Transform3D.position.z` — lower draws first
    pub depth: f32,
    /// label painted from the shared font atlas inside this box. Empty
    /// means no text; monospace lays out one advance per character
    pub text: String,
    /// glyph height in pixels — the atlas's native cells are 12 px. 0
    /// means "don't paint text"
    pub font_size: f32,
    /// texture asset uid an Image box paints, tinted by `color`; None paints it flat
    pub image: Option<i16>,
    /// which taffy node produced this, for hit-test/debug tooling
    pub instance_id: i32,
    /// taffy's own paint order — the tie-breaker when depth matches
    pub order: u32,
}

impl UiDrawCall {
    pub fn new(rect: UiRect, color: Color, kind: UiKind, depth: f32, text: String, font_size: f32, instance_id: i32, order: u32) -> UiDrawCall {
        UiDrawCall {
            rect,
            rotation: 0.0,
            scale: Vector2::one(),
            color,
            kind,
            depth,
            text,
            font_size,
            image: None,
            instance_id,
            order,
        }
    }
    pub fn turned(mut self, rotation: f32, scale: Vector2) -> UiDrawCall {
        self.rotation = rotation;
        self.scale = scale;
        self
    }
    pub fn with_image(mut self, image: Option<i16>) -> UiDrawCall {
        self.image = image;
        self
    }
    /// The box's four on-screen corners in pixels: top-left, top-right, bottom-right, bottom-left.
    pub fn corners(&self) -> [Vector2; 4] {
        self.rect.turned_corners(self.rotation, self.scale)
    }
    /// Is a screen-space point inside the box as it is drawn?
    pub fn contains(&self, x: f32, y: f32) -> bool {
        self.rect.contains_turned(x, y, self.rotation, self.scale)
    }
}
