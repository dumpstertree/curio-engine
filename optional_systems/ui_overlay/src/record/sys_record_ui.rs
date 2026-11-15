use crate::UiDrawCall;
use curio_core::{FieldState, RecordOverride, RecordScope, Vector2};
use record::record;

#[record(name = "Ui", ownership = RecordScope::Instance)]
pub struct SysRecordUi {
    /// every box this screen paints this frame, in the order it is drawn
    pub draw_calls: Vec<UiDrawCall>,
    /// the size of this screen in pixels. The layout habit falls back to the
    /// capture size when this is left at zero.
    pub viewport: Vector2,
    /// the pixel space the `AxisCode::Cursor` arrives in — usually the window
    /// size. Zero falls back to the capture size. Boxes are laid out in
    /// `viewport` pixels, so the hit-test scales the cursor by the two.
    pub cursor_space: Vector2,
}

impl SysRecordUi {
    pub fn default() -> SysRecordUi {
        SysRecordUi {
            draw_calls: Vec::new(),
            viewport: Vector2::new(1280.0, 720.0),
            cursor_space: Vector2::zero(),
        }
    }
    /// The screen size to lay out against. Zero axes mean "not authored" — the
    /// layout habit substitutes the capture size for those.
    pub fn viewport_is_set(&self) -> bool {
        self.viewport.x > 0.0 && self.viewport.y > 0.0
    }
}
impl RecordOverride for SysRecordUi {
    fn set_state(&mut self, field: &str, value: &str) {
        match field {
            "width" => self.viewport.x = value.parse().unwrap_or(self.viewport.x),
            "height" => self.viewport.y = value.parse().unwrap_or(self.viewport.y),
            "cursor_width" => self.cursor_space.x = value.parse().unwrap_or(self.cursor_space.x),
            "cursor_height" => self.cursor_space.y = value.parse().unwrap_or(self.cursor_space.y),
            _ => {}
        }
    }
    fn get_state(&self) -> Vec<FieldState> {
        vec![
            FieldState::new("calls", self.draw_calls.len()),
            FieldState::new("width", self.viewport.x),
            FieldState::new("height", self.viewport.y),
            FieldState::new("cursor_width", self.cursor_space.x),
            FieldState::new("cursor_height", self.cursor_space.y),
        ]
    }
}
