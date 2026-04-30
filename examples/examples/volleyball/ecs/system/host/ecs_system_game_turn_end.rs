use gameplay::{
    context_3d::Context3D,
    traits::{impulse::Impulse, scope::Scope},
};
use impulse::impulse;

use curio_core::{
    collections::{event_queue::EventQueue, ledger::Ledger},
    network_modes::NetworkModes,
};

use crate::{
    cards::{card_event_runner::CardEventRunner, enums::attribute_clear_flag::ModifierClearFlag},
    game_events::GameEvents,
    state::state_position_ball::StatePositionBall,
};

#[derive(Default)]
#[impulse(GameEvents)]
pub struct ECsystemGameEndTurn {}
impl Scope for ECsystemGameEndTurn {
    fn is_enabled(&mut self, _: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut Ledger) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
impl Impulse<GameEvents> for ECsystemGameEndTurn {
    fn dequeue_event(&mut self, ledger: &mut Ledger, _: &mut Context3D, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::TurnEnd(team) => {
                // end this turn
                println!("Instance: {}. End Turn {}", ledger.instance_id, team);

                let state_position_ball = ledger.read::<StatePositionBall>();

                let out_of_bounds = team.is_out_of_bounds(state_position_ball.column, state_position_ball.row);
                let ball_is_on_side = team.is_on_side(state_position_ball.column, state_position_ball.row);

                if ball_is_on_side {
                    if out_of_bounds {
                        println!("Point scored for {}!", *team);
                        event_queue.enqueue_event(GameEvents::PointScored(*team));
                    } else {
                        println!("Point scored for {}!", team.next_team());
                        event_queue.enqueue_event(GameEvents::PointScored(team.next_team()));
                    }

                    return;
                }

                //clear any attributes that end at turn
                let mut runner = CardEventRunner::new();
                runner.enqueue_clear_modifiers(&ModifierClearFlag::Turn);
                runner.post_and_drain(ledger);

                // begin the next player
                event_queue.enqueue_event(GameEvents::TurnBegin(team.next_team()));
                // event_queue.enqueue_event_delayed(GameEvents::TurnBegin(team.next_team()), 1.0);
            }
            _ => {}
        }
    }
}
