use core::{
    collections::{
        event_queue::EventQueue,
        game_state::{self, GameState},
        vector2_int::{self, Vector2Int},
    },
    random::Random,
    system::system_game_state::IState,
};
use mcts::{
    self, CycleBehaviour, Evaluator, GameState as MCTSGameState, MCTS, SearchHandle,
    transposition_table::{ApproxTable, TranspositionHash},
    tree_policy::UCTPolicy,
};
use rand::Fill;
use serde::{Deserialize, Serialize};

use std::{
    clone,
    collections::VecDeque,
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
    marker::PhantomData,
    panic,
    sync::Arc,
    vec,
};

use crate::{
    ai::{
        StateTerminated::StateTerminated,
        dependencies::{data_sources::custom_data_source::CustomDataSource, delegates::custom_delegate::CustomDelegate, hashers::custom_hasher::CustomHasher},
        simulation::Simulation,
    },
    card_parser::AttributeClearFlag,
    cards::{
        card_attributes::{card_attribute_events::CardAttributeEvents, card_attribute_modifier::CardAttributeModifiers},
        card_dependencies::data_dep_filled::DataDepsFilled,
        card_instance::CardInstance,
    },
    event_recievers::{
        event_reciever_apply_card_attribute_event_cards_discard, event_reciever_apply_card_attribute_event_cards_draw, event_reciever_apply_card_attribute_event_cards_energy_edit, event_reciever_apply_card_attribute_event_cards_energy_refill,
        event_reciever_apply_card_attribute_event_move_ball_forward, event_reciever_apply_card_attribute_event_move_entities, event_reciever_apply_card_attribute_event_set_ball_mode, event_reciever_apply_card_attribute_modifier_cost_for_entities,
        event_reciever_apply_card_attribute_modifier_energy_for_entities, event_reciever_apply_card_attribute_modifier_range_for_entities, event_reciever_clear_card_attribute_modifiers_all, event_reciever_clear_card_attribute_modifiers_for_flag,
    },
    game_board::{self, GameBoard},
    game_events::{FilledAttribute, FilledCardResponse, GameEvents},
    state::{
        self,
        host::state_card_attribute_modifier_stack::StateCardAttributeModifierStack,
        state_ball_mode::{self, BallModes, StateBallMode},
        state_deck::StateDeck,
        state_energy::{self, StateEnergy},
        state_position_ball::{self, StatePositionBall},
        state_position_player::StatePositionPlayer,
        state_teams::{self, StateTeamAssignments, Teams},
        state_turn::StateTurn,
    },
};

#[derive(Clone, Debug)]
pub enum Directions {
    Forward,
    Back,
    Left,
    Right,
}
// ----------------- Move Enum -----------------
#[derive(Clone, Debug, Default)]
pub enum Move {
    #[default]
    Invalid,
    Play(Arc<CardInstance>, FilledCardResponse),
    Move(Directions),
    EndTurn,
}
impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Move::Play(card_instance, filled_card_response) => f.write_str(&format!("play card {}", card_instance.card_id)),
            Move::Move(vector2_int) => f.write_str("move"),
            Move::EndTurn => f.write_str("end turn"),
            Move::Invalid => f.write_str("Invalid"),
        }
    }
}

// ----------------- Game State -----------------
pub struct FilledAttributeWithPermutation {
    pub filled: Vec<DataDepsFilledAllPermutations>,
}
impl FilledAttributeWithPermutation {
    pub fn new(filled: Vec<DataDepsFilledAllPermutations>) -> FilledAttributeWithPermutation {
        FilledAttributeWithPermutation { filled }
    }
}

pub struct DataDepsFilledAllPermutations {
    permutations: Vec<DataDepsFilled>,
}
impl DataDepsFilledAllPermutations {
    pub fn new() -> DataDepsFilledAllPermutations {
        DataDepsFilledAllPermutations { permutations: vec![] }
    }
    pub fn add_permutation(&mut self, permutation: DataDepsFilled) {
        self.permutations.push(permutation);
    }
}
pub struct DataDepsFilledForModifiers {
    modifiers_atts: Vec<FilledAttributeWithPermutation>,
    modifiers_events: Vec<FilledAttributeWithPermutation>,
}
impl DataDepsFilledForModifiers {
    pub fn new() -> DataDepsFilledForModifiers {
        DataDepsFilledForModifiers { modifiers_atts: vec![], modifiers_events: vec![] }
    }

    pub fn add_modifier_atts(&mut self, permutation: FilledAttributeWithPermutation) {
        self.modifiers_atts.push(permutation);
    }

    pub fn add_modifier_event(&mut self, permutation: FilledAttributeWithPermutation) {
        self.modifiers_events.push(permutation);
    }
}
impl DataDepsFilledForModifiers {
    pub fn get_data_stack_permutations(&self) -> Vec<FilledCardResponse> {
        // this is incomplete
        let mut output_mods = Vec::new();
        for x in &self.modifiers_atts {
            let mut filled_att = Vec::new();
            for att in &x.filled {
                filled_att.push(att.permutations[0].clone());
            }
            output_mods.push(FilledAttribute::new(filled_att));
        }
        let mut output_events = Vec::new();
        for x in &self.modifiers_events {
            let mut filled_att = Vec::new();
            for att in &x.filled {
                filled_att.push(att.permutations[0].clone());
            }
            output_events.push(FilledAttribute::new(filled_att));
        }

        vec![FilledCardResponse::new(output_mods, output_events)]
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct EventRunner<T, U>
where
    T: Clone + 'static,
    U: Clone + 'static,
{
    recievers: Vec<fn(&T, &mut U) -> Vec<T>>,
    queue: Vec<T>,
}

impl<T, U> EventRunner<T, U>
where
    T: Clone + 'static,
    U: Clone + 'static,
{
    pub fn new(recievers: Vec<fn(&T, &mut U) -> Vec<T>>) -> Self {
        Self { recievers, queue: Vec::new() }
    }

    pub fn enqueue(&mut self, event: &T) {
        self.queue.push(event.clone());
    }

    pub fn post_and_drain(&mut self, data: &mut U) {
        while let Some(event) = self.queue.pop() {
            for func in &self.recievers {
                let new_events = func(&event, data);
                // Optionally enqueue new events generated by handlers
                for new_event in new_events {
                    self.queue.push(new_event);
                }
            }
        }
    }
}
#[derive(Clone)]
pub enum CardEvents {
    // modifier
    ApplyModifierEnergyForEntities(DataDepsFilled, AttributeClearFlag, i32),
    ApplyModifierCostForEntities(DataDepsFilled, AttributeClearFlag, i32),
    ApplyModifierRangeForEntities(DataDepsFilled, AttributeClearFlag, i32),
    // events
    ApplyEventRefillEnergy(DataDepsFilled),
    ApplyEventEditEnergy(DataDepsFilled, i32),
    ApplyEventMoveEntity(DataDepsFilled, DataDepsFilled),
    ApplyEventDrawCards(DataDepsFilled, i32),
    ApplyEventDiscardCards(DataDepsFilled),
    ApplyEventSetBallMode(BallModes),
    /// i32:EntityID, i32:CardID, Vec<i32>:TargetTileIDs
    ApplyEventMoveBall(DataDepsFilled),

    // clear
    ClearModifiersForFlag(AttributeClearFlag),
    ClearModifiersAll(),
}
#[derive(Clone)]
/// Runs events specific to cards. This is broken out so it can be used with any gamestate seamlessly
pub struct CardEventRunner {
    runner: EventRunner<CardEvents, GameState>,
}

impl CardEventRunner {
    pub fn new() -> CardEventRunner {
        // create the list of all the recievers
        let recievers: Vec<fn(&CardEvents, &mut GameState) -> Vec<CardEvents>> = vec![
            event_reciever_apply_card_attribute_event_cards_discard::EventReciever::recieve,
            event_reciever_apply_card_attribute_event_cards_draw::EventReciever::recieve,
            event_reciever_apply_card_attribute_event_cards_energy_edit::EventReciever::recieve,
            event_reciever_apply_card_attribute_event_cards_energy_refill::EventReciever::recieve,
            event_reciever_apply_card_attribute_event_move_ball_forward::EventReciever::recieve,
            event_reciever_apply_card_attribute_event_move_entities::EventReciever::recieve,
            event_reciever_apply_card_attribute_event_set_ball_mode::EventReciever::recieve,
            event_reciever_apply_card_attribute_modifier_cost_for_entities::EventReciever::recieve,
            event_reciever_apply_card_attribute_modifier_energy_for_entities::EventReciever::recieve,
            event_reciever_apply_card_attribute_modifier_range_for_entities::EventReciever::recieve,
            event_reciever_clear_card_attribute_modifiers_all::EventReciever::recieve,
            event_reciever_clear_card_attribute_modifiers_for_flag::EventReciever::recieve,
        ];

        // create the instance
        CardEventRunner { runner: EventRunner::new(recievers) }
    }
    pub fn enqueue_modifier(&mut self, event: &CardAttributeModifiers, data: &FilledAttribute) {
        match event {
            CardAttributeModifiers::EditCostForEntities(attribute_clear_flag, _, count) => self
                .runner
                .enqueue(&CardEvents::ApplyModifierCostForEntities(
                    data.filled[0].clone(),
                    attribute_clear_flag.clone(), //
                    *count,
                )),
            CardAttributeModifiers::EditEnergyForEntities(attribute_clear_flag, _, count) => self
                .runner
                .enqueue(&CardEvents::ApplyModifierEnergyForEntities(
                    data.filled[0].clone(),
                    attribute_clear_flag.clone(), //
                    *count,
                )),
            CardAttributeModifiers::EditRangeForEntities(attribute_clear_flag, _, count) => self
                .runner
                .enqueue(&CardEvents::ApplyModifierRangeForEntities(
                    data.filled[0].clone(),
                    attribute_clear_flag.clone(), //
                    *count,
                )),
        }
    }
    pub fn enqueue_event(&mut self, event: &CardAttributeEvents, data: &FilledAttribute) {
        match event {
            CardAttributeEvents::DiscardCards(_) => {
                self.runner
                    .enqueue(&CardEvents::ApplyEventDiscardCards(data.filled[0].clone()));
            }
            CardAttributeEvents::DrawCards(count, _targeting) => {
                self.runner
                    .enqueue(&CardEvents::ApplyEventDrawCards(data.filled[0].clone(), *count));
            }
            CardAttributeEvents::GainEnergy(count, _targeting) => {
                self.runner
                    .enqueue(&CardEvents::ApplyEventEditEnergy(data.filled[0].clone(), *count));
            }
            CardAttributeEvents::RefillEnergy(_) => {
                self.runner
                    .enqueue(&&CardEvents::ApplyEventRefillEnergy(data.filled[0].clone()));
            }
            CardAttributeEvents::MoveBall(_) => {
                self.runner
                    .enqueue(&CardEvents::ApplyEventMoveBall(0, 0, data.filled[0].clone()));
            }
            CardAttributeEvents::MoveEntity(_, _) => {
                self.runner
                    .enqueue(&CardEvents::ApplyEventMoveEntity(data.filled[0].clone(), data.filled[1].clone()));
            }
            CardAttributeEvents::SetBallMode(mode) => {
                self.runner
                    .enqueue(&CardEvents::ApplyEventSetBallMode(mode.clone()));
            }

            _ => {}
        }
    }
    pub fn enqueue_clear_modifiers(&mut self, flag: &AttributeClearFlag) {
        self.runner
            .enqueue(&CardEvents::ClearModifiersForFlag(*flag));
    }
    pub fn post_and_drain(&mut self, game_state: &mut GameState) {
        self.runner.post_and_drain(game_state);
    }
}
