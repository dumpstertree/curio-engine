use curio_core::{Color, FieldState, Vector3};
use facet::facet;
use gameplay::traits::field_override::FieldOverride;

use crate::LightType;

#[facet]
pub struct Light {
    pub asset: LightType,
    pub direction: Vector3,
    pub color: Color,
    pub radius: f32,
    pub intensity: f32,
}
impl FieldOverride for Light {
    fn apply(&mut self, field: &str, value: &str) {
        match field {
            // "type" => self.asset = value.parse().unwrap_or_default(),
            "direction" => self.direction = value.parse().unwrap_or_default(),
            "color" => self.color = value.parse().unwrap_or_default(),
            "radius" => self.radius = value.parse().unwrap_or_default(),
            "intensity" => self.intensity = value.parse().unwrap_or_default(),
            _ => {}
        }
    }
    fn get_state(&self) -> Vec<FieldState> {
        vec![
            FieldState::new("direction", self.direction), //
            FieldState::new("color", self.color),
            FieldState::new("radius", self.radius),
            FieldState::new("intensity", self.intensity),
        ]
    }
}
