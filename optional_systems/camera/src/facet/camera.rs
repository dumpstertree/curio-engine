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
}
impl FieldOverride for Camera {
    fn apply(&mut self, key: &str, value: &str) {
        match key {
            "fov" => self.fov = value.parse().unwrap_or_default(),
            _ => {}
        }
    }
    fn get_state(&self) -> Vec<FieldState> {
        vec![
            FieldState::new("fov", self.fov), //
        ]
    }
}
