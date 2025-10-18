use crate::{
    card_parser::AttributeClearFlag,
    cards::{card_attribute_events::CardAttributeEvents, card_attribute_modifier::CardAttributeModifiers, card_modifier::CardModifier, data_dep_filled::DataDepsFilled},
    game_events::GameEvents,
    state::{
        host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack,
        state_ball_mode::{BallModes, StateBallMode},
        state_deck::{CardTypes, StateDeck},
        state_energy::StateEnergy,
        state_position_ball::StatePositionBall,
        state_position_player::StatePositionPlayer,
        state_teams::StateTeamAssignments,
        state_turn::StateTurn,
    },
};
use core::{
    collections::{event_queue::EventQueue, game_state::GameState, vector2_int::Vector2Int},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::{ecs_event_reciever, ecs_system::ECSSystemEventless},
};
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;
use std::{panic, vec};

#[global_ecs_system]
pub struct ECSSystemGameRequestManuever {}
impl ECSSystemEventless for ECSSystemGameRequestManuever {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
#[global_ecs_system_event_reciever(GameEvents)]
impl ecs_event_reciever::EventReciever<GameEvents> for ECSSystemGameRequestManuever {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::PlayCard(id, card_instance, data) => {
                // get the stack for the current use case
                let state_card_attribute_modifier_stack = game_state.get_value2::<StateCardAttributeModifierStack>();
                let active_modifiers = CardModifier::flatten(&vec![
                    &state_card_attribute_modifier_stack
                        .get_flat_stack_for_entity(*id)
                        .clone(),
                    &state_card_attribute_modifier_stack
                        .get_flat_stack_for_card(card_instance.instance_id)
                        .clone(),
                ]);

                // reduce energy by cost
                let card_cost = card_instance.get_cost(game_state, *id);
                game_state.edit::<StateEnergy>(|x| {
                    let Some(energy) = x.all_players.get_mut(id) else {
                        return;
                    };

                    energy.0 = energy.0 - card_cost + &active_modifiers.cost;
                });

                let modifiers = card_instance.get_attributes_modifiers(game_state, *id);
                for i in 0..modifiers.len() {
                    let modifiers = &modifiers[i];
                    let data = &data.modifiers[i];
                    match modifiers {
                        CardAttributeModifiers::EditEnergyForEntities(attribute_clear_flag, _, count) => event_queue.enqueue_event(GameEvents::ApplyCardAttributeModifierEnergyForEntities(*attribute_clear_flag, data[0].as_entities(), *count)),
                        CardAttributeModifiers::EditRangeForEntities(attribute_clear_flag, _, count) => event_queue.enqueue_event(GameEvents::ApplyCardAttributeModifierRangeForEntities(*attribute_clear_flag, data[0].as_entities(), *count)),
                        CardAttributeModifiers::EditCostForEntities(attribute_clear_flag, _, count) => event_queue.enqueue_event(GameEvents::ApplyCardAttributeModifierCostForEntities(*attribute_clear_flag, data[0].as_entities(), *count)),
                    }
                }

                let events = card_instance.get_attributes_events(game_state, *id);
                for i in 0..events.len() {
                    let event = &events[i];
                    let data = &data.event[i];
                    match event {
                        CardAttributeEvents::DrawCards(count, _) => event_queue.enqueue_event(GameEvents::ApplyCardAttributeEventDrawCards(data[0].as_players(), *count)),
                        CardAttributeEvents::DiscardCards(_) => event_queue.enqueue_event(GameEvents::ApplyCardAttributeEventDiscardCards(data[0].as_cards())),
                        CardAttributeEvents::MoveEntity(_, _) => event_queue.enqueue_event(GameEvents::ApplyCardAttributeEventMoveEntity(data[0].as_entities(), data[1].as_tiles())),
                        // CardAttributeEvents::MoveBall(_) => event_queue.enqueue_event(GameEvents::ApplyCardAttributeEventMoveBall(*count, state_turn.active_instance_id, card_instance.instance_id)),
                        CardAttributeEvents::MoveBall(_) => event_queue.enqueue_event(GameEvents::ApplyCardAttributeEventMoveBall(data[0].as_tiles(), *id, card_instance.instance_id)),
                        CardAttributeEvents::GainEnergy(count, _) => event_queue.enqueue_event(GameEvents::ApplyCardAttributeEventGainEnergy(data[0].as_entities(), *count)),
                        CardAttributeEvents::RefillEnergy(_) => event_queue.enqueue_event(GameEvents::ApplyCardAttributeEventRefillEnergy(data[0].as_entities())),
                        CardAttributeEvents::SetBallMode(ball_mode) => event_queue.enqueue_event(GameEvents::ApplyCardAttributeEventSetBallMode(ball_mode.clone())),
                    }
                }

                event_queue.enqueue_event(GameEvents::ClearCardAttributeModifiersForFlag(AttributeClearFlag::Play));
            }

            _ => {}
        }
    }
}
