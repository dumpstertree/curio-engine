use crate::UIViewTypes;
use crate::game_events::GameEvents;
use crate::listeners::listener_ui_set_mode::UITypes;
use crate::state::peer::state_peer_entity_ids::{EntityIDTypes, StateEntityIDs};
use curio_core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    collections::network_modes::NetworkModes
};
use gameplay::built_in::impulse::ui_events::UIEvents;
use gameplay::context_3d::Context3D;
use gameplay::traits::{impulse::Impulse, scope::Scope};
use impulse::impulse;

#[derive(Default)]
#[impulse(GameEvents)]
pub struct Listener {}

impl Scope for Listener {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Impulse<GameEvents> for Listener {
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut Context3D, event_queue: &mut EventQueue, event: &GameEvents) {
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

                event_queue.enqueue_event(UIEvents::Close(UIViewTypes::HudEncounterBallMode));
                event_queue.enqueue_event(UIEvents::Close(UIViewTypes::HudEncounterTurn));
                event_queue.enqueue_event(UIEvents::Close(UIViewTypes::HudEncounterScore));
                event_queue.enqueue_event(UIEvents::Close(UIViewTypes::HudEncounterEnergy));
                event_queue.enqueue_event(UIEvents::Close(UIViewTypes::HudPreviouslyPlayed));
                event_queue.enqueue_event(UIEvents::Close(UIViewTypes::PanelRewards));
                event_queue.enqueue_event(UIEvents::Close(UIViewTypes::HUDHeat));
                event_queue.enqueue_event(UIEvents::Close(UIViewTypes::HudEncounterCards));
            }
            _ => {}
        }
    }
}

impl Listener {
    fn despawn_entities(game_state: &mut GameState, _world: &mut Context3D) {
        let id = EntityIDTypes::Entities;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            // let _ = world.despawn(e);
            e.destroy();
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }
    fn despawn_background(game_state: &mut GameState, _world: &mut Context3D) {
        let id = EntityIDTypes::Background;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            // let _ = world.despawn(e);
            e.destroy();
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }

    fn despawn_ball(game_state: &mut GameState, _world: &mut Context3D) {
        let id = EntityIDTypes::Ball;
        for e in game_state.get::<StateEntityIDs>().get(id.clone()) {
            // let _ = world.despawn(e);
            e.destroy();
        }
        game_state.edit::<StateEntityIDs>(|x| x.clear(id.clone()));
    }
}
