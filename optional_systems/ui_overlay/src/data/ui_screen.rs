use camera::camera_snapshot::CameraSnapshot;
use curio_core::{GraphicsMapping, Quaternion, Vector2, Vector3};

use crate::UiRect;

/// One screen's coordinate spaces, tied together in one place.
///
/// A screen has three sizes that must agree:
///
/// 1. **ui px**: `viewport`, the space the layout habit resolves boxes in and
///    `UiRect`s are written in (y grows down, origin top-left).
/// 2. **the mapping rect**: the screen's share of the swapchain, as the
///    fractions `GraphicsMapping` carries (`mapping_min`..`mapping_max`).
/// 3. **the world frustum**: the 3D pass renders this screen into that same
///    rect, so its aspect is the rect's size in capture pixels (`pixels()`),
///    not the ui viewport's aspect.
///
/// The ui pass stretches ui px uniformly onto the mapping rect, so a fraction
/// of the viewport is the same fraction of the frustum. Both the pass
/// (`to_ndc`) and drivers parking a 3D form over a ui rect (`place_in_world`)
/// go through this type, so they use the same numbers.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct UiScreen {
    /// the ui layout space, in ui pixels
    pub viewport: Vector2,
    /// this screen's rect on the swapchain, as 0..1 fractions (y up, as GPU)
    pub mapping_min: Vector2,
    pub mapping_max: Vector2,
    /// the swapchain/capture size in pixels
    pub capture: Vector2,
}

/// Where a 3D form has to sit to cover a ui rect: a unit quad facing the lens,
/// moved to `position`, turned by `rotation` and scaled by `scale`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct UiWorldPlacement {
    pub position: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
}

impl UiScreen {
    /// A mapping with no area (the `GraphicsMapping` default) means the whole
    /// swapchain, as a single-screen formation does.
    pub fn new(viewport: Vector2, mapping: &GraphicsMapping, capture: Vector2) -> UiScreen {
        let mut mapping_min = mapping.viewport_min;
        let mut mapping_max = mapping.viewport_max;
        if !(mapping_max.x > mapping_min.x && mapping_max.y > mapping_min.y) {
            mapping_min = Vector2::zero();
            mapping_max = Vector2::one();
        }
        UiScreen { viewport, mapping_min, mapping_max, capture }
    }

    /// The screen's size on the swapchain in pixels, rounded exactly as the 3D
    /// pass rounds its viewport (`render_feature_3d.rs`, `Viewport::new`).
    /// This is the real screen size an unauthored `SysRecordUi.viewport` takes.
    pub fn pixels(&self) -> Vector2 {
        Vector2::new(((self.mapping_max.x - self.mapping_min.x) * self.capture.x).round(), ((self.mapping_max.y - self.mapping_min.y) * self.capture.y).round())
    }

    /// The world frustum's aspect for this screen: the mapping rect's pixel
    /// size, which is what the 3D pass hands the camera projection.
    pub fn aspect(&self) -> f32 {
        let pixels = self.pixels();
        if pixels.x > 0.0 && pixels.y > 0.0 {
            pixels.x / pixels.y
        } else if self.viewport.x > 0.0 && self.viewport.y > 0.0 {
            self.viewport.x / self.viewport.y
        } else {
            1.0
        }
    }

    /// Can ui px be mapped at all? False for a zero viewport.
    pub fn is_valid(&self) -> bool {
        self.viewport.x > 0.0 && self.viewport.y > 0.0
    }

    /// ui px → normalized device coordinates on the swapchain. The ui's top
    /// edge is the mapping's max.y because ui y grows down and GPU y grows up.
    pub fn to_ndc(&self, px: f32, py: f32) -> [f32; 2] {
        let x = self.mapping_min.x + px / self.viewport.x * (self.mapping_max.x - self.mapping_min.x);
        let y = self.mapping_max.y - py / self.viewport.y * (self.mapping_max.y - self.mapping_min.y);
        [2.0 * x - 1.0, 2.0 * y - 1.0]
    }

    /// The placement that makes a unit quad (a `RendererImage` on a
    /// `Transform3D`) cover `rect` exactly, `depth` metres in front of the
    /// lens. The frustum is the 3D pass's own: vertical fov from the camera,
    /// aspect from `aspect()`. None when the screen, depth or fov is unusable.
    pub fn place_in_world(&self, rect: UiRect, camera: &CameraSnapshot, depth: f32) -> Option<UiWorldPlacement> {
        if !self.is_valid() || depth.is_nan() || depth <= 0.0 || camera.fovy.is_nan() || camera.fovy <= 0.0 || camera.fovy >= 180.0 {
            return None;
        }
        let half = (camera.fovy * 0.5).to_radians();
        let full_h = 2.0 * depth * half.tan();
        let full_w = full_h * self.aspect();
        // the rect's centre as a fraction of the screen, offset from its middle
        let cx = (rect.x + rect.width * 0.5) / self.viewport.x - 0.5;
        let cy = 0.5 - (rect.y + rect.height * 0.5) / self.viewport.y;
        let forward = camera.rotation * Vector3::forward();
        let right = camera.rotation * Vector3::right();
        let up = camera.rotation * Vector3::up();
        let position = camera.position + forward * depth + right * (cx * full_w) + up * (cy * full_h);
        // the quad's plane normal is its local +Z, so the half turn faces it
        // back at the lens — the convention rendering_image's billboard uses
        let rotation = camera.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
        let scale = Vector3::new(rect.width / self.viewport.x * full_w, rect.height / self.viewport.y * full_h, 1.0);
        Some(UiWorldPlacement { position, rotation, scale })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.0001
    }
    fn full() -> GraphicsMapping {
        GraphicsMapping::new(Vector2::zero(), Vector2::one())
    }

    #[test]
    fn ui_screen_pixels_follow_the_mapping_rect() {
        let screen = UiScreen::new(Vector2::new(1280.0, 720.0), &GraphicsMapping::new(Vector2::new(0.5, 0.0), Vector2::new(1.0, 1.0)), Vector2::new(1921.0, 1080.0));
        // same rounding as the 3D pass's Viewport::new
        assert_eq!(screen.pixels(), Vector2::new(961.0, 1080.0));
        assert!(close(screen.aspect(), 961.0 / 1080.0));
        // an empty mapping is the whole swapchain
        let whole = UiScreen::new(Vector2::new(10.0, 10.0), &GraphicsMapping::default(), Vector2::new(800.0, 600.0));
        assert_eq!(whole.pixels(), Vector2::new(800.0, 600.0));
    }

    #[test]
    fn ui_screen_maps_corners_to_ndc() {
        let screen = UiScreen::new(Vector2::new(1280.0, 720.0), &full(), Vector2::new(1920.0, 1080.0));
        let top_left = screen.to_ndc(0.0, 0.0);
        let bottom_right = screen.to_ndc(1280.0, 720.0);
        assert!(close(top_left[0], -1.0) && close(top_left[1], 1.0));
        assert!(close(bottom_right[0], 1.0) && close(bottom_right[1], -1.0));
        // right half of a split screen
        let half = UiScreen::new(Vector2::new(640.0, 720.0), &GraphicsMapping::new(Vector2::new(0.5, 0.0), Vector2::one()), Vector2::new(1280.0, 720.0));
        assert!(close(half.to_ndc(0.0, 0.0)[0], 0.0));
        assert!(close(half.to_ndc(640.0, 0.0)[0], 1.0));
    }

    #[test]
    fn ui_screen_parks_the_whole_viewport_on_the_whole_frustum() {
        let screen = UiScreen::new(Vector2::new(1280.0, 720.0), &full(), Vector2::new(1280.0, 720.0));
        let camera = CameraSnapshot::new(Vector3::new(1.0, 2.0, 3.0));
        let Some(placed) = screen.place_in_world(UiRect::new(0.0, 0.0, 1280.0, 720.0), &camera, 2.0) else {
            panic!("a valid screen must place");
        };
        let full_h = 2.0 * 2.0 * (30.0f32).to_radians().tan();
        assert!(close(placed.scale.y, full_h));
        assert!(close(placed.scale.x, full_h * 1280.0 / 720.0));
        // centred: straight down the lens
        assert!(close(placed.position.x, 1.0) && close(placed.position.y, 2.0) && close(placed.position.z, 5.0));
    }

    #[test]
    fn ui_screen_frustum_aspect_comes_from_the_capture_not_the_viewport() {
        // an authored 1280x720 design space on a 2.4:1 window: the ui is
        // stretched onto the window, and so must the parked quad be
        let screen = UiScreen::new(Vector2::new(1280.0, 720.0), &full(), Vector2::new(1920.0, 800.0));
        let camera = CameraSnapshot::new(Vector3::zero());
        let Some(placed) = screen.place_in_world(UiRect::new(0.0, 0.0, 640.0, 720.0), &camera, 1.0) else {
            panic!("a valid screen must place");
        };
        let full_h = 2.0 * (30.0f32).to_radians().tan();
        let full_w = full_h * 2.4;
        assert!(close(placed.scale.x, full_w * 0.5));
        assert!(close(placed.scale.y, full_h));
        // left half: a quarter of the frustum width towards screen-left, and
        // curio's screen-right is -X, so screen-left is +X
        assert!(close(placed.position.x, full_w * 0.25));
        assert!(close(placed.position.y, 0.0));
    }

    #[test]
    fn ui_screen_refuses_unusable_input() {
        let camera = CameraSnapshot::new(Vector3::zero());
        let empty = UiScreen::new(Vector2::zero(), &full(), Vector2::new(1280.0, 720.0));
        assert!(
            empty
                .place_in_world(UiRect::new(0.0, 0.0, 1.0, 1.0), &camera, 1.0)
                .is_none()
        );
        let screen = UiScreen::new(Vector2::new(1280.0, 720.0), &full(), Vector2::new(1280.0, 720.0));
        assert!(
            screen
                .place_in_world(UiRect::new(0.0, 0.0, 1.0, 1.0), &camera, 0.0)
                .is_none()
        );
    }
}
