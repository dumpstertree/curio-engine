use core::collections::game_state::{self, GameState};

use crate::{
    card_parser::AttributeClearFlag,
    cards::{attribute_target_type_entities::AttribtuteTargetTypesEntities, data_dep_empty::DataDepsEmpty},
    state::{
        state_ball_mode::{BallModes, StateBallMode},
        state_position_ball::StatePositionBall,
        state_position_player::StatePositionPlayer,
    },
};

#[derive(Clone)]
pub enum CardAttributeRequirement {
    BallRangeLessEqual(i32),
    BallRangeGreaterEqual(i32),
    RequireBallMode(BallModes),
    RequireMaxEnergyLessEqual(i32),
    RequireMaxEnergyGreaterEqual(i32),
}

impl CardAttributeRequirement {
    pub fn is_met(&self, game_state: &GameState, user_id: i32) -> bool {
        match self {
            CardAttributeRequirement::BallRangeLessEqual(range) => {
                let ball_loc = game_state.get_value2::<StatePositionBall>();
                // get player loc
                let play_locs = game_state.get_value2::<StatePositionPlayer>();
                let Some(play_loc) = play_locs.positions.get(&user_id) else {
                    return false;
                };

                let distance = (ball_loc.column - play_loc.0).abs() + (ball_loc.row - play_loc.1).abs();
                distance <= *range
            }
            CardAttributeRequirement::BallRangeGreaterEqual(range) => {
                let ball_loc = game_state.get_value2::<StatePositionBall>();
                // get player loc
                let play_locs = game_state.get_value2::<StatePositionPlayer>();
                let Some(play_loc) = play_locs.positions.get(&user_id) else {
                    return false;
                };

                let distance = (ball_loc.column - play_loc.0).abs() + (ball_loc.row - play_loc.1).abs();
                distance >= *range
            }
            CardAttributeRequirement::RequireBallMode(ball_modes) => game_state.get_value2::<StateBallMode>().mode == *ball_modes,
            CardAttributeRequirement::RequireMaxEnergyLessEqual(_) => todo!(),
            CardAttributeRequirement::RequireMaxEnergyGreaterEqual(_) => todo!(),
        }
    }
}
