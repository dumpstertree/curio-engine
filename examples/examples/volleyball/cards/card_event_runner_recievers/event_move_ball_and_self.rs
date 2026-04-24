use crate::{
    cards::enums::card_events::CardEvents,
    game_board::GameBoard,
    state::{
        state_position_ball::StatePositionBall,
        state_position_player::StatePositionEntities,
        state_teams::{StateTeamAssignments, Teams},
    },
};
use curio_core::{Vector2Int, collections::ledger::Ledger};

pub struct EventReciever {}
impl EventReciever {
    pub fn recieve(event: &CardEvents, ledger: &mut Ledger) -> Vec<CardEvents> {
        match event {
            CardEvents::EventMoveBallAndEntity(wrapped_tiles, wrapped_entites) => {
                //
                let tile_ids = wrapped_tiles.as_tiles();
                if tile_ids.len() == 0 {
                    println!("Does not support len of 0 tile");
                    return vec![];
                }
                let entity_ids = wrapped_entites.as_entities();
                if entity_ids.len() == 0 {
                    println!("Does not support len of 0 tile");
                    return vec![];
                }
                //
                if tile_ids.len() > 1 {
                    println!("Does not support more than 1 tile");
                    return vec![];
                }
                if entity_ids.len() > 1 {
                    println!("Does not support more than 1 tile");
                    return vec![];
                }

                let Some(team) = ledger
                    .get::<StateTeamAssignments>()
                    .team_for(&entity_ids[0])
                else {
                    return vec![];
                };

                let min: Vector2Int;
                let max: Vector2Int;
                match team {
                    Teams::Red => {
                        let cmin = GameBoard::get_bounds_min();
                        let cmax = GameBoard::get_bounds_max();
                        let tmax = GameBoard::get_bounds_max_for_team(&Teams::Red);
                        min = Vector2Int::new(cmin.x, cmin.y);
                        max = Vector2Int::new(cmax.x, tmax.y);
                    }
                    Teams::Blue => {
                        let cmin = GameBoard::get_bounds_min();
                        let cmax = GameBoard::get_bounds_max();
                        let tmin = GameBoard::get_bounds_min_for_team(&Teams::Blue);
                        min = Vector2Int::new(cmin.x, cmin.y);
                        max = Vector2Int::new(tmin.y, cmax.x);
                    }
                }

                // note: all distance modifiers shoule be applied at the targeting phase

                let pos_x = tile_ids[0].x.clamp(min.x, max.x);
                let pos_y = tile_ids[0].y.clamp(min.y, max.y);
                // edit ball position
                ledger.edit::<StatePositionBall>(|x| {
                    // x.column = tile_ids[0].x;
                    // x.row = tile_ids[0].y;
                    x.column = pos_x;
                    x.row = pos_y;
                });
                ledger.edit::<StatePositionEntities>(|x| {
                    //
                    if let Some(pos) = x.positions.get_mut(&entity_ids[0]) {
                        pos.0 = pos_x;
                        pos.1 = pos_y;
                    }
                });
            }
            _ => {}
        }
        vec![]
    }
}
