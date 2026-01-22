use crate::{
    form::Form,
    traits::{facet_common::FacetCommon, field_override::FieldOverride},
};
use curio_core::collections::vector3::Vector3;

pub struct AnimatorPositionSin {
    owner: Option<Form>,
    enabled: bool,
    min: Vector3,
    max: Vector3,
    speed: f32,
}
impl AnimatorPositionSin {
    pub fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn min(&self) -> Vector3 {
        self.min
    }
    pub fn max(&self) -> Vector3 {
        self.max
    }
    pub fn speed(&self) -> f32 {
        self.speed
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    pub fn set_min(&mut self, min: Vector3) {
        self.min = min;
    }
    pub fn set_max(&mut self, max: Vector3) {
        self.max = max;
    }
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }
}
impl FieldOverride for AnimatorPositionSin {
    fn apply(&mut self, key: &str, value: &str) {
        match key {
            "min" => self.min = value.parse().unwrap_or_default(),
            "max" => self.max = value.parse().unwrap_or_default(),
            "speed" => self.speed = value.parse().unwrap_or_default(),
            "enabled" => self.enabled = value.parse().unwrap_or_default(),
            _ => {}
        }
    }
}
impl Default for AnimatorPositionSin {
    fn default() -> Self {
        Self {
            owner: None,
            enabled: false,
            min: Vector3::new(0.5, 0.5, 0.5),
            max: Vector3::new(1.5, 1.5, 1.5),
            speed: 1.0,
        }
    }
}
impl FacetCommon for AnimatorPositionSin {
    fn set_ownership(&mut self, owner: Form) {
        self.owner = Some(owner);
    }

    fn form(&self) -> Form {
        self.owner.clone().unwrap()
    }
}
unsafe impl Send for AnimatorPositionSin {}
unsafe impl Sync for AnimatorPositionSin {}
