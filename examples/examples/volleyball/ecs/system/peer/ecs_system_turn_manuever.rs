use crate::cards::card_attribute_fillers::attribute_filler_player::CardAttributeFillerPlayer;
use crate::cards::card_attributes_targets::attribute_target_type_tiles::AttributeTargetTypesTiles;
use crate::cards::card_dependencies::data_dep_empty::DataDepsEmpty;
use crate::cards::card_dependencies::data_dep_filled::DataDepsFilled;
use crate::cards::card_dependencies::filled_card_attribute::FilledCardAttribute;
use crate::cards::card_dependencies::filled_card_response::FilledCardResponse;
use crate::cards::card_instance::CardInstance;
use crate::exploration::exploration_path::RoomTypes;
use crate::game_events::GameEvents;
use crate::state::host::state_exploration::StateExploration;
use crate::state::peer::state_peer_input_mode::{InputModes, StatePeerInputMode};
use crate::state::peer::state_peer_select_targets::{SelectStates, StatePeerSelectTargets, WorkingState};
use crate::state::peer::state_peer_selected_card::StatePeerSelectedCards;
use crate::state::state_deck::CardTypes;
use crate::state::state_teams::StateTeamAssignments;
use crate::state::{state_deck::StateDeck, state_turn::StateTurn};
use curio_core::built_in::record::sys_record_input::SysRecordInput;
use curio_core::collections::{event_queue::EventQueue, ledger::Ledger};
use curio_core::extensions::extensions_i32::ExtensionsI32;
use curio_core::network_modes::NetworkModes;
use gameplay::context_3d::Context3D;
use gameplay::traits::habit::Habit;
use gameplay::traits::scope::Scope;
use habit::habit;
use std::sync::Arc;

pub struct ResponseBuilder {
    card_instance: Arc<CardInstance>,
    mods: Vec<AttributeBuilder>,
    evnts: Vec<AttributeBuilder>,
}

impl ResponseBuilder {
    pub fn new(ledger: &Ledger, card: Arc<CardInstance>) -> ResponseBuilder {
        let mut mod_builders = Vec::new();
        for x in card.get_attributes_modifiers(ledger, ledger.instance_id) {
            mod_builders.push(AttributeBuilder::new(x.get_data_dependencies_empty()));
        }
        let mut event_builders = Vec::new();
        for x in card.get_attributes_events(ledger, ledger.instance_id) {
            event_builders.push(AttributeBuilder::new(x.get_data_dependencies_empty()));
        }
        ResponseBuilder {
            card_instance: card.clone(),
            mods: mod_builders,
            evnts: event_builders,
        }
    }
    pub fn update(&mut self, ledger: &mut Ledger) -> bool {
        // the get the id of the user from the gamestate - possible change to pass in
        let user_id = ledger.instance_id;

        // iterate over each modifier
        for x in self.mods.iter_mut() {
            if !x.get_is_full() {
                if !x.update(ledger, &user_id) {
                    // we updated but still didnt complete so full stop
                    return false;
                }
            }
        }
        // iterate over each event
        for x in self.evnts.iter_mut() {
            if !x.get_is_full() {
                if !x.update(ledger, &user_id) {
                    // we updated but still didnt complete so full stop
                    return false;
                }
            }
        }
        // updated and completed
        return true;
    }

    pub fn try_finalize(&mut self) -> Option<(Arc<CardInstance>, FilledCardResponse)> {
        let mut mods = Vec::new();
        let mut events = Vec::new();

        for m in &self.mods {
            mods.push(FilledCardAttribute::new(m.output.clone()));
        }
        for m in &self.evnts {
            events.push(FilledCardAttribute::new(m.output.clone()));
        }
        //
        Some((self.card_instance.clone(), FilledCardResponse::new(mods, events)))
    }
}

pub struct AttributeBuilder {
    reference: Vec<DataDepsEmpty>,
    output: Vec<DataDepsFilled>,
}
impl AttributeBuilder {
    pub fn new(att: Vec<DataDepsEmpty>) -> AttributeBuilder {
        AttributeBuilder { reference: att, output: Vec::new() }
    }
    pub fn get_is_full(&self) -> bool {
        self.reference.len() == self.output.len()
    }
    pub fn update(&mut self, ledger: &mut Ledger, user_id: &i32) -> bool {
        while self.output.len() < self.reference.len() {
            let i = self.output.len();
            // match for the reference
            match self.reference[i] {
                // reference is an entity
                DataDepsEmpty::Entities(t) => match t {
                    // else ignore
                    _ => {}
                },
                // reference is a card
                DataDepsEmpty::Cards(t) => match t {
                    // else ignore
                    _ => {}
                },
                // reference is tiles
                DataDepsEmpty::Tiles(target_type) => match target_type {
                    // if selection wait
                    AttributeTargetTypesTiles::SelectInRangeLocalToBall(_, _) => {
                        // get the cur state
                        let state_select_targets = ledger.read::<StatePeerSelectTargets>();

                        //if we have a selection
                        let Some(selection_state) = &state_select_targets.enabled else {
                            // try to start waiting on a new selection
                            self.try_start(ledger, target_type);
                            return false;
                        };

                        // try to complete - if completed move on to next else return false and try again next frame
                        if !self.try_complete(ledger, selection_state.clone()) {
                            return false;
                        }

                        // complete did succeed - move to next
                        continue;
                    }
                    // if selection wait
                    AttributeTargetTypesTiles::SelectOpponentBackCorner | AttributeTargetTypesTiles::SelectOnTeamUser | AttributeTargetTypesTiles::SelectOnTeamOpponent | AttributeTargetTypesTiles::SelectAny => {
                        // get the cur state
                        let state_select_targets = ledger.read::<StatePeerSelectTargets>();

                        //if we have a selection
                        let Some(selection_state) = &state_select_targets.enabled else {
                            // try to start waiting on a new selection
                            self.try_start(ledger, target_type);
                            return false;
                        };

                        // try to complete - if completed move on to next else return false and try again next frame
                        if !self.try_complete(ledger, selection_state.clone()) {
                            return false;
                        }

                        // complete did succeed - move to next
                        continue;
                    }
                    // else ignore
                    _ => {}
                },
            }

            // fallback fill
            self.output
                .push(CardAttributeFillerPlayer::fill_event(ledger, user_id, &self.reference[i]));
        }

        return true;
    }
    fn try_start(&mut self, ledger: &mut Ledger, t: AttributeTargetTypesTiles) {
        ledger.write::<StatePeerSelectTargets>(|x| {
            x.enabled = Some(SelectStates::Enabled(t, WorkingState::default()));
        });
    }
    fn try_complete(&mut self, ledger: &mut Ledger, selection_state: SelectStates) -> bool {
        match selection_state {
            SelectStates::Completed(filled) => {
                // add to filled
                self.output.push(filled);

                // clear from state
                ledger.write::<StatePeerSelectTargets>(|x| {
                    x.enabled = None;
                });
                // complete did succeed
                return true;
            }
            _ => {
                // complete did fail
                return false;
            }
        }
    }
}
#[habit]
pub struct Instance {
    builder: Option<ResponseBuilder>,
}
impl Instance {}
impl Scope for Instance {
    fn is_enabled(&mut self, ledger: &mut Ledger) -> bool {
        !ledger.read::<StateExploration>().is_selecting_next
    }
    fn run_on_instance(&mut self, _ledger: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Habit for Instance {
    fn tick(&mut self, ledger: &mut Ledger, _: &mut Context3D, event_queue: &mut EventQueue) {
        let team = ledger
            .read::<StateTeamAssignments>()
            .team_for(&ledger.instance_id);
        let Some(team) = team else {
            return;
        };
        let is_turn = ledger.read::<StateTurn>().active_instance_id == team;

        if is_turn
            && ledger.read::<StatePeerSelectTargets>().enabled.is_none()
            && ledger
                .read::<StateExploration>()
                .exploration
                .get_cur_room()
                .room_type
                == RoomTypes::Combat
            && ledger.read::<StatePeerInputMode>().mode == InputModes::Manuever
        {
            let state_input = ledger.read::<SysRecordInput>();
            let state_deck = ledger.read::<StateDeck>();

            let input_card_left = state_input.mapped[0]
                .get_button_or_default("card_left")
                .went_up;
            let input_card_right = state_input.mapped[0]
                .get_button_or_default("card_right")
                .went_up;
            let input_card_submit = state_input.mapped[0]
                .get_button_or_default("card_submit")
                .went_up;

            let state_team = ledger.read::<StateTeamAssignments>();
            let Some(_) = state_team.team_for(&ledger.instance_id) else {
                return;
            };
            // my deck
            let my_deck = &state_deck.deck[&ledger.instance_id];
            let my_cards_in_hand = my_deck.get_cards_from_hand(|x| x.get_manuever_type() != CardTypes::Move);

            // new bounds for looping
            let bounds_min = 0;
            let bounds_max = my_cards_in_hand.len() as i32;

            let input_card_burn = state_input.mapped[0]
                .get_button_or_default("card_burn")
                .went_up;

            if input_card_burn {
                let state_index = ledger.read::<StatePeerSelectedCards>();

                let card = my_cards_in_hand[state_index.index as usize].clone();
                if !card.get_burnable() {
                    return;
                }

                event_queue.enqueue_event(GameEvents::RequestBurnCard(ledger.instance_id, card.instance_id));

                println!("card burned");
            }
            // move left or right
            if input_card_left || input_card_right {
                // edit the selected cards
                ledger.write::<StatePeerSelectedCards>(|x| {
                    // move left
                    if input_card_left {
                        x.index = (x.index - 1).repeat(bounds_min, bounds_max);
                    }

                    // move right
                    if input_card_right {
                        x.index = (x.index + 1).repeat(bounds_min, bounds_max);
                    }
                });
            }

            // edit the selected cards
            ledger.write::<StatePeerSelectedCards>(|x| {
                // incase its out of bounds clamp it
                x.index = x.index.clamp(bounds_min, bounds_max);
            });
            if input_card_submit && self.builder.is_none() {
                let index = ledger.read::<StatePeerSelectedCards>().index;
                // start the builder
                self.builder = Some(ResponseBuilder::new(ledger, my_cards_in_hand[index as usize].clone()))
            }
        }

        let mut did_finalize = false;
        if let Some(builder) = &mut self.builder {
            // update the builder
            if builder.update(ledger) {
                // try finalize - this should pass because the update returned true
                if let Some(card_response) = builder.try_finalize() {
                    let x = card_response.1.clone();
                    println!("sending {}", x.event.len());
                    // finished building run event
                    event_queue.enqueue_event(GameEvents::RequestUseManeuverConsumable(ledger.instance_id, card_response.0.instance_id, card_response.1));
                    // mark as finalized
                    did_finalize = true;
                }
            }
        }
        // if finalized clear the builder
        if did_finalize {
            self.builder = None;
        }
    }
}
