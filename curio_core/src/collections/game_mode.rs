use crate::{
    collections::{game_instance::GameInstance, network_modes::NetworkModes},
    graphics::graphics_mapping::GraphicsMapping,
    input::input_mapping::InputMapping,
    Vector2,
};

pub struct GameMode {
    // pub input_mappings: Vec<InputMapping>,
    // pub graphics_mappings: Vec<GraphicsMapping>,
    // pub network_mode: NetworkModes,
    pub game_instances: Vec<GameInstance>,
}
impl GameMode {
    // pub fn new(input_mappings: Vec<InputMapping>, graphics_mappings: Vec<GraphicsMapping>, network_mode: NetworkModes) -> GameMode {
    //     GameMode { input_mappings, graphics_mappings, network_mode }
    // }
    pub fn new(game_instances: Vec<GameInstance>) -> GameMode {
        GameMode { game_instances }
    }

    pub fn new_local_single(input: InputMapping) -> GameMode {
        GameMode {
            game_instances: vec![
                GameInstance::new("peer_player_a", GraphicsMapping::new(Vector2::zero(), Vector2::one()), vec![input], NetworkModes::LocalPeer),
                GameInstance::new("host_logic", GraphicsMapping::new(Vector2::new(0.9, 0.9), Vector2::new(1.0, 1.0)), vec![], NetworkModes::LocalHost),
            ],
        }
    }
    pub fn new_local_splitscreen_2p_vertical(input_p1: InputMapping, input_p2: InputMapping) -> GameMode {
        GameMode {
            game_instances: vec![
                GameInstance::new("peer_player_a", GraphicsMapping::new(Vector2::new(0.5, 0.0), Vector2::new(1.0, 1.0)), vec![input_p1], NetworkModes::LocalPeer),
                GameInstance::new("peer_player_b", GraphicsMapping::new(Vector2::new(0.5, 0.0), Vector2::new(1.0, 1.0)), vec![input_p2], NetworkModes::LocalPeer),
                GameInstance::new("host_logic", GraphicsMapping::new(Vector2::new(0.0, 0.0), Vector2::new(0.5, 1.0)), vec![], NetworkModes::LocalHost),
            ],
        }
    }
    pub fn new_local_splitscreen_2p_horizontal(input_p1: InputMapping, input_p2: InputMapping) -> GameMode {
        GameMode {
            game_instances: vec![
                GameInstance::new("peer_player_a", GraphicsMapping::new(Vector2::new(0.0, 0.0), Vector2::new(1.0, 0.5)), vec![input_p1], NetworkModes::LocalPeer),
                GameInstance::new("peer_player_a", GraphicsMapping::new(Vector2::new(0.0, 0.5), Vector2::new(1.0, 1.0)), vec![input_p2], NetworkModes::LocalPeer),
                GameInstance::new("host_logic", GraphicsMapping::new(Vector2::new(0.9, 0.9), Vector2::new(1.0, 1.0)), vec![], NetworkModes::LocalHost),
            ],
        }
    }
}
