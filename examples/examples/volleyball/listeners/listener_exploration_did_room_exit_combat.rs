use crate::ecs::components::component_card::ComponentCard;
use crate::ecs::components::component_energy_token::ComponentEnergyToken;
use crate::ecs::components::component_player::ComponentPlayer;
use crate::ecs::components::component_view_player::ComponentViewPlayer;
use crate::game_events::GameEvents;
use crate::state::peer::state_peer_entity_ids::{EntityIDTypes, StateEntityIDs};
use built_in::component::component_renderer_animated::RendererAnimated;
use built_in::component::component_renderer_static::Renderer;
use built_in::component::component_transform::Transform;
use core::gameplay::ecs::traits::ecs_event_reciever::{self, InstanceLimiter};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::global_ecs_system_event_reciever;
use hecs::World;

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
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ExplorationDidRoomExitCombat(_, _) => {
                println!("exit combat room");
                // add all entities to world
                Self::despawn_entities(game_state, world);
                // add all energy to world
                Self::despawn_energy(game_state, world);
                // add background to world
                Self::despawn_background(game_state, world);
                // add cards to world
                Self::despawn_ui_cards(game_state, world);
                // add score to world
                Self::despawn_ui_score(game_state, world);
                // add turn to world
                Self::despawn_ui_turn(game_state, world);
                // add ball mode
                Self::despawn_ui_ball_mode(game_state, world);
                // add score to world
                Self::despawn_ball(game_state, world);
            }
            _ => {}
        }
    }
}

impl ECSSystemGamePointScored {
    fn despawn_entities(game_state: &mut GameState, world: &mut World) {
        let id = EntityIDTypes::Entities;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            let _ = world.despawn(e);
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }
    fn despawn_energy(game_state: &mut GameState, world: &mut World) {
        let id = EntityIDTypes::UIEnergy;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            let _ = world.despawn(e);
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }
    fn despawn_background(game_state: &mut GameState, world: &mut World) {
        let id = EntityIDTypes::Background;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            let _ = world.despawn(e);
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }
    fn despawn_ui_cards(game_state: &mut GameState, world: &mut World) {
        let id = EntityIDTypes::UICards;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            let _ = world.despawn(e);
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }
    fn despawn_ui_score(game_state: &mut GameState, world: &mut World) {
        let id = EntityIDTypes::UIScore;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            let _ = world.despawn(e);
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }
    fn despawn_ui_turn(game_state: &mut GameState, world: &mut World) {
        let id = EntityIDTypes::UITurn;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            let _ = world.despawn(e);
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }
    fn despawn_ui_ball_mode(game_state: &mut GameState, world: &mut World) {
        let id = EntityIDTypes::UIBallMode;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            let _ = world.despawn(e);
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }
    fn despawn_ball(game_state: &mut GameState, world: &mut World) {
        let id = EntityIDTypes::Ball;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            let _ = world.despawn(e);
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }
}
