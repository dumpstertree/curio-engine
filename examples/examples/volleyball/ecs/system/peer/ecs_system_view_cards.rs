use crate::AssetMappingUIDs;
use crate::cards::card_instance::CardInstance;
use crate::ecs::components::component_card::ComponentCard;
use crate::game_board::GameBoard;
use crate::state::peer::state_peer_input_mode::{InputModes, StatePeerInputMode};
use crate::state::peer::state_peer_selected_card::StatePeerSelectedCards;
use crate::state::state_deck::{self, StateDeck};
use crate::state::state_energy::StateEnergy;
use crate::state::state_position_ball::StatePositionBall;
use crate::state::state_position_player::StatePositionEntities;
use crate::state::state_teams::StateTeamAssignments;
use built_in::component::component_renderer_static::Renderer;
use built_in::component::component_renderer_text::{ComponentRendererText, RendererCommon};
// use built_in::component::component_renderer::Renderer;
use built_in::component::component_transform::Transform;
use built_in_state::state_camera::CameraState;
use built_in_state::state_time::TimeState;
use core::collections::color::Color;
use core::collections::game_state;
use core::collections::quaternion::Quaternion;
use core::collections::vector2::Vector2;
use core::collections::vector2_int::Vector2Int;
use core::collections::vector3::Vector3;
use core::io::asset_loader::AssetLoader;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};
use ecs_system::global_ecs_system;
use hecs::World;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[global_ecs_system]
pub struct ECSSystemViewCards {
    // asset: Arc<ModelAsset>,
    // asset_card: HashMap<String, Option<Arc<ModelAsset>>>,
    cnt: i32,
}
impl ECSSystemEventless for ECSSystemViewCards {
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalPeer, NetworkModes::OnlinePeer]
    }
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn init(&mut self, game_state: &mut GameState, world: &mut World, _: &mut EventQueue) {
        // self.asset_card = AssetLoader::load_model_static_from_database(AssetMappingUIDs::Card.uid());
    }

    fn tick(&mut self, game_state: &mut GameState, world: &mut World, events: &mut EventQueue) {
        self.cnt += 1;
        if self.cnt < 15 {
            return;
        }
        if self.cnt == 15 {
            let state_deck = game_state.get::<StateDeck>();
            let my_deck = state_deck.deck.get(&game_state.instance_id).unwrap();
            let camera_state = game_state.get::<CameraState>();

            for card in &my_deck.all_cards {
                self.spawn_card(world, card.clone(), camera_state.cameras.rotation, game_state);
            }
            // run the ai
            // println!("will running the ai");
            // run_ai(game_state, events);
            // println!("did running the ai");
        }
        let y_selected = 0.25;
        let y_unselected = 0.5;
        let spacing = 0.5;
        let z_selected = 1.0;
        let z_unselected = 1.5;

        let state_input_mode = game_state.get::<StatePeerInputMode>();
        let is_manuever_mode = state_input_mode.mode == InputModes::Manuever;
        for (_, (card, transform, renderer)) in world
            .query::<(&ComponentCard, &mut Transform, &mut Renderer)>()
            .iter()
        {
            let state_deck = game_state.get::<StateDeck>();
            let my_deck = state_deck.deck.get(&game_state.instance_id).unwrap();
            let camera_state = game_state.get::<CameraState>();
            let state_selected = game_state.get::<StatePeerSelectedCards>();

            let Some(card_instance) = &card.card_instance else {
                continue;
            };

            match my_deck.get_location(card_instance.clone()) {
                state_deck::CardLocation::Deck(index) => {
                    let pos = camera_state.cameras.position + (camera_state.cameras.rotation * Vector3::new(0.5, 0.5, 1.0));
                    let rot = camera_state.cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, -0.0));
                    transform.position = Vector3::lerp(transform.position, pos, 0.2);
                    transform.rotation = transform.rotation.slerp(rot, 0.2);
                    transform.scale = Vector3::lerp(transform.scale, Vector3::one() * 0.25, 0.2);
                    renderer.set_enabled(index == 0);
                }
                state_deck::CardLocation::Discard(index) => {
                    let pos = camera_state.cameras.position + (camera_state.cameras.rotation * Vector3::new(-0.5, 0.5, 1.0));
                    let rot = camera_state.cameras.rotation * Quaternion::from_euler(Vector3::new(0.0, 180.0, -0.0));

                    transform.position = Vector3::lerp(transform.position, pos, 0.2);
                    transform.rotation = transform.rotation.slerp(rot, 0.2);
                    transform.scale = Vector3::lerp(transform.scale, Vector3::one() * 0.25, 0.2);
                    renderer.set_enabled(index == 0);
                }
                state_deck::CardLocation::Hand(index) => {
                    let mut z = z_unselected;
                    let mut y = y_unselected;
                    let is_met = card_instance.has_statement(game_state, game_state.instance_id);

                    let state_team = game_state
                        .get::<StateTeamAssignments>()
                        .team_for(&game_state.instance_id)
                        .unwrap();
                    if is_manuever_mode && state_selected.index == index {
                        z = z_selected;
                        y = y_selected;
                    }

                    if !is_manuever_mode {
                        if is_met {
                            y = y + 0.4;
                        } else {
                            y = y + 0.5;
                        }
                    }

                    let dir = state_team.convert_dir(1, 0).0 as f32;
                    let pos = camera_state.cameras.position + (camera_state.cameras.rotation * Vector3::forward()) * z + Vector3::right() * dir * ((index - state_selected.index) as f32 * spacing) + camera_state.cameras.rotation * Vector3::down() * y;
                    let rot = camera_state.cameras.rotation;

                    transform.position = Vector3::lerp(transform.position, pos, 0.2);
                    transform.rotation = transform.rotation.slerp(rot, 0.2);
                    transform.scale = Vector3::lerp(transform.scale, Vector3::one(), 0.2);
                    renderer.set_enabled(true);

                    let col_spell = Color::new_hex("#f7a5f3");
                    let col_persistent = Color::new_hex("#f7c8a5");
                    let col_bump = Color::new_hex("#4efff9");
                    let col_set = Color::new_hex("#abff4e");
                    let col_spike = Color::new_hex("#ff4e85");

                    // let mut cur_tint = renderer.get_tint();
                    if is_met {
                        match &card.card_instance.clone().unwrap().get_manuever_type() {
                            state_deck::CardTypes::Serve => renderer.set_tint(col_persistent),
                            state_deck::CardTypes::Rest => renderer.set_tint(col_persistent),
                            state_deck::CardTypes::Bump => renderer.set_tint(col_bump),
                            state_deck::CardTypes::Set => renderer.set_tint(col_set),
                            state_deck::CardTypes::Spike => renderer.set_tint(col_spike),
                            state_deck::CardTypes::Move => renderer.set_tint(Color::white()),
                            state_deck::CardTypes::Spell => renderer.set_tint(col_spell),
                        }
                        // renderer.set_tint(cur_tint);
                    } else {
                        match &card.card_instance.clone().unwrap().get_manuever_type() {
                            state_deck::CardTypes::Serve => renderer.set_tint(col_persistent * 0.15),
                            state_deck::CardTypes::Rest => renderer.set_tint(col_persistent * 0.15),
                            state_deck::CardTypes::Bump => renderer.set_tint(col_bump * 0.15),
                            state_deck::CardTypes::Set => renderer.set_tint(col_set * 0.15),
                            state_deck::CardTypes::Spike => renderer.set_tint(col_spike * 0.15),
                            state_deck::CardTypes::Move => renderer.set_tint(Color::white() * 0.15),
                            state_deck::CardTypes::Spell => renderer.set_tint(col_persistent * 0.15),
                        }
                    }
                }
            }
        }
    }
}
impl ECSSystemViewCards {
    fn spawn_card(&self, world: &mut World, x: Arc<CardInstance>, rotation: Quaternion, game_state: &GameState) {
        let asset = AssetLoader::load_model_static_from_database(AssetMappingUIDs::Card.uid());
        let parent = world.spawn((Transform::default().set_rotation(rotation), Renderer::default().set_asset(Some(asset.clone())), ComponentCard::default().set_instance(x.clone())));
        // create description
        let mut r = ComponentRendererText::default();
        r.set_bounds(Vector2::new(0.25, 0.2));
        r.set_font_size(0.02);
        r.set_contents(&x.get_master().description);
        r.set_parent(Some(parent));
        world.spawn((
            r,
            Transform::default()
                .set_position(Vector3::back() * 0.02 + Vector3::down() * 0.155)
                .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0)))
                .set_parent(Some(parent)),
        ));
        // create title
        let mut r = ComponentRendererText::default();
        r.set_bounds(Vector2::new(0.5, 0.2));
        r.set_font_size(0.03);
        r.set_contents(&x.get_title());
        r.set_parent(Some(parent));
        world.spawn((
            r,
            Transform::default()
                .set_position(Vector3::back() * 0.02 + Vector3::up() * 0.235)
                .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0)))
                .set_parent(Some(parent)),
        ));
        // create type
        let mut r = ComponentRendererText::default();
        r.set_bounds(Vector2::new(0.25, 0.2));
        r.set_font_size(0.02);
        r.set_contents(&format!("{}", x.get_manuever_type()));
        r.set_parent(Some(parent));
        world.spawn((
            r,
            Transform::default()
                .set_position(Vector3::back() * 0.02 + Vector3::down() * 0.06)
                .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0)))
                .set_parent(Some(parent)),
        ));
        // create cost
        let mut r = ComponentRendererText::default();
        r.set_bounds(Vector2::new(0.25, 0.2));
        r.set_font_size(0.03);
        r.set_contents(&x.get_cost(&game_state, game_state.instance_id).to_string());
        r.set_parent(Some(parent));
        world.spawn((
            r,
            Transform::default()
                .set_position(Vector3::back() * 0.02 + Vector3::down() * 0.25 + Vector3::right() * 0.135)
                .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0)))
                .set_parent(Some(parent)),
        ));
    }
}
impl ECSSystemViewCards {
    fn run_ai(&self, world: &mut World, game_state: &mut GameState) {}
}

// #[derive(Clone)]
// struct AIGameSimulation {
//     // player
//     player_energy: i32,
//     player_deck: Vec<Arc<CardInstance>>,
//     player_hand: Vec<Arc<CardInstance>>,
//     player_discard: Vec<Arc<CardInstance>>,

//     // ai
//     ai_energy: i32,
//     ai_deck: Vec<Arc<CardInstance>>,
//     ai_hand: Vec<Arc<CardInstance>>,
//     ai_discard: Vec<Arc<CardInstance>>,
// }

// impl mcts::GameState for AIGameSimulation {
//     type Move = Move;
//     type Player = ();
//     type MoveList = Vec<Move>;

//     fn current_player(&self) -> Self::Player {
//         ()
//     }
//     fn available_moves(&self) -> Vec<Move> {
//         // let x = self.0;
//         // if x == 100 { vec![] } else { vec![Move::Add, Move::Sub] }
//         vec![]
//     }
//     fn make_move(&mut self, mov: &Self::Move) {
//         match *mov {
//             Move::Move(_) => todo!(),
//             Move::Play(_) => todo!(),
//             Move::Rest => todo!(),
//             Move::End => todo!(),
//         }
//     }
// }
// impl mcts::transposition_table::TranspositionHash for AIGameSimulation {
//     fn hash(&self) -> u64 {
//         // self.0 as u64
//         0
//     }
// }

// #[derive(Clone)]
// enum Move {
//     Move(Vector2Int),
//     Play(Arc<CardInstance>),
//     Rest,
//     End,
// }
// struct AiGameEvaluator {}
// impl mcts::Evaluator<MyMCTS> for AiGameEvaluator {
//     type StateEvaluation = i64;

//     fn evaluate_new_state(&self, state: &AIGameSimulation, moves: &Vec<Move>, _: Option<mcts::SearchHandle<MyMCTS>>) -> (Vec<()>, i64) {
//         // (vec![(); moves.len()], state.0)
//     }
//     fn interpret_evaluation_for_player(&self, evaln: &i64, _player: &()) -> i64 {
//         *evaln
//     }
//     fn evaluate_existing_state(&self, _: &AIGameSimulation, evaln: &i64, _: mcts::SearchHandle<MyMCTS>) -> i64 {
//         *evaln
//     }
// }

// #[derive(Default)]
// struct MyMCTS;

// impl mcts::MCTS for MyMCTS {
//     type State = AIGameSimulation;
//     type Eval = AiGameEvaluator;
//     type NodeData = ();
//     type ExtraThreadData = ();
//     type TreePolicy = mcts::tree_policy::UCTPolicy;
//     type TranspositionTable = mcts::transposition_table::ApproxTable<Self>;

//     fn cycle_behaviour(&self) -> mcts::CycleBehaviour<Self> {
//         mcts::CycleBehaviour::UseCurrentEvalWhenCycleDetected
//     }
// }
