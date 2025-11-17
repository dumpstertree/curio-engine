pub(crate) mod constants;
pub(crate) mod game_events;
pub mod ecs {
    pub mod system {
        pub(crate) mod system_ball_move;
        pub(crate) mod system_paddle_move;
        pub(crate) mod system_pong_init;
        pub(crate) mod system_spin;
    }
    pub mod component {
        pub(crate) mod component_ball;
        pub(crate) mod component_paddle;
        pub(crate) mod component_spin;
    }
}
pub mod state {
    pub mod state_score;
}

use core::{
    collections::vector2::Vector2,
    dumpster_engine::NetworkModes,
    graphics::graphics_mapping::GraphicsMapping,
    input::{input_mapping::InputMapping, key_code::ButtonCode},
};
use std::env;

fn main() {
    let mut mode = NetworkModes::LocalHost;

    let host_type = env::var("HOST_TYPE").unwrap();
    println!("type: {}", host_type);
    if host_type == "host" {
        // init_host();
        println!("is host");

        mode = NetworkModes::OnlineHost;
    } else if host_type == "peer" {
        // init_peer();
        mode = NetworkModes::OnlinePeer;
    }

    let input_mapping_0 = InputMapping::new(
        vec![
            (String::from("move_forward"), ButtonCode::KeyW),
            (String::from("move_back"), ButtonCode::KeyS),
            (String::from("move_left"), ButtonCode::KeyA),
            (String::from("move_right"), ButtonCode::KeyD),
        ],
        vec![],
    );
    let input_mapping_1 = InputMapping::new(
        vec![
            (String::from("move_forward"), ButtonCode::KeyI),
            (String::from("move_back"), ButtonCode::KeyK),
            (String::from("move_left"), ButtonCode::KeyJ),
            (String::from("move_right"), ButtonCode::KeyL),
        ],
        vec![],
    );
    let graphics_mapping_0 = GraphicsMapping::new(Vector2::new(0.0, 0.0), Vector2::new(0.5, 1.0));
    let graphics_mapping_1 = GraphicsMapping::new(Vector2::new(0.5, 0.0), Vector2::new(1.0, 1.0));
    println!("init game start");

    // DumpsterEngine::run::<GameEvents>(
    //     //
    //     // loop for engine
    //     SystemGPU::init().block_on(),
    //     //
    //     // components
    //     SystemComponentDefaultTime::new(),
    //     SystemComponentDefaultInput::new(),
    //     SystemComponentDefaultGameplay::<GameEvents>::new(),
    //     SystemComponentDefaultPhysics::new(),
    //     SystemComponentDefaultGraphics::new(),
    //     SystemComponentDefaultNetworking::new(),
    //     //
    //     // window settings
    //     WindowLayout::windowed_1080(),
    //     //

    //     // create game states
    //     GameMode::new(vec![input_mapping_0, input_mapping_1], vec![graphics_mapping_0, graphics_mapping_1], mode),
    // );
    // println!("init game end");
}
