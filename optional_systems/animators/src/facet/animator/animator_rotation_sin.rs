use curio_core::{FieldState, Vector3};
use facet::facet;
use gameplay::traits::field_override::FieldOverride;

#[facet]
pub struct AnimatorRotationSin {
    pub enabled: bool,
    pub min: Vector3,
    pub max: Vector3,
    pub speed: f32,
}
impl AnimatorRotationSin {
    pub fn default() -> AnimatorRotationSin {
        AnimatorRotationSin {
            owner: None,
            enabled: false,
            min: Vector3::new(0.5, 0.5, 0.5),
            max: Vector3::new(1.5, 1.5, 1.5),
            speed: 1.0,
        }
    }
    pub fn builder() -> AnimatorRotationSinBuilder {
        AnimatorRotationSinBuilder {
            enabled: false,
            min: Vector3::new(0.5, 0.5, 0.5),
            max: Vector3::new(1.5, 1.5, 1.5),
            speed: 1.0,
        }
    }
}
impl FieldOverride for AnimatorRotationSin {
    fn apply(&mut self, key: &str, value: &str) {
        match key {
            // keep the current values on a bad parse
            "min" => self.min = value.trim().parse().unwrap_or(self.min),
            "max" => self.max = value.trim().parse().unwrap_or(self.max),
            "speed" => self.speed = value.trim().parse().unwrap_or(self.speed),
            "enabled" => self.enabled = value.parse().unwrap_or_default(),
            _ => {}
        }
    }
    fn get_state(&self) -> Vec<FieldState> {
        vec![
            FieldState::new("min", self.min), //
            FieldState::new("max", self.max),
            FieldState::new("speed", self.speed),
            FieldState::new("enabled", self.enabled),
        ]
    }
}

pub struct AnimatorRotationSinBuilder {
    enabled: bool,
    min: Vector3,
    max: Vector3,
    speed: f32,
}
impl AnimatorRotationSinBuilder {
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    pub fn min(mut self, min: Vector3) -> Self {
        self.min = min;
        self
    }
    pub fn max(mut self, max: Vector3) -> Self {
        self.max = max;
        self
    }
    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }
    pub fn build(self) -> AnimatorRotationSin {
        AnimatorRotationSin {
            owner: None,
            enabled: self.enabled,
            min: self.min,
            max: self.max,
            speed: self.speed,
        }
    }
}
