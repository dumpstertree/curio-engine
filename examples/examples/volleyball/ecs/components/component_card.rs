// #[derive(Debug, Clone, Serialize, RegisterComponent)]
pub struct ComponentCard {
    pub index: i32,
}
impl ComponentCard {
    pub fn default() -> ComponentCard {
        ComponentCard { index: 0 }
    }
    pub fn set_index(mut self, index: i32) -> Self {
        self.index = index;
        self
    }
}
