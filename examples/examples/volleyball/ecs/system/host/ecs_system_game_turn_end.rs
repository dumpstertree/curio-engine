use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use system_component_default_gameplay::{
    traits::habit::Habit,
    traits::{impulse::Impulse, scope::Scope},
    world_context::WorldContext,
};

use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};

use crate::{
    cards::{card_event_runner::CardEventRunner, enums::attribute_clear_flag::ModifierClearFlag},
    game_events::GameEvents,
    state::state_position_ball::StatePositionBall,
};

#[derive(Default)]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct ECSSystemGameEndTurn {}
impl Scope for ECSSystemGameEndTurn {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
impl Impulse<GameEvents> for ECSSystemGameEndTurn {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut WorldContext, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::TurnEnd(team) => {
                // end this turn
                println!("Instance: {}. End Turn {}", game_state.instance_id, team);

                let state_position_ball = game_state.get::<StatePositionBall>();
                let ball_is_on_side = team.on_side(state_position_ball.column, state_position_ball.row);
                if ball_is_on_side {
                    println!("Point scored for {}!", team.next_team());
                    event_queue.enqueue_event(GameEvents::PointScored(team.next_team()));
                    return;
                }

                //clear any attributes that end at turn
                let mut runner = CardEventRunner::new();
                runner.enqueue_clear_modifiers(&ModifierClearFlag::Turn);
                runner.post_and_drain(game_state);

                // begin the next player
                event_queue.enqueue_event(GameEvents::TurnBegin(team.next_team()));
                // event_queue.enqueue_event_delayed(GameEvents::TurnBegin(team.next_team()), 1.0);
            }
            _ => {}
        }
    }
}
