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
        attribute_target_type_entities::AttribtuteTargetTypesEntities, attribute_target_type_tiles::AttributeTargetTypesTiles, card_attribute_events::CardAttributeEvents, card_attribute_modifier::CardAttributeModifiers, card_instance::CardInstance, card_modifier::CardModifier,
        data_dep_empty::DataDepsEmpty, data_dep_filled::DataDepsFilled,
    },
    event_recievers::{
        event_reciever_apply_card_attribute_event_cards_draw, event_reciever_apply_card_attribute_event_cards_energy_edit, event_reciever_apply_card_attribute_event_move_ball_forward, event_reciever_apply_card_attribute_event_set_ball_mode,
        event_reciever_apply_card_attribute_modifier_cost_for_entities, event_reciever_apply_card_attribute_modifier_energy_for_entities, event_reciever_apply_card_attribute_modifier_range_for_entities,
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
    ApplyModifierEnergyForEntities(AttributeClearFlag, Vec<i32>, i32),
    ApplyModifierCostForEntities(AttributeClearFlag, Vec<i32>, i32),
    ApplyModifierRangeForEntities(AttributeClearFlag, Vec<i32>, i32),
    // events
    ApplyEventRefillEnergy,
    ApplyEventGainEnergy(DataDepsFilled, i32),
    ApplyEventMoveEntity,
    ApplyEventDrawCards(DataDepsFilled, i32),
    ApplyEventDiscardCards(Vec<i32>, i32),
    ApplyEventSetBallMode(BallModes),
    /// i32:EntityID, i32:CardID, Vec<i32>:TargetTileIDs
    ApplyEventMoveBall(i32, i32, DataDepsFilled),
}
#[derive(Clone)]
pub struct CardEventRunner {
    runner: EventRunner<CardEvents, GameState>,
}

impl CardEventRunner {
    pub fn new() -> CardEventRunner {
        // create the list of all the recievers
        let recievers: Vec<fn(&CardEvents, &mut GameState) -> Vec<CardEvents>> = vec![
            // modifiers
            event_reciever_apply_card_attribute_modifier_energy_for_entities::EventReciever::recieve,
            event_reciever_apply_card_attribute_modifier_cost_for_entities::EventReciever::recieve,
            event_reciever_apply_card_attribute_modifier_range_for_entities::EventReciever::recieve,
            // events
            event_reciever_apply_card_attribute_event_move_ball_forward::EventReciever::recieve,
            event_reciever_apply_card_attribute_event_set_ball_mode::EventReciever::recieve,
            event_reciever_apply_card_attribute_event_cards_draw::EventReciever::recieve,
            event_reciever_apply_card_attribute_event_cards_energy_edit::EventReciever::recieve,
        ];

        // create the instance
        CardEventRunner { runner: EventRunner::new(recievers) }
    }
    pub fn enqueue_modifier(&mut self, event: &CardAttributeModifiers) {
        match event {
            CardAttributeModifiers::EditEnergyForEntities(attribute_clear_flag, _, count) => self
                .runner
                .enqueue(&CardEvents::ApplyModifierEnergyForEntities(
                    attribute_clear_flag.clone(), //
                    vec![],
                    *count,
                )),
            CardAttributeModifiers::EditRangeForEntities(attribute_clear_flag, _, count) => self
                .runner
                .enqueue(&CardEvents::ApplyModifierRangeForEntities(
                    attribute_clear_flag.clone(), //
                    vec![],
                    *count,
                )),
            CardAttributeModifiers::EditCostForEntities(attribute_clear_flag, _, count) => self
                .runner
                .enqueue(&CardEvents::ApplyModifierCostForEntities(
                    attribute_clear_flag.clone(), //
                    vec![],
                    *count,
                )),
        }
    }
    pub fn enqueue_event(&mut self, event: &CardAttributeEvents, data: &FilledAttribute) {
        match event {
            // CardAttributeEvents::DiscardCards(_) => {
            //     self.runner.enqueue(&CardEvents::ApplyEventDiscardCards());
            // }
            CardAttributeEvents::DrawCards(count, _targeting) => {
                self.runner
                    .enqueue(&CardEvents::ApplyEventDrawCards(data.filled[0].clone(), *count));
            }
            CardAttributeEvents::GainEnergy(count, _targeting) => {
                self.runner
                    .enqueue(&CardEvents::ApplyEventGainEnergy(data.filled[0].clone(), *count));
            }
            CardAttributeEvents::SetBallMode(mode) => {
                self.runner
                    .enqueue(&CardEvents::ApplyEventSetBallMode(mode.clone()));
            }
            CardAttributeEvents::MoveBall(_) => {
                self.runner
                    .enqueue(&CardEvents::ApplyEventMoveBall(0, 0, data.filled[0].clone()));
            }
            _ => {}
        }
    }
    pub fn post_and_drain(&mut self, game_state: &mut GameState) {
        self.runner.post_and_drain(game_state);
    }
}
