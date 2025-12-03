use core::collections::game_state::GameState;

use crate::state::{
    host::state_heat::StateHeat,
    state_ball_mode::{BallModes, StateBallMode},
    state_position_ball::StatePositionBall,
    state_position_player::StatePositionEntities,
};

#[derive(Clone)]
pub enum CardAttributeRequirement {
    BallRangeLessEqual(i32),
    BallRangeGreaterEqual(i32),
    RequireBallMode(BallModes),
    RequireNotBallMode(BallModes),
    RequireMaxEnergyLessEqual(i32),
    RequireMaxEnergyGreaterEqual(i32),
    RequireHeatLessEqual(i32),
    RequireHeatGreaterEqual(i32),
}

impl CardAttributeRequirement {
    pub fn is_met(&self, game_state: &GameState, user_id: i32) -> bool {
        match self {
            CardAttributeRequirement::BallRangeLessEqual(range) => {
                let ball_loc = game_state.get::<StatePositionBall>();
                // get player loc
                let play_locs = game_state.get::<StatePositionEntities>();
                let Some(play_loc) = play_locs.positions.get(&user_id) else {
                    return false;
                };

                let distance = (ball_loc.column - play_loc.0).abs() + (ball_loc.row - play_loc.1).abs();
                distance <= *range
            }
            CardAttributeRequirement::BallRangeGreaterEqual(range) => {
                let ball_loc = game_state.get::<StatePositionBall>();
                // get player loc
                let play_locs = game_state.get::<StatePositionEntities>();
                let Some(play_loc) = play_locs.positions.get(&user_id) else {
                    return false;
                };

                let distance = (ball_loc.column - play_loc.0).abs() + (ball_loc.row - play_loc.1).abs();
                distance >= *range
            }
            CardAttributeRequirement::RequireBallMode(ball_modes) => game_state.get::<StateBallMode>().mode == *ball_modes,
            CardAttributeRequirement::RequireNotBallMode(ball_modes) => game_state.get::<StateBallMode>().mode != *ball_modes,
            CardAttributeRequirement::RequireMaxEnergyLessEqual(_) => todo!(),
            CardAttributeRequirement::RequireMaxEnergyGreaterEqual(_) => todo!(),
            CardAttributeRequirement::RequireHeatLessEqual(count) => {
                game_state
                    .get::<StateHeat>()
                    .all_players
                    .get(&user_id)
                    .unwrap_or(&0)
                    <= count
            }
            CardAttributeRequirement::RequireHeatGreaterEqual(count) => {
                game_state
                    .get::<StateHeat>()
                    .all_players
                    .get(&user_id)
                    .unwrap_or(&0)
                    >= count
            }
        }
    }
}
