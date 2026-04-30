use crate::{graphics::graphics_mapping::GraphicsMapping, input::input_mapping::InputMapping, network_modes::NetworkModes};

pub struct GameInstance {
    pub name: String,
    pub graphics_mappings: GraphicsMapping,
    pub input_mappings: Vec<InputMapping>,
    pub network_mode: NetworkModes,
}
impl GameInstance {
    pub fn new(name: &str, graphics_mappings: GraphicsMapping, input_mappings: Vec<InputMapping>, network_mode: NetworkModes) -> GameInstance {
        GameInstance {
            name: String::from(name),
            graphics_mappings,
            input_mappings,
            network_mode,
        }
    }
}
