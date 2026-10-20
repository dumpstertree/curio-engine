use curio_core::{Color, FieldState, Vector3};
use facet::facet;
use gameplay::traits::field_override::FieldOverride;

use crate::LightType;

#[facet]
pub struct Light {
    pub light_type: LightType,
    pub direction: Vector3,
    pub color: Color,
    pub radius: f32,
    pub intensity: f32,
}
impl Light {
    pub fn builder() -> LightBuilder {
        LightBuilder {
            light_type: LightType::Directional,
            // default straight down so a directional light is never a zero-length vector
            direction: Vector3::down(),
            color: Color::white(),
            radius: 5.0,
            intensity: 1.0,
        }
    }
}
impl FieldOverride for Light {
    fn apply(&mut self, field: &str, value: &str) {
        let value = value.trim();
        match field {
            "type" => {
                if let Some(light_type) = parse_light_type(value) {
                    self.light_type = light_type;
                }
            }
            "direction" => self.direction = value.parse().unwrap_or(self.direction),
            "color" => self.color = value.parse().unwrap_or(self.color),
            // keep the current radius on a bad parse - a radius of 0 breaks point light falloff
            "radius" => self.radius = value.parse().unwrap_or(self.radius),
            "intensity" => self.intensity = value.parse().unwrap_or(self.intensity),
            _ => {}
        }
    }
    fn get_state(&self) -> Vec<FieldState> {
        vec![
            FieldState::new("type", serialize_light_type(&self.light_type)),
            FieldState::new("direction", self.direction),
            FieldState::new("color", self.color),
            FieldState::new("radius", self.radius),
            FieldState::new("intensity", self.intensity),
        ]
    }
}

fn parse_light_type(value: &str) -> Option<LightType> {
    match value.to_ascii_lowercase().as_str() {
        "point" => Some(LightType::Point),
        "directional" => Some(LightType::Directional),
        _ => None,
    }
}
fn serialize_light_type(light_type: &LightType) -> String {
    match light_type {
        LightType::Point => "point".to_string(),
        LightType::Directional => "directional".to_string(),
    }
}

pub struct LightBuilder {
    light_type: LightType,
    direction: Vector3,
    color: Color,
    radius: f32,
    intensity: f32,
}
impl LightBuilder {
    pub fn light_type(mut self, light_type: LightType) -> Self {
        self.light_type = light_type;
        self
    }
    pub fn direction(mut self, direction: Vector3) -> Self {
        self.direction = direction;
        self
    }
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
    pub fn intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    pub fn build(self) -> Light {
        Light {
            light_type: self.light_type,
            direction: self.direction,
            color: self.color,
            radius: self.radius,
            intensity: self.intensity,
            owner: None,
        }
    }
}
