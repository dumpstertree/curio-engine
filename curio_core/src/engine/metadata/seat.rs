use crate::{GraphicsMapping, InputMapping, NetworkModes};

/// A definition defines the input, graphics and network of a user in a Curio
#[derive(Clone)]
pub struct Seat {
    pub network: NetworkModes,
    pub graphics: GraphicsMapping,
    pub input: Vec<InputMapping>,
}
impl Seat {
    pub fn new(graphics: GraphicsMapping, input: Vec<InputMapping>, network: NetworkModes) -> Seat {
        Seat { graphics, input, network }
    }
}
