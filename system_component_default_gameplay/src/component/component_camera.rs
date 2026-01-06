use core::{
    PrefabOverridable,
    collections::{color::Color, light_uniform::LightType, vector3::Vector3},
};

use serde::Deserialize;

use crate::field_override::FieldDeserialize;

#[derive(Default, Deserialize, Clone)]
pub struct Camera {
    pub fov: f32,
}

impl Camera {
    pub fn default() -> Camera {
        Camera { fov: 60.0 }
    }
}

impl FieldDeserialize for Camera {
    fn override_field(&mut self, key: &str, value: &str) {
        match value {
            "fov" => self.fov = value.parse().unwrap_or_default(),
            _ => {}
        }
    }
}
