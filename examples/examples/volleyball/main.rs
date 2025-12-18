pub mod game_board;
pub mod game_events;
pub mod cards {
    pub mod deck_library;
    pub mod card_attribute_fillers {
        pub mod attribute_filler_ai;
        pub mod attribute_filler_player;
    }
    pub mod enums {
        pub mod attribute_clear_flag;
        pub mod card_events;
        pub mod simulation_manuevers;
    }
    pub mod card_attributes_targets {
        pub mod attribute_target_type_cards;
        pub mod attribute_target_type_entities;
        pub mod attribute_target_type_players;
        pub mod attribute_target_type_tiles;
    }
    pub mod card_event_runner;
    pub mod card_event_runner_recievers {
        pub(crate) mod clear_modifier_all;
        pub(crate) mod clear_modifier_for_flag;
        pub(crate) mod event_card_discard;
        pub(crate) mod event_card_draw;
        pub(crate) mod event_change_ball_mode;
        pub(crate) mod event_energy_edit;
        pub(crate) mod event_energy_fill;
        pub(crate) mod event_heat_drain;
        pub(crate) mod event_move_ball;
        pub(crate) mod event_move_entities;
        pub(crate) mod modifier_cost_for_entities;
        pub(crate) mod modifier_energy_for_entities;
        pub(crate) mod modifier_range_for_entities;
    }
    pub mod card_attributes {
        pub mod card_attribute_events;
        pub mod card_attribute_modifier;
        pub mod card_attribute_requirement;
    }
    pub mod card_instance;
    pub mod card_library;
    pub mod card_master;
    pub mod card_modifier;
    pub mod card_statement;
    pub mod card_dependencies {
        pub mod data_dep_empty;
        pub mod data_dep_filled;
        pub mod filled_card_attribute;
        pub mod filled_card_response;
        pub mod builder {
            pub mod data_dep_filled_all_permutations;
            pub mod data_dep_filled_for_modifiers;
            pub mod filled_attribute_with_permutation;
        }
    }
}
pub mod exploration {
    pub mod exploration_path;
}
pub mod listeners {
    pub mod listener_did_initialize_exploration;
    pub mod listener_encounter_did_pass;
    pub mod listener_encounter_failed;
    pub mod listener_encounter_passed;
    pub mod listener_encounter_scored;
    pub mod listener_exploration_did_pick_room_complete;
    pub mod listener_exploration_did_pick_room_start;
    pub mod listener_exploration_did_room_enter_combat;
    pub mod listener_exploration_did_room_enter_heal;
    pub mod listener_exploration_did_room_enter_shop;
    pub mod listener_exploration_did_room_exit_combat;
    pub mod listener_exploration_did_room_exit_heal;
    pub mod listener_exploration_did_room_exit_shop;
    pub mod listener_exploration_pick_room_complete;
    pub mod listener_exploration_request_leave_room;
    pub mod listener_exploration_room_enter;
    pub mod listener_exploration_room_exit;
    pub mod listener_finalize_encounter;
    pub mod listener_finalize_shop;
    pub mod listener_initialize_encounter;
    pub mod listener_initialize_exploration;
    pub mod listener_initialize_shop;
    pub mod listener_request_heal;
    pub mod listener_request_purchase;
    pub mod listener_ui_set_mode;
    pub mod ui {
        pub mod ui_hud_encounter_ball_mode;
        pub mod ui_hud_encounter_cards;
        pub mod ui_hud_encounter_energy;
        pub mod ui_hud_encounter_score;
        pub mod ui_hud_encounter_turn;
        pub mod ui_hud_previously_played;
        pub mod ui_hud_status;
        pub mod ui_panel_exploration;
        pub mod ui_panel_medic;
        pub mod ui_panel_rewards;
        pub mod ui_panel_shop;
    }
}
pub mod state {
    pub mod other {
        pub mod state_terminated;
    }
    pub mod state_ball_mode;
    pub mod state_controller;
    pub mod state_deck;
    pub mod state_energy;
    pub mod state_position_ball;
    pub mod state_position_player;
    pub mod state_score;
    pub mod state_teams;
    pub mod state_turn;
    pub mod peer {

        pub mod state_peer_entity_ids;
        pub mod state_peer_input_mode;
        pub mod state_peer_selected_card;
    }
    pub mod host {
        pub mod state_card_attribute_modifier_stack;
        pub mod state_currency;
        pub mod state_deck_exploration;
        pub mod state_enounter_mode;
        pub mod state_entity_visual;
        pub mod state_exploration;
        pub mod state_health_exploration;
        pub mod state_heat;
        pub mod state_play_history;
        pub mod state_shop;
    }
}
pub mod ai {
    pub mod ai_simulator;
    pub mod evalation;
    pub mod mcts;
    pub mod simulation;
    pub mod dependencies {
        pub mod simulation_evaluator;
        pub mod simulation_evaluators {
            pub mod custom_evaluator;
        }
        pub mod simulation_data_source;
        pub mod simulation_data_sources {
            pub mod custom_data_source;
        }
        pub mod simulation_delegate;
        pub mod simulation_delegates {
            pub mod custom_delegate;
        }
        pub mod simulation_hasher;
        pub mod simulation_hashers {
            pub mod custom_hasher;
        }
    }
    pub mod enums {
        pub mod fidelity;
        pub mod threading;
    }
}
pub mod event_recievers {}

pub mod ecs {

    pub mod components {
        pub mod component_ball;
        pub mod component_card;
        pub mod component_energy_token;
        pub mod component_player;
        pub mod component_ui_ball_state;
        pub mod component_ui_score;
        pub mod component_ui_turn;
        pub mod component_view_player;
    }
    pub mod system {

        pub mod peer {
            mod ecs_system_peer_did_turn_begin;
            mod ecs_system_peer_start;
            mod ecs_system_peer_update_input_mode;
            mod ecs_system_peer_view_ball_state;
            mod ecs_system_render;
            mod ecs_system_turn_end;
            mod ecs_system_turn_manuever;
            mod ecs_system_turn_move;
            mod ecs_system_view_cards;
            mod ecs_system_view_move_ball;
            mod ecs_system_view_move_player;
        }
        pub mod host {
            mod ecs_system_game_host_play_card;
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
use crate::{
    game_events::GameEvents,
    listeners::ui::{ui_hud_encounter_ball_mode, ui_hud_encounter_energy, ui_hud_encounter_score, ui_hud_encounter_turn, ui_hud_previously_played, ui_hud_status, ui_panel_exploration, ui_panel_medic, ui_panel_rewards, ui_panel_shop},
};
use core::{
    dumpster_engine::{CurioMetadata, GameMode, VersionNumber, WindowLayout},
    engine::{curio::Curio, curio_cabinet::CurioCabinet},
    input::{input_mapping::InputMapping, key_code::ButtonCode},
    io::{
        asset_database::{AssetDatabase, AssetDatabaseListing},
        asset_loader::AssetLoader,
    },
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use system_component_default_gameplay::{IUIEvent, SystemComponentDefaultGameplay};
use system_component_default_input::SystemComponentDefaultInput;
use system_component_default_networking::SystemComponentDefaultNetworking;
use system_component_default_physics::SystemComponentDefaultPhysics;
use system_component_default_rendering::SystemComponentDefaultGraphics;
use system_component_default_time::SystemComponentDefaultTime;

#[derive(Default, Hash, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetMappingUIDs {
    #[default]
    Invald,
    // animated
    Ball,
    Goblin,
    CharCrab,
    CharGrunt,
    EnergyToken,
    // static
    Court,
    Card,
}
impl AssetMappingUIDs {
    pub fn uid(&self) -> String {
        match self {
            AssetMappingUIDs::Ball => String::from("mesh_animated_ball"),
            AssetMappingUIDs::Goblin => String::from("mesh_animated_goblin"),
            AssetMappingUIDs::EnergyToken => String::from("mesh_animated_energy"),
            AssetMappingUIDs::CharCrab => String::from("mesh_animated_crab"),
            AssetMappingUIDs::CharGrunt => String::from("mesh_animated_grunt"),
            AssetMappingUIDs::Court => String::from("mesh_static_court"),
            AssetMappingUIDs::Card => String::from("mesh_static_card"),
            AssetMappingUIDs::Invald => String::from(""),
        }
    }
}
fn main() {
    AssetLoader::set_database(AssetDatabase::new_from_explicit(vec![
        // remote
        (AssetMappingUIDs::Goblin.uid(), AssetDatabaseListing::RemoteToCache(String::from("downloaded_spine.asset"), String::from("https://drive.dumpstertree.com/api/public/dl/z-P4xIan"))),
        (AssetMappingUIDs::EnergyToken.uid(), AssetDatabaseListing::RemoteToCache(String::from("energy.asset"), String::from("https://drive.dumpstertree.com/api/public/dl/A3DUMAqu"))),
        (AssetMappingUIDs::CharCrab.uid(), AssetDatabaseListing::Local(String::from("mesh/char_crab.asset"))),
        (AssetMappingUIDs::CharGrunt.uid(), AssetDatabaseListing::Local(String::from("mesh/char_grunt.asset"))),
        // local
        (AssetMappingUIDs::Court.uid(), AssetDatabaseListing::Local(String::from("mesh/court.glb"))),
        (AssetMappingUIDs::Card.uid(), AssetDatabaseListing::Local(String::from("mesh/card_empty.glb"))),
        (AssetMappingUIDs::Ball.uid(), AssetDatabaseListing::Local(String::from("mesh/ball.glb"))),
    ]));
    // create instance
    CurioCabinet::display_curio(
        CurioMetadata::new(
            "Volleyball", //
            "icon.png",
            VersionNumber::new(0, 1, 0),
        ),
        || {
            Curio::imbue(
                vec![
                    // components
                    SystemComponentDefaultTime::new(),
                    SystemComponentDefaultInput::new(),
                    SystemComponentDefaultPhysics::new(),
                    SystemComponentDefaultGameplay::<GameEvents, UIViewTypes>::new(),
                    SystemComponentDefaultGraphics::new(),
                    SystemComponentDefaultNetworking::new(),
                ],
                GameMode::new_local_single(
                    InputMapping::new(
                        vec![
                            (String::from("card_mode"), ButtonCode::ShiftLeft),
                            (String::from("move_forward"), ButtonCode::KeyW),
                            (String::from("move_back"), ButtonCode::KeyS),
                            (String::from("move_left"), ButtonCode::KeyA),
                            (String::from("move_right"), ButtonCode::KeyD),
                            (String::from("turn_end"), ButtonCode::KeyP),
                            (String::from("card_left"), ButtonCode::KeyA),
                            (String::from("card_right"), ButtonCode::KeyD),
                            (String::from("card_submit"), ButtonCode::ArrowUp),
                        ],
                        vec![],
                    ),
                    // InputMapping::new(
                    //     vec![
                    //         (String::from("card_mode"), ButtonCode::ShiftLeft),
                    //         (String::from("move_forward"), ButtonCode::KeyW),
                    //         (String::from("move_back"), ButtonCode::KeyS),
                    //         (String::from("move_left"), ButtonCode::KeyA),
                    //         (String::from("move_right"), ButtonCode::KeyD),
                    //         (String::from("turn_end"), ButtonCode::KeyP),
                    //         (String::from("card_left"), ButtonCode::KeyA),
                    //         (String::from("card_right"), ButtonCode::KeyD),
                    //         (String::from("card_submit"), ButtonCode::ArrowUp),
                    //     ],
                    //     vec![],
                    // ),
                ),
            )
        },
        WindowLayout::fullscreen_1080(),
    );
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
enum UIViewTypes {
    PanelMedic,
    PanelShop,
    PanelExploration,
    PanelRewards,
    HudEncounterEnergy,
    HudEncounterScore,
    HudEncounterTurn,
    HudEncounterBallMode,
    HudStatus,
    HudPreviouslyPlayed,
}
impl IUIEvent for UIViewTypes {
    fn new_instance(&self) -> Box<dyn system_component_default_gameplay::UIPanel> {
        match self {
            UIViewTypes::PanelMedic => ui_panel_medic::UIPanelMedic::new(),
            UIViewTypes::PanelShop => ui_panel_shop::UIPanelInstance::new(),
            UIViewTypes::PanelExploration => ui_panel_exploration::UIPanelInstance::new(),
            UIViewTypes::HudEncounterEnergy => ui_hud_encounter_energy::UIHUD::new(),
            UIViewTypes::HudEncounterScore => ui_hud_encounter_score::UIHUD::new(),
            UIViewTypes::HudEncounterTurn => ui_hud_encounter_turn::UIHUD::new(),
            UIViewTypes::HudEncounterBallMode => ui_hud_encounter_ball_mode::UIHUD::new(),
            UIViewTypes::PanelRewards => ui_panel_rewards::UIPanelInstance::new(),
            UIViewTypes::HudStatus => ui_hud_status::UIHUD::new(),
            UIViewTypes::HudPreviouslyPlayed => ui_hud_previously_played::UIHUD::new(),
        }
    }
}
impl Display for UIViewTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UIViewTypes::PanelMedic => f.write_str("medic"),
            UIViewTypes::PanelShop => f.write_str("shop"),
            UIViewTypes::PanelExploration => f.write_str("exploration"),
            UIViewTypes::HudEncounterEnergy => f.write_str("energy"),
            UIViewTypes::HudEncounterScore => f.write_str("score"),
            UIViewTypes::HudEncounterTurn => f.write_str("turn"),
            UIViewTypes::HudEncounterBallMode => f.write_str("ball mode"),
            UIViewTypes::PanelRewards => f.write_str("rewards"),
            UIViewTypes::HudStatus => f.write_str("status"),
            UIViewTypes::HudPreviouslyPlayed => f.write_str("previously played"),
        }
    }
}
