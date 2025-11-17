use crate::{card_parser::AttributeClearFlag, cards::card_modifier::CardModifier};
use core::{collections::game_state::StateOwnerships, system::system_game_state::IState};
use macro_state_serialize::global_state_serialize;

#[derive(PartialEq, Eq, Hash)]
#[global_state_serialize]
pub struct StateCardAttributeModifierStack {
    stack: Vec<CardModifier>,
}
impl StateCardAttributeModifierStack {
    pub fn get_flat_stack_for_card(&self, id: i32) -> CardModifier {
        let x: Vec<&CardModifier> = self
            .stack
            .iter()
            .filter(|x| x.applies_to_cards.contains(&id))
            .collect();
        CardModifier::flatten(&x)
    }
    pub fn get_flat_stack_for_entity(&self, id: i32) -> CardModifier {
        let x: Vec<&CardModifier> = self
            .stack
            .iter()
            .filter(|x| x.applies_to_entities.contains(&id))
            .collect();
        CardModifier::flatten(&x)
    }
    pub fn get_flat_stack_for_player(&self, id: i32) -> CardModifier {
        let x: Vec<&CardModifier> = self
            .stack
            .iter()
            .filter(|x| x.applies_to_players.contains(&id))
            .collect();
        CardModifier::flatten(&x)
    }

    pub fn clear_all(&mut self) {
        self.stack.clear();
    }
    pub fn add_to_stack(&mut self, modifier: CardModifier) {
        println!("add from stack {} ", self.stack.len());
        self.stack.push(modifier);
        println!("add from stack {} ", self.stack.len());
    }
    pub fn clear_from_stack(&mut self, clear_flag: AttributeClearFlag) {
        println!("remove from stack {} ", self.stack.len());
        self.stack.retain(|x| x.clear_flag != clear_flag);
        println!("remove from stack {}", self.stack.len());
    }
}
impl IState for StateCardAttributeModifierStack {
    fn id() -> i32 {
        0099
    }
    fn ownership() -> StateOwnerships {
        StateOwnerships::Host
    }
}
