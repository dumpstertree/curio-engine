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
impl Light {
    pub fn builder() -> LightBuilder {
        LightBuilder {
            asset: LightType::Directional,
            direction: Vector3::zero(),
            color: Color::white(),
            radius: 5.0,
            intensity: 1.0,
        }
    }
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

pub struct LightBuilder {
    asset: LightType,
    direction: Vector3,
    color: Color,
    radius: f32,
    intensity: f32,
}
impl LightBuilder {
    pub fn light_type(mut self, light_type: LightType) -> Self {
        self.asset = light_type;
        self
    }
    pub fn direction(mut self, direction: Vector3) -> Self {
        self.direction = direction;
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
            asset: self.asset,
            direction: self.direction,
            color: self.color,
            radius: self.radius,
            intensity: self.intensity,
            owner: None,
        }
    }
}
