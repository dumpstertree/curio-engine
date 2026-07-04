use crate::{GraphicsMapping, InputMapping, NetworkModes, Seat, Vector2};

/// A formation of Seats used to define the visuals and input for a Curio
#[derive(Clone)]
pub struct Formation {
    pub seats: Vec<Seat>,
}
impl Formation {
    /// Create a custom instance
    pub fn custom(game_instances: Vec<Seat>) -> Formation {
        Formation { seats: game_instances }
    }

    /// Create an instance for single player gameplay
    pub fn template_local_single(input: InputMapping) -> Formation {
        Formation {
            seats: vec![
                Seat::new(GraphicsMapping::new(Vector2::zero(), Vector2::one()), vec![input], NetworkModes::LocalPeer),
                Seat::new(GraphicsMapping::new(Vector2::new(0.9, 0.9), Vector2::new(1.0, 1.0)), vec![], NetworkModes::LocalHost),
            ],
        }
    }

    /// Create an instance for two player gameplay with a screen on top and a screen on bottom
    pub fn template_local_splitscreen_2p_vertical(input_p1: InputMapping, input_p2: InputMapping) -> Formation {
        Formation {
            seats: vec![
                Seat::new(GraphicsMapping::new(Vector2::new(0.5, 0.0), Vector2::new(1.0, 1.0)), vec![input_p1], NetworkModes::LocalPeer),
                Seat::new(GraphicsMapping::new(Vector2::new(0.5, 0.0), Vector2::new(1.0, 1.0)), vec![input_p2], NetworkModes::LocalPeer),
                Seat::new(GraphicsMapping::new(Vector2::new(0.0, 0.0), Vector2::new(0.5, 1.0)), vec![], NetworkModes::LocalHost),
            ],
        }
    }

    /// Create an instance for two player gameplay with a screen on left and a screen on right
    pub fn template_local_splitscreen_2p_horizontal(input_p1: InputMapping, input_p2: InputMapping) -> Formation {
        Formation {
            seats: vec![
                Seat::new(GraphicsMapping::new(Vector2::new(0.0, 0.0), Vector2::new(1.0, 0.5)), vec![input_p1], NetworkModes::LocalPeer),
                Seat::new(GraphicsMapping::new(Vector2::new(0.0, 0.5), Vector2::new(1.0, 1.0)), vec![input_p2], NetworkModes::LocalPeer),
                Seat::new(GraphicsMapping::new(Vector2::new(0.9, 0.9), Vector2::new(1.0, 1.0)), vec![], NetworkModes::LocalHost),
            ],
        }
    }
}
