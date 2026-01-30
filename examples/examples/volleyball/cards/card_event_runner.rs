use curio_core::collections::{event_runner::EventRunner, game_state::GameState};
use std::vec;

use crate::cards::{
    card_attributes::{card_attribute_events::CardAttributeEvents, card_attribute_modifier::CardAttributeModifiers},
    card_dependencies::filled_card_attribute::FilledCardAttribute,
    card_event_runner_recievers::{
        clear_modifier_all, clear_modifier_for_flag, event_card_discard, event_card_draw, event_change_ball_mode, event_energy_edit, event_energy_fill, event_heat_drain, event_move_ball, event_move_ball_and_self, event_move_entities, modifier_cost_for_entities, modifier_energy_for_entities,
        modifier_range_for_entities,
    },
    enums::{attribute_clear_flag::ModifierClearFlag, card_events::CardEvents},
};

#[derive(Clone)]
/// Runs events specific to cards. This is broken out so it can be used with any gamestate seamlessly
pub struct CardEventRunner {
    runner: EventRunner<CardEvents, GameState>,
}
impl CardEventRunner {
    pub fn new() -> CardEventRunner {
        // create the list of all the recievers
        let recievers: Vec<fn(&CardEvents, &mut GameState) -> Vec<CardEvents>> = vec![
            event_heat_drain::EventReciever::recieve,
            event_card_discard::EventReciever::recieve,
            event_card_draw::EventReciever::recieve,
            event_energy_edit::EventReciever::recieve,
            event_energy_fill::EventReciever::recieve,
            event_move_ball::EventReciever::recieve,
            event_move_ball_and_self::EventReciever::recieve,
            event_move_entities::EventReciever::recieve,
            event_change_ball_mode::EventReciever::recieve,
            modifier_cost_for_entities::EventReciever::recieve,
            modifier_energy_for_entities::EventReciever::recieve,
            modifier_range_for_entities::EventReciever::recieve,
            clear_modifier_all::EventReciever::recieve,
            clear_modifier_for_flag::EventReciever::recieve,
        ];

        // create the instance
        CardEventRunner { runner: EventRunner::new(recievers) }
    }
    pub fn enqueue_modifier(&mut self, event: &CardAttributeModifiers, data: &FilledCardAttribute) {
        match event {
            CardAttributeModifiers::EditCostForEntities(attribute_clear_flag, _, count) => self.runner.enqueue(&CardEvents::ModifierCostForEntities(
                data.filled[0].clone(),
                attribute_clear_flag.clone(), //
                *count,
            )),
            CardAttributeModifiers::EditEnergyForEntities(attribute_clear_flag, _, count) => self.runner.enqueue(&CardEvents::ModifierEnergyForEntities(
                data.filled[0].clone(),
                attribute_clear_flag.clone(), //
                *count,
            )),
            CardAttributeModifiers::EditRangeForEntities(attribute_clear_flag, _, count) => self.runner.enqueue(&CardEvents::ModifierRangeForEntities(
                data.filled[0].clone(),
                attribute_clear_flag.clone(), //
                *count,
            )),
        }
    }
    pub fn enqueue_event(&mut self, event: &CardAttributeEvents, data: &FilledCardAttribute) {
        match event {
            CardAttributeEvents::DiscardCards(_) => {
                self.runner
                    .enqueue(&CardEvents::EventCardDiscard(data.filled[0].clone()));
            }
            CardAttributeEvents::DrawCards(count, _targeting) => {
                self.runner
                    .enqueue(&CardEvents::EventCardDraw(data.filled[0].clone(), *count));
            }
            CardAttributeEvents::GainEnergy(count, _targeting) => {
                self.runner
                    .enqueue(&CardEvents::EventEnergyEdit(data.filled[0].clone(), *count));
            }
            CardAttributeEvents::RefillEnergy(_) => {
                self.runner
                    .enqueue(&&CardEvents::EventEnergyFill(data.filled[0].clone()));
            }
            CardAttributeEvents::MoveBall(_) => {
                self.runner
                    .enqueue(&CardEvents::EventMoveBall(data.filled[0].clone()));
            }
            CardAttributeEvents::MoveBallAndEntity(_, _) => {
                self.runner
                    .enqueue(&CardEvents::EventMoveBallAndEntity(data.filled[0].clone(), data.filled[1].clone()));
            }
            CardAttributeEvents::MoveEntity(_, _) => {
                self.runner
                    .enqueue(&CardEvents::EventMoveEntities(data.filled[0].clone(), data.filled[1].clone()));
            }
            CardAttributeEvents::SetBallMode(mode) => {
                self.runner
                    .enqueue(&CardEvents::EventChangeBallMode(mode.clone()));
            }
            CardAttributeEvents::DrainHeat(_mode) => {
                self.runner
                    .enqueue(&&CardEvents::EventHeatDrain(data.filled[0].clone()));
            }
        }
    }
    pub fn enqueue_clear_modifiers(&mut self, flag: &ModifierClearFlag) {
        self.runner
            .enqueue(&CardEvents::ClearModifiersForFlag(*flag));
    }
    pub fn post_and_drain(&mut self, game_state: &mut GameState) {
        self.runner.post_and_drain(game_state);
    }
}
