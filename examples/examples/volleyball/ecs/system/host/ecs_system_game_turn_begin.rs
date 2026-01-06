use crate::{
    game_events::GameEvents,
    state::{
        self,
        host::{state_card_attribute_modifier_stack::StateCardAttributeModifierStack, state_heat::StateHeat},
        state_deck::StateDeck,
        state_energy::StateEnergy,
        state_teams::StateTeamAssignments,
        state_turn::StateTurn,
    },
};
use built_in_state::state_time::TimeState;
use system_component_default_gameplay::{ecs_event_reciever::{EventReciever, InstanceLimiter}, ecs_system::ECSSystemEventless, world_context::WorldContext};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::{

    },
};
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;
use serde::de;

#[global_ecs_system]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct ECSSystemGameTurnBegin {}
impl InstanceLimiter for ECSSystemGameTurnBegin {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
impl ECSSystemEventless for ECSSystemGameTurnBegin {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut WorldContext) -> bool {
        true
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
impl EventReciever<GameEvents> for ECSSystemGameTurnBegin {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut WorldContext, events: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::TurnBegin(id) => {
                // end this turn
                println!("Instance: {}. Begin Turn {}", game_state.instance_id, id);

                game_state.edit::<StateTurn>(|x| {
                    x.active_instance_id = *id;
                });

                for guid in game_state
                    .get::<StateTeamAssignments>()
                    .team_assignments
                    .get(id)
                    .unwrap()
                {
                    let state_modifiers = game_state.get::<StateCardAttributeModifierStack>();
                    let mod_stack = state_modifiers.get_flat_stack_for_entity(*guid);

                    let state_energy = game_state.get::<StateEnergy>();
                    let cur_energy = state_energy.all_players.get(guid).unwrap_or(&(0, 0));

                    println!("cur energy {}", cur_energy.0);
                    game_state.edit::<StateHeat>(|x| {
                        if !x.all_players.contains_key(guid) {
                            x.all_players.insert(*guid, cur_energy.0);
                        } else {
                            let c = x.all_players[guid];
                            x.all_players.insert(*guid, c + cur_energy.0);
                        }
                    });

                    println!("heat {}", game_state.get::<StateHeat>().all_players[guid]);
                    // update energy
                    game_state.edit::<StateEnergy>(|x| {
                        if let Some(y) = x.all_players.get_mut(guid) {
                            y.0 = y.1 + mod_stack.energy;
                        }
                    });
                }

                println!("send did turn begin");
                events.enqueue_event(GameEvents::DidTurnBegin(*id));
            }
            _ => {}
        }
    }
}
