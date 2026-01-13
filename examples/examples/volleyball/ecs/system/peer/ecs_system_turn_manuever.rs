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
use built_in_state::state_input::InputState;
use core::collections::{event_queue::EventQueue, game_state::GameState};
use core::dumpster_engine::NetworkModes;
use core::extensions::extensions_i32::ExtensionsI32;
use ecs_system::habit;
use std::sync::Arc;
use system_component_default_gameplay::traits::habit::Habit;
use system_component_default_gameplay::traits::scope::Scope;
use system_component_default_gameplay::context_3d::Context3D;

pub struct ResponseBuilder {
    card_instance: Arc<CardInstance>,
    mods: Vec<AttributeBuilder>,
    evnts: Vec<AttributeBuilder>,
}

impl ResponseBuilder {
    pub fn new(game_state: &GameState, card: Arc<CardInstance>) -> ResponseBuilder {
        let mut mod_builders = Vec::new();
        for x in card.get_attributes_modifiers(game_state, game_state.instance_id) {
            mod_builders.push(AttributeBuilder::new(x.get_data_dependencies_empty()));
        }
        let mut event_builders = Vec::new();
        for x in card.get_attributes_events(game_state, game_state.instance_id) {
            event_builders.push(AttributeBuilder::new(x.get_data_dependencies_empty()));
        }
        ResponseBuilder {
            card_instance: card.clone(),
            mods: mod_builders,
            evnts: event_builders,
        }
    }
    pub fn update(&mut self, game_state: &mut GameState) -> bool {
        // the get the id of the user from the gamestate - possible change to pass in
        let user_id = game_state.instance_id;

        // iterate over each modifier
        for x in self.mods.iter_mut() {
            if !x.get_is_full() {
                if !x.update(game_state, &user_id) {
                    // we updated but still didnt complete so full stop
                    return false;
                }
            }
        }
        // iterate over each event
        for x in self.evnts.iter_mut() {
            if !x.get_is_full() {
                if !x.update(game_state, &user_id) {
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
    pub fn update(&mut self, game_state: &mut GameState, user_id: &i32) -> bool {
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
                    AttributeTargetTypesTiles::SelectOnTeamUser | AttributeTargetTypesTiles::SelectOnTeamOpponent | AttributeTargetTypesTiles::SelectAny => {
                        // get the cur state
                        let state_select_targets = game_state.get::<StatePeerSelectTargets>();

                        //if we have a selection
                        let Some(selection_state) = state_select_targets.enabled else {
                            // try to start waiting on a new selection
                            self.try_start(game_state, target_type);
                            return false;
                        };

                        // try to complete - if completed move on to next else return false and try again next frame
                        if !self.try_complete(game_state, selection_state) {
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
                .push(CardAttributeFillerPlayer::fill_event(game_state, user_id, &self.reference[i]));
        }

        return true;
    }
    fn try_start(&mut self, game_state: &mut GameState, t: AttributeTargetTypesTiles) {
        game_state.edit::<StatePeerSelectTargets>(|x| {
            x.enabled = Some(SelectStates::Enabled(t, WorkingState::default()));
        });
    }
    fn try_complete(&mut self, game_state: &mut GameState, selection_state: SelectStates) -> bool {
        match selection_state {
            SelectStates::Completed(filled) => {
                // add to filled
                self.output.push(filled);

                // clear from state
                game_state.edit::<StatePeerSelectTargets>(|x| {
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
    fn is_enabled(&mut self, game_state: &mut GameState) -> bool {
        !game_state.get::<StateExploration>().is_selecting_next
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_peer()
    }
}
impl Habit for Instance {
    fn tick(&mut self, game_state: &mut GameState, _: &mut Context3D, event_queue: &mut EventQueue) {
        let team = game_state
            .get::<StateTeamAssignments>()
            .team_for(&game_state.instance_id);
        let Some(team) = team else {
            return;
        };
        let is_turn = game_state.get::<StateTurn>().active_instance_id == team;

        if is_turn
            && game_state.get::<StatePeerSelectTargets>().enabled.is_none()
            && game_state
                .get::<StateExploration>()
                .exploration
                .get_cur_room()
                .room_type
                == RoomTypes::Combat
            && game_state.get::<StatePeerInputMode>().mode == InputModes::Manuever
        {
            let state_input = game_state.get::<InputState>();
            let state_deck = game_state.get::<StateDeck>();

            let input_card_left = state_input.mapped[0]
                .get_button_or_default("card_left")
                .went_up;
            let input_card_right = state_input.mapped[0]
                .get_button_or_default("card_right")
                .went_up;
            let input_card_submit = state_input.mapped[0]
                .get_button_or_default("card_submit")
                .went_up;

            let state_team = game_state.get::<StateTeamAssignments>();
            let Some(_) = state_team.team_for(&game_state.instance_id) else {
                return;
            };
            // my deck
            let my_deck = &state_deck.deck[&game_state.instance_id];
            let my_cards_in_hand = my_deck.get_cards_from_hand(|x| x.get_manuever_type() != CardTypes::Move);

            // new bounds for looping
            let bounds_min = 0;
            let bounds_max = my_cards_in_hand.len() as i32;
            // let bounds_max = (my_deck.hand_persistent.len() + my_deck.hand_consumable.len()) as i32;

            // move left or right
            if input_card_left || input_card_right {
                // edit the selected cards
                game_state.edit::<StatePeerSelectedCards>(|x| {
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
            game_state.edit::<StatePeerSelectedCards>(|x| {
                // incase its out of bounds clamp it
                x.index = x.index.clamp(bounds_min, bounds_max);
            });
            if input_card_submit && self.builder.is_none() {
                let index = game_state.get::<StatePeerSelectedCards>().index;
                // start the builder
                self.builder = Some(ResponseBuilder::new(game_state, my_cards_in_hand[index as usize].clone()))
            }
        }

        let mut did_finalize = false;
        if let Some(builder) = &mut self.builder {
            // update the builder
            if builder.update(game_state) {
                // try finalize - this should pass because the update returned true
                if let Some(card_response) = builder.try_finalize() {
                    let x = card_response.1.clone();
                    println!("sending {}", x.event.len());
                    // finished building run event
                    event_queue.enqueue_event(GameEvents::RequestUseManeuverConsumable(game_state.instance_id, card_response.0.instance_id, card_response.1));
                    // mark as finalized
                    did_finalize = true;
                }
            }
        }
        // if finalized clear the builder
        if did_finalize {
            self.builder = None;
        }

        // // try to submit card
        // if input_card_submit && self.fill_future.is_none() {
        //     let index = game_state.get::<StatePeerSelectedCards>().index;

        //     let is_met = my_cards_in_hand[index as usize].has_statement(&game_state, game_state.instance_id);
        //     if !is_met {
        //         println!("Requirements not met");
        //         return;
        //     }

        //     println!("try start");

        //     // self.fill_future = Some(Box::pin(Self::fill_all_attributes_async(game_state, my_cards_in_hand[index as usize].clone())));

        //     // let mut evnt_filled = vec![];
        //     // for evnt in &my_cards_in_hand[index as usize].get_attributes_events(&game_state, game_state.instance_id) {
        //     //     evnt_filled.push(FilledCardAttribute::new(CardAttributeFillerPlayer::fill_events(game_state, &game_state.instance_id, &evnt.get_data_dependencies_empty())));
        //     // }
        //     // let mut mod_filled = vec![];
        //     // for evnt in &my_cards_in_hand[index as usize].get_attributes_modifiers(&game_state, game_state.instance_id) {
        //     //     mod_filled.push(FilledCardAttribute::new(CardAttributeFillerPlayer::fill_events(game_state, &game_state.instance_id, &evnt.get_data_dependencies_empty())));
        //     // }

        //     // event_queue.enqueue_event(GameEvents::RequestUseManeuverConsumable(game_state.instance_id, my_cards_in_hand[index as usize].instance_id, FilledCardResponse::new(mod_filled, evnt_filled)));
        //     // }
        // }

        // if self.fill_future.is_some() {
        //     // Poll once per tick
        //     if let Some(fut) = &mut self.fill_future {
        //         let waker = noop_waker_ref();
        //         let mut cx = Context::from_waker(waker);

        //         if let Poll::Ready(result) = fut.as_mut().poll(&mut cx) {
        //             self.fill_future = None;
        //             event_queue.enqueue_event(GameEvents::RequestUseManeuverConsumable(game_state.instance_id, result.0.instance_id, result.1));
        //         }
        //     }
        // }
    }
}

impl Instance {
    async fn fill_all_attributes_async(game_state: GameState, card: Arc<CardInstance>) -> (Arc<CardInstance>, FilledCardResponse) {
        println!("did start");
        // set flag -> true
        // self.is_selecting = true;

        // get value we are populating
        let mut out_mod = Vec::new();
        let mut out_evnt = Vec::new();

        let user_uid = game_state.instance_id;
        // populate modifiers
        let mods = card.get_attributes_modifiers(&game_state, user_uid);
        for evnt in mods {
            let filled = CardAttributeFillerPlayer::fill_events(&game_state, &user_uid, &evnt.get_data_dependencies_empty()).await;
            out_mod.push(FilledCardAttribute::new(filled));
        }
        // // populate events
        for evnt in card.get_attributes_events(&game_state, user_uid) {
            for x in evnt.get_data_dependencies_empty() {
                match x {
                    // DataDepsEmpty::Entities(attribtute_target_types_entities) => todo!(),
                    // DataDepsEmpty::Cards(attribute_target_types_cards) => todo!(),
                    DataDepsEmpty::Tiles(t) => match t {
                        AttributeTargetTypesTiles::SelectAny => todo!(),
                        _ => {}
                    },
                    _ => {}
                }
            }
            let filled = CardAttributeFillerPlayer::fill_events(&game_state, &user_uid, &evnt.get_data_dependencies_empty()).await;
            out_evnt.push(FilledCardAttribute::new(filled));
        }

        // set flag -> false
        // self.is_selecting = false;
        // event_queue.enqueue_event(GameEvents::RequestUseManeuverConsumable(game_state.instance_id, card.instance_id, FilledCardResponse::new(mod_filled, evnt_filled)));

        (card.clone(), FilledCardResponse::new(out_mod, out_evnt))
        // out
    }
}
