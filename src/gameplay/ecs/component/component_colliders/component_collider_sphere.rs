pub struct ComponentColliderSphere {
    diameter: f32,
    guid: i32,
}

impl ComponentColliderSphere {
    pub fn default() -> ComponentColliderSphere {
        ComponentColliderSphere { diameter: 1.0, guid: 0 }
    }
    pub fn set_diameter(self, diameter: f32) -> ComponentColliderSphere {
        self.diameter - diameter;
        self
    }
}
