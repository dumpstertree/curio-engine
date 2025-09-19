use crate::{
    cards::card_modifier::CardModifier,
    game_events::GameEvents,
    state::{
        host::state_card_attribute_modifier_stack::{self, StateCardAttributeModifierStack},
        state_energy::StateEnergy,
        state_position_ball::StatePositionBall,
        state_teams::StateTeamAssignments,
        state_turn::StateTurn,
    },
};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::{ecs_event_reciever, ecs_system::ECSSystemEventless},
};
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
pub struct EventReciever {}
impl ECSSystemEventless for EventReciever {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
#[global_ecs_system_event_reciever(GameEvents)]
impl ecs_event_reciever::EventReciever<GameEvents> for EventReciever {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, _: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::ApplyCardAttributeEventMoveBallForward(move_forward, entity_id, card_id) => {
                // get the stack for the current use case
                let state_card_attribute_modifier_stack = game_state.get_value2::<StateCardAttributeModifierStack>();
                let active_modifiers = CardModifier::flatten(&vec![
                    &state_card_attribute_modifier_stack
                        .get_flat_stack_for_entity(*entity_id)
                        .clone(),
                    &state_card_attribute_modifier_stack
                        .get_flat_stack_for_card(*card_id)
                        .clone(),
                ]);
                state_card_attribute_modifier_stack.get_flat_stack_for_card(0);
                let cur_turn = &game_state.get_value2::<StateTurn>().active_instance_id; // get the team for this player
                let team = &game_state
                    .get_value2::<StateTeamAssignments>()
                    .team_for(cur_turn)
                    .unwrap();

                // edit ball position
                game_state.edit::<StatePositionBall>(|x| {
                    // convert based on team
                    let diff = team.convert_dir(0, move_forward + active_modifiers.range);
                    // move
                    x.collun = x.collun + diff.0;
                    x.row = x.row + diff.1;

                    println!("Ball moved for team ({}): ({},{}) -> ({},{})", team, x.collun - diff.0, x.row - diff.1, x.collun, x.row);
                })
            }
            _ => {}
        }
    }
}
