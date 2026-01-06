use crate::UIViewTypes;
use crate::game_events::GameEvents;
use crate::listeners::listener_ui_set_mode::UITypes;
use crate::state::peer::state_peer_entity_ids::{EntityIDTypes, StateEntityIDs};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::global_ecs_system_event_reciever;
use system_component_default_gameplay::ecs_event_reciever::{EventReciever, InstanceLimiter};
use system_component_default_gameplay::world_context::WorldContext;

#[derive(Default)]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct Listener {}

impl InstanceLimiter for Listener {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl EventReciever<GameEvents> for Listener {
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut WorldContext, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ExplorationDidRoomExitCombat(_, _) => {
                println!("exit combat room");
                // add all entities to world
                Self::despawn_entities(game_state, world);
                // add background to world
                Self::despawn_background(game_state, world);
                // add score to world
                Self::despawn_ball(game_state, world);

                // change ui
                event_queue.enqueue_event(GameEvents::SetUIMode(UITypes::None));

                event_queue.enqueue_event(system_component_default_gameplay::UIEvents::Close(UIViewTypes::HudEncounterBallMode));
                event_queue.enqueue_event(system_component_default_gameplay::UIEvents::Close(UIViewTypes::HudEncounterTurn));
                event_queue.enqueue_event(system_component_default_gameplay::UIEvents::Close(UIViewTypes::HudEncounterScore));
                event_queue.enqueue_event(system_component_default_gameplay::UIEvents::Close(UIViewTypes::HudEncounterEnergy));
                event_queue.enqueue_event(system_component_default_gameplay::UIEvents::Close(UIViewTypes::HudPreviouslyPlayed));
                event_queue.enqueue_event(system_component_default_gameplay::UIEvents::Close(UIViewTypes::PanelRewards));
                event_queue.enqueue_event(system_component_default_gameplay::UIEvents::Close(UIViewTypes::HUDHeat));
                event_queue.enqueue_event(system_component_default_gameplay::UIEvents::Close(UIViewTypes::HudEncounterCards));
            }
            _ => {}
        }
    }
}

impl Listener {
    fn despawn_entities(game_state: &mut GameState, world: &mut WorldContext) {
        let id = EntityIDTypes::Entities;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            // let _ = world.despawn(e);
            e.destroy();
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }
    fn despawn_background(game_state: &mut GameState, world: &mut WorldContext) {
        let id = EntityIDTypes::Background;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            // let _ = world.despawn(e);
            e.destroy();
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }

    fn despawn_ball(game_state: &mut GameState, world: &mut WorldContext) {
        let id = EntityIDTypes::Ball;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            // let _ = world.despawn(e);
            e.destroy();
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }
}
