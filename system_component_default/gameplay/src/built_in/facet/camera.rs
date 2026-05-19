use curio_core::FieldState;

use crate::{
    form::Form,
    traits::{facet_common::FacetCommon, field_override::FieldOverride},
};

#[derive(Default, Clone)]
pub struct Camera {
    pub fov: f32,
    owner: Option<Form>,
}
impl FacetCommon for Camera {
    fn set_ownership(&mut self, owner: Form) {
        self.owner = Some(owner);
    }
    fn form(&self) -> Form {
        self.owner.clone().unwrap()
    }
}
unsafe impl Send for Camera {}
unsafe impl Sync for Camera {}
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
