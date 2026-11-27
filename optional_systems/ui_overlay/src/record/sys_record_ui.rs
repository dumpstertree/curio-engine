use crate::{UiDrawCall, UiScreen};
use curio_core::{FieldState, GraphicsMapping, RecordOverride, RecordScope, Services, Vector2};
use record::record;

#[record(name = "Ui", ownership = RecordScope::Instance)]
pub struct SysRecordUi {
    /// every box this screen paints this frame, in the order it is drawn
    pub draw_calls: Vec<UiDrawCall>,
    /// this screen's size in ui pixels, the one space layout, pass and 3D parking share.
    /// Left at zero, the layout habit fills in the real screen size each tick; an authored value is kept.
    pub viewport: Vector2,
    /// the pixel space the `AxisCode::Cursor` arrives in — usually the window
    /// size. Zero falls back to the capture size. Boxes are laid out in
    /// `viewport` pixels, so the hit-test scales the cursor by the two.
    pub cursor_space: Vector2,
    /// this screen's rect on the swapchain, written by the engine; zero area means the whole swapchain
    pub mapping: GraphicsMapping,
    // the engine's last fill, so a later fill can tell its own value from an authored one
    filled_viewport: Vector2,
}

impl SysRecordUi {
    /// False when the viewport is zero or still holds the size the engine filled in.
    pub fn viewport_is_set(&self) -> bool {
        self.viewport.x > 0.0 && self.viewport.y > 0.0 && self.viewport != self.filled_viewport
    }
    pub fn viewport_needs_fill(&self, screen_size: Vector2) -> bool {
        !self.viewport_is_set() && screen_size.x > 0.0 && screen_size.y > 0.0 && self.viewport != screen_size
    }
    /// Fill an unauthored viewport with the real screen size; returns whether it changed.
    pub fn fill_viewport(&mut self, screen_size: Vector2) -> bool {
        if !self.viewport_needs_fill(screen_size) {
            return false;
        }
        self.viewport = screen_size;
        self.filled_viewport = screen_size;
        true
    }
    pub fn screen_on(&self, capture: Vector2) -> UiScreen {
        UiScreen::new(self.viewport, &self.mapping, capture)
    }
    /// This screen's spaces against the live capture size.
    pub fn screen(&self) -> UiScreen {
        let gpu = &Services::get().gpu;
        self.screen_on(Vector2::new(gpu.capture_width as f32, gpu.capture_height as f32))
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
