use crate::{
    card_parser::{AttributeClearFlag, CardData, CardParser, DataDepsEmpty, DataDepsFilled},
    game_events::GameEvents,
    state::{
        state_ball_mode::{BallModes, StateBallMode},
        state_deck::{CardLibrary, CardTypes, StateDeck},
        state_energy::StateEnergy,
        state_position_ball::StatePositionBall,
        state_position_player::StatePositionPlayer,
        state_teams::StateTeamAssignments,
        state_turn::StateTurn,
    },
};
use core::{
    collections::{
        event_queue::{self, EventQueue},
        game_state::{self, GameState},
        vector2_int::Vector2Int,
    },
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::{ecs_event_reciever, ecs_system::ECSSystemEventless},
};
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;
use std::panic;
use winit::platform::x11;

#[global_ecs_system]
pub struct ECSSystemGameRequestManuever {}
impl ECSSystemGameRequestManuever {
    fn dequeue_move_entity(game_state: &mut GameState, player_ids: &DataDepsFilled, tile_ids: &DataDepsFilled) {
        let unwraped_player_ids: &Vec<i32>;
        match player_ids {
            DataDepsFilled::Entities(x) => unwraped_player_ids = x,
            _ => panic!(""),
        }
        let unwrapped_tile_ids: &Vec<Vector2Int>;
        match tile_ids {
            DataDepsFilled::Tiles(x) => unwrapped_tile_ids = x,
            _ => panic!(""),
        }

        if unwrapped_tile_ids.len() != 1 {
            panic!("only supports one tile");
        }

        game_state.edit::<StatePositionPlayer>(|y| {
            for x in unwraped_player_ids {
                let Some(position) = y.positions.get_mut(x) else { return };
                position.0 = unwrapped_tile_ids[0].x;
                position.1 = unwrapped_tile_ids[0].y;
            }
        });
    }
    fn dequeue_gain_energy(game_state: &mut GameState, count: &i32, player_ids: &DataDepsFilled) {
        let unwraped_player_ids: &Vec<i32>;
        match player_ids {
            DataDepsFilled::Entities(x) => unwraped_player_ids = x,
            _ => panic!(""),
        }

        game_state.edit::<StateEnergy>(|y| {
            for x in unwraped_player_ids {
                let Some(entity) = y.all_players.get_mut(x) else {
                    continue;
                };

                println!("from {}", entity.0);
                entity.0 = entity.0 + count;
                println!("to {}", entity.0);
            }
        });
    }
    fn dequeue_move_ball_forward(game_state: &mut GameState, card_data: &CardData, move_forward: &i32) {
        let cur_turn = &game_state.get_value2::<StateTurn>().active_instance_id; // get the team for this player
        let team = &game_state
            .get_value2::<StateTeamAssignments>()
            .team_for(cur_turn)
            .unwrap();

        // edit ball position
        game_state.edit::<StatePositionBall>(|x| {
            // convert based on team
            let diff = team.convert_dir(0, move_forward + card_data.range);
            // move
            x.collun = x.collun + diff.0;
            x.row = x.row + diff.1;

            println!("Ball moved for team ({}): ({},{}) -> ({},{})", team, x.collun - diff.0, x.row - diff.1, x.collun, x.row);
        })
    }
    fn dequeue_card_discard() {}
    fn dequeue_card_draw(game_state: &mut GameState, count: &i32, player_ids: &DataDepsFilled) {
        let unwraped_player_ids: &Vec<i32>;
        match player_ids {
            DataDepsFilled::Players(x) => unwraped_player_ids = x,
            _ => panic!(""),
        }

        game_state.edit::<StateDeck>(|y| {
            for x in unwraped_player_ids {
                let Some(deck) = y.deck.get_mut(x) else { return };
                for _ in 0..*count {
                    deck.draw();
                }
            }
        });
    }
}
impl ECSSystemEventless for ECSSystemGameRequestManuever {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
#[global_ecs_system_event_reciever(GameEvents)]
impl ecs_event_reciever::EventReciever<GameEvents> for ECSSystemGameRequestManuever {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::PlayCard(id, card_id, data) => {
                println!("Played Card ({})", card_id);

                // pull card from library
                let library = CardLibrary::new();
                let card = library.get_card(card_id);

                // get the data for the card by this user
                let card_data = CardParser::play_card(game_state, id, card.clone());
                println!("Did Played Card ({})", card_id);

                // set the ball state
                game_state.edit::<StateBallMode>(|x| match card.card_type {
                    CardTypes::Set => x.mode = BallModes::Set,
                    CardTypes::Bump => x.mode = BallModes::Bump,
                    CardTypes::Spike => x.mode = BallModes::Spike,
                    _ => {}
                });

                // if its a rest card do rest logic
                if card.card_type == CardTypes::Rest {
                    game_state.edit::<StateEnergy>(|x| {
                        let Some(energy) = x.all_players.get_mut(id) else {
                            return;
                        };

                        energy.1 = energy.1 - 1;
                        energy.0 = energy.1;
                    });
                }

                // deeduce energy cost
                game_state.edit::<StateEnergy>(|x| {
                    let Some(energy) = x.all_players.get_mut(id) else {
                        return;
                    };

                    energy.0 = energy.0 - card.cost + card_data.cost;
                });

                // enqueue any events from the card data
                for i in 0..card.get_events().len() {
                    println!("len 0: {}", &card.get_events().len());
                    println!("len 1: {}", &data.event.len());
                    let event = &card.get_events()[i];
                    let data = &data.event[i];
                    match event {
                        crate::card_parser::CardEvents::DrawCards(count, _) => ECSSystemGameRequestManuever::dequeue_card_draw(game_state, &count, &data[0]),
                        crate::card_parser::CardEvents::DiscardCards(_) => ECSSystemGameRequestManuever::dequeue_card_discard(),
                        crate::card_parser::CardEvents::MoveEntity(_, _) => ECSSystemGameRequestManuever::dequeue_move_entity(game_state, &data[0], &data[1]),
                        crate::card_parser::CardEvents::MoveBallForward(count) => ECSSystemGameRequestManuever::dequeue_move_ball_forward(game_state, &card_data, &count),
                        crate::card_parser::CardEvents::GainEnergy(count, _) => ECSSystemGameRequestManuever::dequeue_gain_energy(game_state, &count, &data[0]),
                    }
                }
            }

            _ => {}
        }
    }
}
