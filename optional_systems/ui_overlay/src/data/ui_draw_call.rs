use crate::UiKind;
use crate::UiRect;
use curio_core::Color;

/// One painted box in the UI overlay, produced every frame by the
/// `update_ui_layout` habit and consumed by `UiOverlayPass`.
#[derive(Clone, PartialEq, Debug)]
pub struct UiDrawCall {
    /// where the box landed on its screen, in pixels
    pub rect: UiRect,
    /// hover/click feedback is already folded in
    pub color: Color,
    pub kind: UiKind,
    /// painter's order within the screen — lower draws first
    pub z_index: i32,
    /// label painted from the shared font atlas inside this box. Empty
    /// means no text; monospace lays out one advance per character
    pub text: String,
    /// glyph height in pixels — the atlas's native cells are 12 px. 0
    /// means "don't paint text"
    pub font_size: f32,
    /// which taffy node produced this, for hit-test/debug tooling
    pub instance_id: i32,
    /// taffy's own paint order — the tie-breaker when z_index matches
    pub order: u32,
}

impl UiDrawCall {
    pub fn new(rect: UiRect, color: Color, kind: UiKind, z_index: i32, text: String, font_size: f32, instance_id: i32, order: u32) -> UiDrawCall {
        UiDrawCall { rect, color, kind, z_index, text, font_size, instance_id, order }
    }
}
