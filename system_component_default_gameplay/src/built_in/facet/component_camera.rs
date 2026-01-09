use serde::Deserialize;

use crate::traits::field_override::FieldOverride;

#[derive(Default, Deserialize, Clone)]
pub struct Camera {
    pub fov: f32,
}

impl Camera {
    pub fn default() -> Camera {
        Camera { fov: 60.0 }
    }
}

impl FieldOverride for Camera {
    fn apply(&mut self, key: &str, value: &str) {
        match key {
            "fov" => self.fov = value.parse().unwrap_or_default(),
            _ => {}
        }
    }
}
