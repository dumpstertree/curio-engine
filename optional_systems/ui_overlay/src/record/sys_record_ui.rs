use crate::{UiDrawCall, UiScreen};
use curio_core::{FieldState, GraphicsMapping, RecordOverride, RecordScope, Services, Vector2};
use record::record;

/// A screen's ui: what it paints and the spaces it paints in. See `UiScreen`
/// for how `viewport`, `mapping` and the world frustum relate.
#[record(name = "Ui", ownership = RecordScope::Instance)]
pub struct SysRecordUi {
    /// every box this screen paints this frame, in the order it is drawn
    pub draw_calls: Vec<UiDrawCall>,
    /// the size of this screen in ui pixels — the one space the layout habit,
    /// the pass and anything parked over a ui rect agree on. Left at zero
    /// (unauthored), the layout habit fills it with the screen's real size on
    /// the swapchain every tick, so it follows resizes. A value a project
    /// writes here is kept and stretched onto the screen instead.
    pub viewport: Vector2,
    /// the pixel space the `AxisCode::Cursor` arrives in — usually the window
    /// size. Zero falls back to the capture size. Boxes are laid out in
    /// `viewport` pixels, so the hit-test scales the cursor by the two.
    pub cursor_space: Vector2,
    /// this screen's rect on the swapchain, as the graphics system's formation
    /// gives it. Written by the engine; zero area means the whole swapchain.
    pub mapping: GraphicsMapping,
    // the viewport the engine filled in last, so a later fill can tell its own
    // value (refresh it) from one a project wrote (keep it)
    filled_viewport: Vector2,
}

impl SysRecordUi {
    /// Has a project authored the viewport? False when it is zero or still
    /// holds the real screen size the engine filled in.
    pub fn viewport_is_set(&self) -> bool {
        self.viewport.x > 0.0 && self.viewport.y > 0.0 && self.viewport != self.filled_viewport
    }
    /// Would `fill_viewport` change the record? Lets a caller skip the write.
    pub fn viewport_needs_fill(&self, screen_size: Vector2) -> bool {
        !self.viewport_is_set() && screen_size.x > 0.0 && screen_size.y > 0.0 && self.viewport != screen_size
    }
    /// Fill an unauthored viewport with the real screen size. Returns whether
    /// the record changed.
    pub fn fill_viewport(&mut self, screen_size: Vector2) -> bool {
        if !self.viewport_needs_fill(screen_size) {
            return false;
        }
        self.viewport = screen_size;
        self.filled_viewport = screen_size;
        true
    }
    /// This screen's spaces against a given capture size.
    pub fn screen_on(&self, capture: Vector2) -> UiScreen {
        UiScreen::new(self.viewport, &self.mapping, capture)
    }
    /// This screen's spaces against the live capture size — what a driver uses
    /// to park a 3D form over a ui rect (`UiScreen::place_in_world`).
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

#[test]
fn ui_record_fills_an_unauthored_viewport_and_follows_resizes() {
    let mut record = SysRecordUi::default();
    assert!(!record.viewport_is_set());
    assert!(record.fill_viewport(Vector2::new(1920.0, 1080.0)));
    assert_eq!(record.viewport, Vector2::new(1920.0, 1080.0));
    // the engine's own value is still "unauthored", so a resize refreshes it
    assert!(!record.viewport_is_set());
    assert!(record.fill_viewport(Vector2::new(800.0, 600.0)));
    assert_eq!(record.viewport, Vector2::new(800.0, 600.0));
    assert!(!record.fill_viewport(Vector2::new(800.0, 600.0)));
}

#[test]
fn ui_record_keeps_an_authored_viewport() {
    let mut record = SysRecordUi::default();
    record.fill_viewport(Vector2::new(1920.0, 1080.0));
    // a project pins a design space after the engine filled one in
    record.viewport = Vector2::new(1280.0, 720.0);
    assert!(record.viewport_is_set());
    assert!(!record.fill_viewport(Vector2::new(800.0, 600.0)));
    assert_eq!(record.viewport, Vector2::new(1280.0, 720.0));
    // the same via the editor override
    let mut overridden = SysRecordUi::default();
    overridden.set_state("width", "640");
    overridden.set_state("height", "360");
    assert!(!overridden.fill_viewport(Vector2::new(1920.0, 1080.0)));
    assert_eq!(overridden.viewport, Vector2::new(640.0, 360.0));
}
