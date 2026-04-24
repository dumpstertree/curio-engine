use curio_core::collections::ledger::Ledger;

use crate::{
    game_board::{Directions, GameBoard},
    state::{
        host::state_heat::StateHeat,
        state_ball_mode::{BallModes, StateBallMode},
        state_position_ball::StatePositionBall,
        state_position_player::StatePositionEntities,
        state_teams::StateTeamAssignments,
    },
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
    RequireCanMove(Directions),
}

impl CardAttributeRequirement {
    pub fn is_met(&self, ledger: &Ledger, user_id: i32) -> bool {
        match self {
            CardAttributeRequirement::BallRangeLessEqual(range) => {
                let ball_loc = ledger.get::<StatePositionBall>();
                // get player loc
                let play_locs = ledger.get::<StatePositionEntities>();
                let Some(play_loc) = play_locs.positions.get(&user_id) else {
                    return false;
                };

                let distance = (ball_loc.column - play_loc.0).abs() + (ball_loc.row - play_loc.1).abs();
                distance <= *range
            }
            CardAttributeRequirement::BallRangeGreaterEqual(range) => {
                let ball_loc = ledger.get::<StatePositionBall>();
                // get player loc
                let play_locs = ledger.get::<StatePositionEntities>();
                let Some(play_loc) = play_locs.positions.get(&user_id) else {
                    return false;
                };

                let distance = (ball_loc.column - play_loc.0).abs() + (ball_loc.row - play_loc.1).abs();
                distance >= *range
            }
            CardAttributeRequirement::RequireBallMode(ball_modes) => ledger.get::<StateBallMode>().mode == *ball_modes,
            CardAttributeRequirement::RequireNotBallMode(ball_modes) => ledger.get::<StateBallMode>().mode != *ball_modes,
            CardAttributeRequirement::RequireMaxEnergyLessEqual(_) => todo!(),
            CardAttributeRequirement::RequireMaxEnergyGreaterEqual(_) => todo!(),
            CardAttributeRequirement::RequireHeatLessEqual(count) => {
                ledger
                    .get::<StateHeat>()
                    .all_players
                    .get(&user_id)
                    .unwrap_or(&0)
                    <= count
            }
            CardAttributeRequirement::RequireHeatGreaterEqual(count) => {
                ledger
                    .get::<StateHeat>()
                    .all_players
                    .get(&user_id)
                    .unwrap_or(&0)
                    >= count
            }
            CardAttributeRequirement::RequireCanMove(direction) => {
                let state_pos = ledger.get::<StatePositionEntities>();
                let tile = state_pos.positions.get(&user_id).unwrap();
                let team = ledger
                    .get::<StateTeamAssignments>()
                    .team_for(&user_id)
                    .unwrap();
                GameBoard::can_move(&team, tile, direction.clone())
            }
        }
    }
}
