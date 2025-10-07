// #[derive(Debug, Clone, Serialize, RegisterComponent)]
pub struct ComponentEnergyToken {
    pub index: i32,
}
impl ComponentEnergyToken {
    pub fn default() -> ComponentEnergyToken {
        ComponentEnergyToken { index: 0 }
    }
    pub fn set_index(mut self, index: i32) -> Self {
        self.index = index;
        self
    }
}
