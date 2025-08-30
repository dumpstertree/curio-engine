use built_in_state::state_camera::CameraState;
use ecs_system::global_ecs_system;
use hecs::World;

use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};

use crate::{
    game_events::GameEvents,
    state::{
        state_ball_mode::{BallModes, StateBallMode},
        state_deck::{Card, CardTypes, StateDeck},
        state_energy::StateEnergy,
        state_position_ball::StatePositionBall,
        state_position_player::StatePositionPlayer,
        state_turn::StateTurn,
    },
};

#[global_ecs_system]
pub struct ECSSystemGameStart {}
impl ECSSystemEventless for ECSSystemGameStart {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn init(&mut self, game_state: &mut GameState, _: &mut World, events: &mut EventQueue, _: &mut core::io::asset_loader::AssetLoader) {
        // set resolution
        game_state.edit::<StateEnergy>(|x| {
            x.cur_energy = 5;
            x.max_energy = 5;
        });
        game_state.edit::<StateDeck>(|x| {
            // add all cards
            x.deck.hand_persistent = vec![Card::new("Rest", CardTypes::Rest, 0), Card::new("Move", CardTypes::Move, 0)];
            x.deck.pile_draw = vec![
                Card::new("Bump 0", CardTypes::Bump, 1),
                Card::new("Bump 1", CardTypes::Bump, 1),
                Card::new("Bump 2", CardTypes::Bump, 1),
                Card::new("Bump 3", CardTypes::Bump, 1),
                Card::new("Bump 4", CardTypes::Bump, 1),
                Card::new("Set 0", CardTypes::Set, 1),
                Card::new("Set 1", CardTypes::Set, 1),
                Card::new("Set 2", CardTypes::Set, 1),
                Card::new("Set 3", CardTypes::Set, 1),
                Card::new("Set 4", CardTypes::Set, 1),
                Card::new("Spike 0", CardTypes::Spike, 3),
                Card::new("Spike 1", CardTypes::Spike, 3),
                Card::new("Spike 2", CardTypes::Spike, 3),
                Card::new("Spike 3", CardTypes::Spike, 3),
                Card::new("Spike 4", CardTypes::Spike, 3),
            ];

            // shuffle the deck
            x.deck.reshuffle();
            x.deck.draw();
        });
        // set resolution
        game_state.edit::<CameraState>(|x| {
            x.resolution_width = 1920 / 1;
            x.resolution_height = 1080 / 1;
        });

        // setup player positions
        game_state.edit::<StatePositionPlayer>(|x| {
            x.row = 0;
            x.collun = 0;
        });

        // setup ball position
        game_state.edit::<StatePositionBall>(|x| {
            x.row = 0;
            x.collun = 0;
        });

        // setup turns
        game_state.edit::<StateTurn>(|x| x.active_instance_id = 0);

        // setup ball
        game_state.edit::<StateBallMode>(|x| x.mode = BallModes::Serve);

        // start the game
        events.enqueue_event(GameEvents::TurnBegin(0));
    }
}
