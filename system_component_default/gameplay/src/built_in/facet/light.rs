use crate::{
    form::Form,
    traits::{facet_common::FacetCommon, field_override::FieldOverride},
};
use curio_core::{Color, FieldState, LightType, Vector3};

#[derive(Default, Clone)]
pub struct Light {
    pub asset: LightType,
    pub direction: Vector3,
    pub color: Color,
    pub radius: f32,
    pub intensity: f32,
    owner: Option<Form>,
}
impl Light {
    pub fn default() -> Light {
        Light {
            asset: LightType::Point,
            direction: Vector3::zero(),
            color: Color::white(),
            radius: 10.0,
            intensity: 1.0,
            owner: None,
        }
    }
}
unsafe impl Send for Light {}
unsafe impl Sync for Light {}
impl FacetCommon for Light {
    fn set_ownership(&mut self, owner: Form) {
        self.owner = Some(owner);
    }
    fn form(&self) -> Form {
        self.owner.clone().unwrap()
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
