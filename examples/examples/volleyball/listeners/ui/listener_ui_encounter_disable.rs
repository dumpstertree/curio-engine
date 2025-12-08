use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::{
        ecs::traits::ecs_event_reciever::{self, InstanceLimiter},
        world_context::WorldContext,
    },
};
use ecs_event::global_ecs_system_event_reciever;
use hecs::World;

use crate::{
    game_events::GameEvents,
    state::peer::state_peer_entity_ids::{EntityIDTypes, StateEntityIDs},
};

#[derive(Default)]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct Listener {}

// Impl - Instance
impl InstanceLimiter for Listener {
    fn is_enabled(&mut self, _game_state: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
// Impl - Listener
impl ecs_event_reciever::EventReciever<GameEvents> for Listener {
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut WorldContext, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::DisableUICombat => {
                println!("disable combat");
                // add all energy to world
                Self::despawn_ui_energy(game_state, world);
                // add cards to world
                Self::despawn_ui_cards(game_state, world);
                // add score to world
                Self::despawn_ui_score(game_state, world);
                // add turn to world
                Self::despawn_ui_turn(game_state, world);
                // add ball mode
                Self::despawn_ui_ball_mode(game_state, world);
            }
            _ => {}
        }
    }
}
impl Listener {
    fn despawn_ui_energy(game_state: &mut GameState, world: &mut WorldContext) {
        let id = EntityIDTypes::UIEnergy;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            // let _ = world.despawn(e);
            e.destroy();
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }
    fn despawn_ui_cards(game_state: &mut GameState, world: &mut WorldContext) {
        let id = EntityIDTypes::UICards;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            // let _ = world.despawn(e);
            e.destroy();
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }
    fn despawn_ui_score(game_state: &mut GameState, world: &mut WorldContext) {
        let id = EntityIDTypes::UIScore;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            // let _ = world.despawn(e);
            e.destroy();
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }
    fn despawn_ui_turn(game_state: &mut GameState, world: &mut WorldContext) {
        let id = EntityIDTypes::UITurn;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            // let _ = world.despawn(e);
            e.destroy();
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }
    fn despawn_ui_ball_mode(game_state: &mut GameState, world: &mut WorldContext) {
        let id = EntityIDTypes::UIBallMode;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            // let _ = world.despawn(e);
            e.destroy();
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }
}
