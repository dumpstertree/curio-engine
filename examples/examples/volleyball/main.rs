pub mod card_parser;
pub mod dependency_filler;
pub mod game_board;
pub mod game_events;
pub mod state {
    pub mod state_ball_mode;
    pub mod state_deck;
    pub mod state_energy;
    pub mod state_position_ball;
    pub mod state_position_player;
    pub mod state_score;
    pub mod state_teams;
    pub mod state_turn;
    pub mod peer {
        pub mod state_peer_selected_card;
    }
}
pub mod ecs {
    pub mod components {
        pub mod component_ball;
        pub mod component_card;
        pub mod component_player;
    }
    pub mod system {
        pub mod peer {
            mod ecs_system_peer_start;
            mod ecs_system_render;
            mod ecs_system_turn_end;
            mod ecs_system_turn_manuever;
            mod ecs_system_turn_move;
            mod ecs_system_view_cards;
        }
        pub mod host {
            mod ecs_system_game_host_play_card;
            mod ecs_system_game_host_point_scored;
            mod ecs_system_game_host_request_manuever;
            mod ecs_system_game_host_request_move;
            mod ecs_system_game_host_reset_board;
            mod ecs_system_game_start;
            mod ecs_system_game_turn_begin;
            mod ecs_system_game_turn_end;
            mod ecs_system_request_turn_end;
        }
    }
}
use crate::game_events::GameEvents;
use core::{
    collections::vector2::Vector2,
    dumpster_engine::{DumpsterEngine, GameMode, NetworkModes, WindowLayout},
    graphics::graphics_mapping::GraphicsMapping,
    input::{input_mapping::InputMapping, key_code::KeyCode},
    system_adapters::adapter_system_gpu::SystemGPU,
};
use pollster::FutureExt;
use std::env;
use system_component_default_gameplay::SystemComponentDefaultGameplay;
use system_component_default_input::SystemComponentDefaultInput;
use system_component_default_networking::SystemComponentDefaultNetworking;
use system_component_default_physics::SystemComponentDefaultPhysics;
use system_component_default_rendering::SystemComponentDefaultGraphics;
use system_component_default_time::SystemComponentDefaultTime;

fn main() {
    // let mut mode = NetworkModes::Offline;

    // let host_type = env::var("HOST_TYPE").unwrap();
    // println!("type: {}", host_type);
    // if host_type == "host" {
    //     // init_host();
    //     println!("is host");

    //     mode = NetworkModes::OnlineHost;
    // } else if host_type == "peer" {
    //     // init_peer();
    //     mode = NetworkModes::OnlinePeer;
    // }

    let input_mapping_0 = InputMapping::new(
        vec![
            (String::from("move_forward"), KeyCode::KeyW),
            (String::from("move_back"), KeyCode::KeyS),
            (String::from("move_left"), KeyCode::KeyA),
            (String::from("move_right"), KeyCode::KeyD),
            (String::from("turn_end"), KeyCode::KeyP),
            (String::from("card_left"), KeyCode::ArrowLeft),
            (String::from("card_right"), KeyCode::ArrowRight),
            (String::from("card_submit"), KeyCode::ArrowUp),
        ],
        vec![],
    );
    let input_mapping_1 = InputMapping::new(
        vec![
            (String::from("move_forward"), KeyCode::KeyW),
            (String::from("move_back"), KeyCode::KeyS),
            (String::from("move_left"), KeyCode::KeyA),
            (String::from("move_right"), KeyCode::KeyD),
            (String::from("turn_end"), KeyCode::KeyP),
            (String::from("card_left"), KeyCode::ArrowLeft),
            (String::from("card_right"), KeyCode::ArrowRight),
            (String::from("card_submit"), KeyCode::ArrowUp),
        ],
        vec![],
    );

    println!("init game start");

    DumpsterEngine::run::<GameEvents>(
        //
        // loop for engine
        SystemGPU::init().block_on(),
        //
        // components
        SystemComponentDefaultTime::new(),
        SystemComponentDefaultInput::new(),
        SystemComponentDefaultGameplay::<GameEvents>::new(),
        SystemComponentDefaultPhysics::new(),
        SystemComponentDefaultGraphics::new(),
        SystemComponentDefaultNetworking::new(),
        //
        // window settings
        WindowLayout::windowed_1080(),
        //

        // create game states
        GameMode::new_local_splitscreen_2p_horizontal(input_mapping_0, input_mapping_1),
        // GameMode::new_local_single(input_mapping_0),
    );
    println!("init game end");
}
