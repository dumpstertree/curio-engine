use crate::AssetMappingUIDs;
use crate::cards::card_instance::CardInstance;
use crate::ecs::components::component_ball::ComponentBall;
use crate::ecs::components::component_card::ComponentCard;
use crate::ecs::components::component_energy_token::ComponentEnergyToken;
use crate::ecs::components::component_player::ComponentPlayer;
use crate::ecs::components::component_ui_ball_state::ComponentUIBallState;
use crate::ecs::components::component_ui_score::ComponentUIScoreState;
use crate::ecs::components::component_ui_turn::ComponentUITurnState;
use crate::ecs::components::component_view_player::ComponentViewPlayer;
use crate::game_events::GameEvents;
use crate::state::peer::state_peer_entity_ids::{EntityIDTypes, StateEntityIDs};
use crate::state::state_deck::{Deck, StateDeck};
use crate::state::state_teams::{StateTeamAssignments, Teams};
use built_in::component::component_renderer_animated::RendererAnimated;
use built_in::component::component_renderer_static::Renderer;
use built_in::component::component_renderer_text::{ComponentRendererText, RendererCommon};
use built_in::component::component_transform::Transform;
use built_in_state::state_camera::CameraState;
use core::collections::game_state;
use core::collections::quaternion::Quaternion;
use core::collections::vector2::Vector2;
use core::collections::vector3::Vector3;
use core::gameplay::ecs::traits::ecs_event_reciever::{self, InstanceLimiter};
use core::io::asset_loader::AssetLoader;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::global_ecs_system_event_reciever;
use hecs::World;
use std::sync::Arc;

#[derive(Default)]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct ECSSystemGamePointScored {}

impl InstanceLimiter for ECSSystemGamePointScored {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl ecs_event_reciever::EventReciever<GameEvents> for ECSSystemGamePointScored {
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut World, _event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ExplorationDidRoomEnterCombat(_, _) => {
                println!("enter combat room");
                // add all entities to world
                Self::spawn_entities(game_state, world);
                // add all energy to world
                Self::spawn_ui_energy(game_state, world);
                // add background to world
                Self::spawn_background(game_state, world);
                // add cards to world
                Self::spawn_ui_cards(game_state, world);
                // add score to world
                Self::spawn_ui_score(game_state, world);
                // add turn to world
                Self::spawn_ui_turn(game_state, world);
                // add ball mode
                Self::spawn_ui_ball_mode(game_state, world);
                // add score to world
                Self::spawn_ball(game_state, world);
            }
            _ => {}
        }
    }
}
impl ECSSystemGamePointScored {
    pub fn spawn_ball(game_state: &mut GameState, world: &mut World) {
        let mut r = Renderer::default();
        r = r.set_asset(Some(AssetLoader::load_model_static_from_database(AssetMappingUIDs::Ball.uid())));
        let e = world.spawn((Transform::default(), r, ComponentBall::default()));
        game_state.edit::<StateEntityIDs>(|x| {
            x.add(EntityIDTypes::Ball, e);
        });
    }
    pub fn spawn_ui_score(game_state: &mut GameState, world: &mut World) {
        let e = world.spawn((Transform::default(), ComponentRendererText::default(), ComponentUIScoreState::default()));
        game_state.edit::<StateEntityIDs>(|x| {
            x.add(EntityIDTypes::UIScore, e);
        });
    }
    pub fn spawn_ui_ball_mode(game_state: &mut GameState, world: &mut World) {
        let e = world.spawn((Transform::default(), ComponentRendererText::default(), ComponentUIBallState::default()));
        game_state.edit::<StateEntityIDs>(|x| {
            x.add(EntityIDTypes::UIBallMode, e);
        });
    }
    pub fn spawn_ui_turn(game_state: &mut GameState, world: &mut World) {
        let e = world.spawn((Transform::default(), ComponentRendererText::default(), ComponentUITurnState::default()));
        game_state.edit::<StateEntityIDs>(|x| {
            x.add(EntityIDTypes::UITurn, e);
        });
    }
    pub fn spawn_ui_cards(game_state: &mut GameState, world: &mut World) {
        let state_deck = game_state.get::<StateDeck>();
        let state_teams = game_state.get::<StateTeamAssignments>();

        let my_deck: &Deck;
        if let Some(deck) = state_deck.deck.get(&game_state.instance_id) {
            my_deck = deck
        } else if let Some(_) = state_deck
            .deck
            .get(&state_teams.team_assignments.get(&Teams::Red).unwrap()[0])
        {
            // my_deck = deck;
            return;
        } else {
            return;
        }
        let camera_state = game_state.get::<CameraState>();

        for card in &my_deck.all_cards {
            Self::spawn_card(world, card.clone(), camera_state.cameras.rotation, game_state);
        }
    }
    pub fn spawn_background(game_state: &mut GameState, world: &mut World) {
        // let spine = AssetLoader::load_spine_from_path("path");
        let asset_court = AssetLoader::load_model_static_from_database(AssetMappingUIDs::Court.uid());

        // court
        let e = world.spawn((
            Transform::default()
                .set_position(Vector3::new(0.0, 0.0, 0.0))
                .set_rotation(Quaternion::from_euler(Vector3::new(0.0, -90.0, 0.0))),
            Renderer::default().set_asset(Some(asset_court)),
        ));

        game_state.edit::<StateEntityIDs>(|x| {
            x.add(EntityIDTypes::Background, e);
        });
    }
    pub fn spawn_ui_energy(game_state: &mut GameState, world: &mut World) {
        let asset = AssetLoader::load_model_animated_from_database(AssetMappingUIDs::EnergyToken.uid());
        for team in game_state.get::<StateTeamAssignments>().team_assignments {
            for player_id in team.1 {
                for i in 0..9 {
                    let mut r = RendererAnimated::default();
                    r.set_fps(60).set_asset(Some(asset.clone()));
                    // r.set_animation("add", true);
                    let e = world.spawn((ComponentEnergyToken::default().set_index(i), ComponentPlayer::default().set_player_id(player_id), Transform::default(), r));

                    game_state.edit::<StateEntityIDs>(|x| {
                        x.add(EntityIDTypes::UIEnergy, e);
                    });
                }
            }
        }
    }
    pub fn spawn_entities(game_state: &mut GameState, world: &mut World) {
        let asset_goblin = AssetLoader::load_model_animated_from_database(AssetMappingUIDs::Goblin.uid());

        let state_teams = game_state.get::<StateTeamAssignments>();
        for team in Teams::all() {
            if let Some(guids) = state_teams.team_assignments.get(&team) {
                for guid in guids {
                    let mut rend = RendererAnimated::default();
                    rend.set_asset(Some(asset_goblin.clone()));
                    // players
                    let e = world.spawn((
                        ComponentViewPlayer::default(),
                        ComponentPlayer::default().set_player_id(*guid),
                        Transform::default()
                            .set_position(Vector3::new(-5.0, -5.0, 10.0))
                            .set_rotation(Quaternion::from_euler(Vector3::new(1.0, 0.0, 1.0))),
                        rend,
                    ));
                    game_state.edit::<StateEntityIDs>(|x| {
                        x.add(EntityIDTypes::Entities, e);
                    });
                }
            }
        }
    }
    fn spawn_card(world: &mut World, x: Arc<CardInstance>, rotation: Quaternion, game_state: &mut GameState) {
        let asset = AssetLoader::load_model_static_from_database(AssetMappingUIDs::Card.uid());
        let parent = world.spawn((Transform::default().set_rotation(rotation), Renderer::default().set_asset(Some(asset.clone())), ComponentCard::default().set_instance(x.clone())));
        // create description
        let mut desc = x.get_master().description.clone();
        for life in x.get_attributes_lifecycle() {
            match life {
                crate::state::state_deck::CardAttributeLifecycle::Quick => desc = desc + ".QUICK. ",
                crate::state::state_deck::CardAttributeLifecycle::Exhuast => desc = desc + ".EXHUAST. ",
                crate::state::state_deck::CardAttributeLifecycle::Linger => desc = desc + ".LINGER. ",
                crate::state::state_deck::CardAttributeLifecycle::Light => desc = desc + ".LIGHT. ",
            }
        }
        let mut r = ComponentRendererText::default();
        r.set_bounds(Vector2::new(0.25, 0.2));
        r.set_font_size(0.02);
        r.set_contents(&desc);
        r.set_parent(Some(parent));
        let e0 = world.spawn((
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
        let e1 = world.spawn((
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
        let e2 = world.spawn((
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
        let e3 = world.spawn((
            r,
            Transform::default()
                .set_position(Vector3::back() * 0.02 + Vector3::down() * 0.25 + Vector3::right() * 0.135)
                .set_rotation(Quaternion::from_euler(Vector3::new(0.0, 180.0, 0.0)))
                .set_parent(Some(parent)),
        ));
        game_state.edit::<StateEntityIDs>(|x: &mut StateEntityIDs| {
            x.add(EntityIDTypes::UICards, parent);
            x.add(EntityIDTypes::UICards, e0);
            x.add(EntityIDTypes::UICards, e1);
            x.add(EntityIDTypes::UICards, e2);
            x.add(EntityIDTypes::UICards, e3);
        });
    }
}
