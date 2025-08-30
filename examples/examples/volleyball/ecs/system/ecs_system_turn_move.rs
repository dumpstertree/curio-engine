use built_in_state::state_input::InputState;
use ecs_system::global_ecs_system;
use hecs::World;

use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};

use crate::state::{state_energy::StateEnergy, state_position_player::StatePositionPlayer, state_turn::StateTurn};

#[global_ecs_system]
pub struct ECSSystemTurnMove {}
impl ECSSystemEventless for ECSSystemTurnMove {
    fn is_enabled(&mut self, game_state: &mut GameState, _: &mut World) -> bool {
        game_state.get_value2::<StateTurn>().active_instance_id == game_state.instance_id
    }
    fn tick(&mut self, game_state: &mut GameState, _: &mut World, _: &mut EventQueue) {
        // get states
        let state_energy = game_state.get_value2::<StateEnergy>();
        let state_input = game_state.get_value2::<InputState>();

        let move_forward = state_input.mapped[0]
            .get_button_or_default("move_forward")
            .went_up;
        let move_back: bool = state_input.mapped[0]
            .get_button_or_default("move_back")
            .went_up;
        let move_left = state_input.mapped[0]
            .get_button_or_default("move_left")
            .went_up;
        let move_right: bool = state_input.mapped[0]
            .get_button_or_default("move_right")
            .went_up;

        // if any movement detected
        if move_forward || move_back || move_left || move_right {
            // check to make sure we have enough energy
            let has_energy = state_energy.cur_energy > 0;
            if !has_energy {
                println!("Can't Move. No Energy");
                return;
            }

            // edit the position based on input
            game_state.edit::<StatePositionPlayer>(|x| {
                if move_forward {
                    x.row += 1;
                    println!("Move Forward. New Pos x:{}, y:{}", x.row, x.collun);
                }

                if move_back {
                    x.row -= 1;
                    println!("Move Back. New Pos x:{}, y:{}", x.row, x.collun);
                }

                if move_left {
                    x.collun -= 1;
                    println!("Move Left. New Pos x:{}, y:{}", x.row, x.collun);
                }

                if move_right {
                    x.row += 1;
                    println!("Move Right. New Pos x:{}, y:{}", x.row, x.collun);
                }
            });

            // edit the energy
            game_state.edit::<StateEnergy>(|x| {
                x.cur_energy -= 1;
            });
        }
    }
}
