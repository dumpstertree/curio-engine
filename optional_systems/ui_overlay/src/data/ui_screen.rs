use camera::camera_snapshot::CameraSnapshot;
use curio_core::{GraphicsMapping, Quaternion, Vector2, Vector3};

use crate::{UiRect, UiWorldPlacement};

/// One screen's ui px, its rect on the swapchain and its world frustum, tied together.
/// The ui pass paints through `to_ndc` and drivers park 3D forms through `place_in_world`,
/// so both use the same numbers. See the vault doc `docs/ui-screen-coordinates`.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct UiScreen {
    /// the ui layout space, origin top-left, y down
    pub viewport: Vector2,
    /// this screen's rect on the swapchain, as 0..1 fractions, y up
    pub mapping_min: Vector2,
    pub mapping_max: Vector2,
    /// the swapchain/capture size in pixels
    pub capture: Vector2,
}

impl UiScreen {
    /// A zero-area mapping (the `GraphicsMapping` default) means the whole swapchain.
    pub fn new(viewport: Vector2, mapping: &GraphicsMapping, capture: Vector2) -> UiScreen {
        let mut mapping_min = mapping.viewport_min;
        let mut mapping_max = mapping.viewport_max;
        if !(mapping_max.x > mapping_min.x && mapping_max.y > mapping_min.y) {
            mapping_min = Vector2::zero();
            mapping_max = Vector2::one();
        }
        UiScreen { viewport, mapping_min, mapping_max, capture }
    }

    /// The screen's real size on the swapchain, rounded as the 3D pass's `Viewport::new` rounds.
    pub fn pixels(&self) -> Vector2 {
        Vector2::new(((self.mapping_max.x - self.mapping_min.x) * self.capture.x).round(), ((self.mapping_max.y - self.mapping_min.y) * self.capture.y).round())
    }

    /// The world frustum's aspect: the mapping rect's pixel size, not the viewport's.
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

    pub fn is_valid(&self) -> bool {
        self.viewport.x > 0.0 && self.viewport.y > 0.0
    }

    /// ui px to NDC; the ui's top edge is the mapping's max.y because GPU y grows up.
    pub fn to_ndc(&self, px: f32, py: f32) -> [f32; 2] {
        let x = self.mapping_min.x + px / self.viewport.x * (self.mapping_max.x - self.mapping_min.x);
        let y = self.mapping_max.y - py / self.viewport.y * (self.mapping_max.y - self.mapping_min.y);
        [2.0 * x - 1.0, 2.0 * y - 1.0]
    }

    /// The placement that makes a unit quad cover `rect`, `depth` metres in front of the lens.
    /// None for a zero viewport, a non-positive depth, or a fov outside 0..180.
    pub fn place_in_world(&self, rect: UiRect, camera: &CameraSnapshot, depth: f32) -> Option<UiWorldPlacement> {
        if !self.is_valid() || depth.is_nan() || depth <= 0.0 || camera.fovy.is_nan() || camera.fovy <= 0.0 || camera.fovy >= 180.0 {
            return None;
        }
        let half = (camera.fovy * 0.5).to_radians();
        let full_h = 2.0 * depth * half.tan();
        let full_w = full_h * self.aspect();
        let cx = (rect.x + rect.width * 0.5) / self.viewport.x - 0.5;
        let cy = 0.5 - (rect.y + rect.height * 0.5) / self.viewport.y;
        let forward = camera.rotation * Vector3::forward();
        let right = camera.rotation * Vector3::right();
        let up = camera.rotation * Vector3::up();
        let position = camera.position + forward * depth + right * (cx * full_w) + up * (cy * full_h);
        // the quad's normal is its local +Z, so the half turn faces it back at the lens, as rendering_image's billboard does
        let rotation = camera.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0));
        let scale = Vector3::new(rect.width / self.viewport.x * full_w, rect.height / self.viewport.y * full_h, 1.0);
        Some(UiWorldPlacement { position, rotation, scale })
    }
}
