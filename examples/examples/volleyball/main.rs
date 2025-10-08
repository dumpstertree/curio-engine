pub mod card_parser;
pub mod dependency_filler;
pub mod game_board;
pub mod game_events;
pub mod cards {
    pub mod attribute_target_type_cards;
    pub mod attribute_target_type_entities;
    pub mod attribute_target_type_players;
    pub mod attribute_target_type_tiles;
    pub mod card_attribute_events;
    pub mod card_attribute_modifier;
    pub mod card_instance;
    pub mod card_library;
    pub mod card_master;
    pub mod card_modifier;
    pub mod data_dep_empty;
    pub mod data_dep_filled;
}
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
        pub mod state_peer_input_mode;
        pub mod state_peer_selected_card;
    }
    pub mod host {
        pub mod state_card_attribute_modifier_stack;
    }
}
pub mod event_recievers {
    mod event_reciever_apply_card_attribute_event_cards_discard;
    mod event_reciever_apply_card_attribute_event_cards_draw;
    mod event_reciever_apply_card_attribute_event_cards_energy_edit;
    mod event_reciever_apply_card_attribute_event_cards_energy_refill;
    mod event_reciever_apply_card_attribute_event_move_ball_forward;
    mod event_reciever_apply_card_attribute_event_move_ball_horizontal;
    mod event_reciever_apply_card_attribute_event_move_entities;
    mod event_reciever_apply_card_attribute_modifier_cost_for_entities;
    mod event_reciever_apply_card_attribute_modifier_energy_for_entities;
    mod event_reciever_apply_card_attribute_modifier_range_for_entities;
    mod event_reciever_clear_card_attribute_modifiers_all;
    mod event_reciever_clear_card_attribute_modifiers_for_flag;
}
pub mod ecs {
    pub mod components {
        pub mod component_ball;
        pub mod component_card;
        pub mod component_energy_token;
        pub mod component_player;
        pub mod component_view_player;
    }
    pub mod system {
        pub mod peer {
            mod ecs_system_peer_start;
            mod ecs_system_peer_update_input_mode;
            mod ecs_system_render;
            mod ecs_system_turn_end;
            mod ecs_system_turn_manuever;
            mod ecs_system_turn_move;
            mod ecs_system_view_cards;
            mod ecs_system_view_energy;
            mod ecs_system_view_move_ball;
            mod ecs_system_view_move_player;
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
    dumpster_engine::{DumpsterEngine, GameMode, WindowLayout},
    input::{input_mapping::InputMapping, key_code::KeyCode},
    io::{
        asset_database::{AssetDatabase, AssetDatabaseListing},
        asset_loader::AssetLoader,
    },
    system_adapters::adapter_system_gpu::SystemGPU,
};
use pollster::FutureExt;
use system_component_default_gameplay::SystemComponentDefaultGameplay;
use system_component_default_input::SystemComponentDefaultInput;
use system_component_default_networking::SystemComponentDefaultNetworking;
use system_component_default_physics::SystemComponentDefaultPhysics;
use system_component_default_rendering::SystemComponentDefaultGraphics;
use system_component_default_time::SystemComponentDefaultTime;

pub enum AssetMappingUIDs {
    // animated
    Ball,
    Goblin,
    EnergyToken,
    // static
    Court,
    Card,
}
impl AssetMappingUIDs {
    pub fn uid(&self) -> String {
        match self {
            // mesh - animated
            AssetMappingUIDs::Ball => String::from("mesh_animated_ball"),
            AssetMappingUIDs::Goblin => String::from("mesh_animated_goblin"),
            AssetMappingUIDs::EnergyToken => String::from("mesh_animated_energy"),
            // mesh - static
            AssetMappingUIDs::Court => String::from("mesh_static_court"),
            AssetMappingUIDs::Card => String::from("mesh_static_card"),
        }
    }
}
fn main() {
    AssetLoader::set_database(AssetDatabase::new_from_explicit(vec![
        // remote
        (AssetMappingUIDs::Goblin.uid(), AssetDatabaseListing::RemoteToCache(String::from("downloaded_spine.asset"), String::from("https://drive.dumpstertree.com/api/public/dl/439SQ2FW"))),
        (AssetMappingUIDs::Ball.uid(), AssetDatabaseListing::RemoteToCache(String::from("downloaded_spine2.asset"), String::from("https://drive.dumpstertree.com/api/public/dl/XTh-OVkc"))),
        // local
        (AssetMappingUIDs::EnergyToken.uid(), AssetDatabaseListing::RemoteToCache(String::from("energy.asset"), String::from("https://drive.dumpstertree.com/api/public/dl/69VQTiko"))),
        (AssetMappingUIDs::Court.uid(), AssetDatabaseListing::Local(String::from("mesh/court.glb"))),
        (AssetMappingUIDs::Card.uid(), AssetDatabaseListing::Local(String::from("mesh/card_bump.glb"))),
    ]));

    let input_mapping_0 = InputMapping::new(
        vec![
            (String::from("card_mode"), KeyCode::ShiftLeft),
            (String::from("move_forward"), KeyCode::KeyW),
            (String::from("move_back"), KeyCode::KeyS),
            (String::from("move_left"), KeyCode::KeyA),
            (String::from("move_right"), KeyCode::KeyD),
            (String::from("turn_end"), KeyCode::KeyP),
            (String::from("card_left"), KeyCode::KeyA),
            (String::from("card_right"), KeyCode::KeyD),
            (String::from("card_submit"), KeyCode::ArrowUp),
        ],
        vec![],
    );
    let input_mapping_1 = InputMapping::new(
        vec![
            (String::from("card_mode"), KeyCode::ShiftLeft),
            (String::from("move_forward"), KeyCode::KeyW),
            (String::from("move_back"), KeyCode::KeyS),
            (String::from("move_left"), KeyCode::KeyA),
            (String::from("move_right"), KeyCode::KeyD),
            (String::from("turn_end"), KeyCode::KeyP),
            (String::from("card_left"), KeyCode::KeyA),
            (String::from("card_right"), KeyCode::KeyD),
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
        // GameMode::new_local_splitscreen_2p_horizontal(input_mapping_0, input_mapping_1),
        GameMode::new_local_single(input_mapping_0),
    );
    println!("init game end");
}
