use crate::{
    cards::{card_event_runner::CardEventRunner, enums::attribute_clear_flag::ModifierClearFlag},
    game_events::GameEvents,
    state::{host::state_play_history::StatePlayHistory, state_deck::CardTypes, state_position_ball::StatePositionBall, state_position_player::StatePositionEntities},
};
use curio_core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use gameplay::{
    context_3d::Context3D,
    traits::{impulse::Impulse, scope::Scope},
};
use impulse::impulse;

#[derive(Default)]
#[impulse(GameEvents)]
pub struct ECSSystemGameRequestManuever {}

impl Scope for ECSSystemGameRequestManuever {
    fn is_enabled(&mut self, _game_state: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _game_state: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_host()
    }
}
impl Impulse<GameEvents> for ECSSystemGameRequestManuever {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut Context3D, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::PlayCard(id, card_instance, data) => {
                // creates an event runner to all the events on
                let mut event_runner = CardEventRunner::new();

                let was_state_ball_pos = game_state.get::<StatePositionBall>();

                // get the attributes out of this card
                let atts_mods = card_instance.get_attributes_modifiers(&game_state, *id);
                let atts_evnt = card_instance.get_attributes_events(&game_state, *id);

                println!("PLAYED CARD: {}", card_instance.card_id);
                // iterate over each mod and add it and its data to the runner
                for i in 0..atts_mods.len() {
                    event_runner.enqueue_modifier(&atts_mods[i], &data.modifiers[i]);
                }

                // iterate over each event and add it and its data to the runner
                for i in 0..atts_evnt.len() {
                    event_runner.enqueue_event(&atts_evnt[i], &data.event[i]);
                }

                // enqueue the clear flag
                event_runner.enqueue_clear_modifiers(&ModifierClearFlag::Play);

                // run all inside runner
                event_runner.post_and_drain(game_state);

                // add play to history
                game_state.edit::<StatePlayHistory>(|x| {
                    x.history.push((*id, card_instance.clone(), data.clone()));
                });

                let t = card_instance.get_manuever_type();
                match t {
                    CardTypes::Bump => {
                        let state_pos_ball = game_state.get::<StatePositionBall>();
                        let delta_x = state_pos_ball.column - was_state_ball_pos.column;
                        let delta_y = state_pos_ball.row - was_state_ball_pos.row;
                        game_state.edit::<StatePositionEntities>(|x| {
                            if let Some(pos) = x.positions.get_mut(id) {
                                pos.0 = pos.0 - delta_x.clamp(-1, 1);
                                pos.1 = pos.1 - delta_y.clamp(-1, 1);
                            }
                        });
                    }
                    CardTypes::Spike => {
                        let state_pos_ball = game_state.get::<StatePositionBall>();
                        let delta_x = state_pos_ball.column - was_state_ball_pos.column;
                        let delta_y = state_pos_ball.row - was_state_ball_pos.row;
                        game_state.edit::<StatePositionEntities>(|x| {
                            if let Some(pos) = x.positions.get_mut(id) {
                                pos.0 = pos.0 + delta_x.clamp(-1, 1);
                                pos.1 = pos.1 + delta_y.clamp(-1, 1);
                            }
                        });
                    }
                    _ => {}
                }
                // send event that we did play the card
                event_queue.enqueue_event(GameEvents::DidPlayCard(*id, card_instance.clone(), data.clone()));
            }

            _ => {}
        }
    }
}
