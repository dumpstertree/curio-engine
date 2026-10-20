use curio_core::FieldState;
use facet::facet;
use gameplay::traits::field_override::FieldOverride;

#[facet]
pub struct Camera {
    pub fov: f32,
}
impl Camera {
    pub fn default() -> Camera {
        Camera { fov: 60.0, owner: None }
    }
    pub fn builder() -> CameraBuilder {
        CameraBuilder { fov: 60.0 }
    }
}
impl FieldOverride for Camera {
    fn apply(&mut self, key: &str, value: &str) {
        match key {
            // keep the current fov on a bad parse - a fov of 0 produces a degenerate projection
            "fov" => self.fov = value.trim().parse().unwrap_or(self.fov),
            _ => {}
        }
    }
    fn get_state(&self) -> Vec<FieldState> {
        vec![
            FieldState::new("fov", self.fov), //
        ]
    }
}

pub struct CameraBuilder {
    fov: f32,
}
impl CameraBuilder {
    pub fn fov(mut self, fov: f32) -> Self {
        self.fov = fov;
        self
    }

    pub fn build(self) -> Camera {
        Camera { fov: self.fov, owner: None }
    }
}
