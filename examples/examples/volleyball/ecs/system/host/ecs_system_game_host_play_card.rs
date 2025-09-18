use crate::{
    card_parser::AttributeClearFlag,
    cards::{card_attribute_events::CardAttributeEvents, card_attribute_modifier::CardAttributeModifiers, card_library::CardLibrary, card_modifier::CardModifier, data_dep_filled::DataDepsFilled},
    game_events::GameEvents,
    state::{
        host::state_card_attribute_modifier_stack::{self, StateCardAttributeModifierStack},
        state_ball_mode::{BallModes, StateBallMode},
        state_deck::{CardTypes, StateDeck},
        state_energy::StateEnergy,
        state_position_ball::StatePositionBall,
        state_position_player::StatePositionPlayer,
        state_teams::StateTeamAssignments,
        state_turn::StateTurn,
    },
};
use core::{
    collections::{
        event_queue::EventQueue,
        game_state::{self, GameState},
        vector2_int::Vector2Int,
    },
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::{ecs_event_reciever, ecs_system::ECSSystemEventless},
};
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;
use std::{panic, vec};

#[global_ecs_system]
pub struct ECSSystemGameRequestManuever {}
impl ECSSystemGameRequestManuever {
    fn modify_energy_for_entity(game_state: &mut GameState, clear_flag: &AttributeClearFlag, entity_ids: &Vec<i32>, count: &i32) {
        game_state.edit::<StateCardAttributeModifierStack>(|x| {
            x.add_to_stack(CardModifier {
                clear_flag: *clear_flag,
                applies_to_players: vec![],
                applies_to_entities: entity_ids.clone(),
                applies_to_cards: vec![],
                range: 0,
                cost: 0,
                energy: *count,
            });
        });
    }
    fn modify_range_for_entity(game_state: &mut GameState, clear_flag: &AttributeClearFlag, entity_ids: &Vec<i32>, count: &i32) {
        game_state.edit::<StateCardAttributeModifierStack>(|x| {
            x.add_to_stack(CardModifier {
                clear_flag: *clear_flag,
                applies_to_players: vec![],
                applies_to_entities: entity_ids.clone(),
                applies_to_cards: vec![],
                range: *count,
                cost: 0,
                energy: 0,
            });
        });
    }
    fn modify_cost_for_entity(game_state: &mut GameState, clear_flag: &AttributeClearFlag, entity_ids: &Vec<i32>, count: &i32) {
        game_state.edit::<StateCardAttributeModifierStack>(|x| {
            x.add_to_stack(CardModifier {
                clear_flag: *clear_flag,
                applies_to_players: vec![],
                applies_to_entities: entity_ids.clone(),
                applies_to_cards: vec![],
                range: 0,
                cost: *count,
                energy: 0,
            });
        });
    }
}
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
    fn dequeue_refill_energy(game_state: &mut GameState, entity_ids: Vec<i32>) {
        let state_card_attribute_modifier_stack = game_state.get_value2::<StateCardAttributeModifierStack>();
        game_state.edit::<StateEnergy>(|y| {
            for x in &entity_ids {
                let d = state_card_attribute_modifier_stack.get_flat_stack_for_entity(*x);
                let Some(entity) = y.all_players.get_mut(x) else {
                    continue;
                };

                println!("from {}", entity.0);
                entity.0 = entity.1 + d.energy;
                println!("to {}", entity.0);
            }
        });
    }
    fn dequeue_move_ball_forward(game_state: &mut GameState, modifier_range: i32, move_forward: &i32) {
        let cur_turn = &game_state.get_value2::<StateTurn>().active_instance_id; // get the team for this player
        let team = &game_state
            .get_value2::<StateTeamAssignments>()
            .team_for(cur_turn)
            .unwrap();

        // edit ball position
        game_state.edit::<StatePositionBall>(|x| {
            // convert based on team
            let diff = team.convert_dir(0, move_forward + modifier_range);
            // move
            x.collun = x.collun + diff.0;
            x.row = x.row + diff.1;

            println!("Ball moved for team ({}): ({},{}) -> ({},{})", team, x.collun - diff.0, x.row - diff.1, x.collun, x.row);
        })
    }
    fn dequeue_card_discard(game_state: &mut GameState, card_ids: Vec<i32>) {
        game_state.edit::<StateDeck>(|x| {
            for y in x.deck.iter_mut() {
                for i in (0..y.1.hand_consumable.len()).rev() {
                    let remove = card_ids.contains(&y.1.hand_consumable[i].instance_id);
                    if remove {
                        y.1.pile_discard.push(y.1.hand_consumable[i].clone());
                        y.1.hand_consumable.remove(i);
                    }
                }
            }
        });
    }
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
            GameEvents::PlayCard(id, card_instance, data) => {
                println!("Played Card ({})", card_instance.card_id);

                let modifiers = card_instance.get_attributes_modifiers();
                for i in 0..modifiers.len() {
                    let modifiers = &modifiers[i];
                    let data = &data.modifiers[i];

                    match modifiers {
                        CardAttributeModifiers::EditEnergyForEntities(clear_flag, _, count) => ECSSystemGameRequestManuever::modify_energy_for_entity(game_state, clear_flag, &data[0].as_entities(), count),
                        CardAttributeModifiers::EditRangeForEntities(clear_flag, _, count) => ECSSystemGameRequestManuever::modify_range_for_entity(game_state, clear_flag, &data[0].as_entities(), count),
                        CardAttributeModifiers::EditCostForEntities(clear_flag, _, count) => ECSSystemGameRequestManuever::modify_cost_for_entity(game_state, clear_flag, &data[0].as_entities(), count),
                    }
                }

                let state_card_attribute_modifier_stack = game_state.get_value2::<StateCardAttributeModifierStack>();
                let state_turn = game_state.get_value2::<StateTurn>();
                let active_modifiers = CardModifier::flatten(&vec![
                    &state_card_attribute_modifier_stack
                        .get_flat_stack_for_entity(state_turn.active_instance_id)
                        .clone(),
                    &state_card_attribute_modifier_stack
                        .get_flat_stack_for_card(card_instance.instance_id)
                        .clone(),
                ]);

                println!("modss {}", active_modifiers.range);

                // set the ball state
                game_state.edit::<StateBallMode>(|x| match card_instance.get_manuever_type() {
                    CardTypes::Set => x.mode = BallModes::Set,
                    CardTypes::Bump => x.mode = BallModes::Bump,
                    CardTypes::Spike => x.mode = BallModes::Spike,
                    _ => {}
                });

                // if its a rest card do rest logic
                if card_instance.get_manuever_type() == CardTypes::Rest {
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

                    energy.0 = energy.0 - card_instance.get_cost() + &active_modifiers.cost;
                });

                let events = card_instance.get_attributes_events();
                // enqueue any events from the card data
                for i in 0..events.len() {
                    let event = &events[i];
                    let data = &data.event[i];
                    match event {
                        CardAttributeEvents::DrawCards(count, _) => ECSSystemGameRequestManuever::dequeue_card_draw(game_state, &count, &data[0]),
                        CardAttributeEvents::DiscardCards(_) => ECSSystemGameRequestManuever::dequeue_card_discard(game_state, data[0].as_cards()),
                        CardAttributeEvents::MoveEntity(_, _) => ECSSystemGameRequestManuever::dequeue_move_entity(game_state, &data[0], &data[1]),
                        CardAttributeEvents::MoveBallForward(count) => ECSSystemGameRequestManuever::dequeue_move_ball_forward(game_state, 0, &(count + active_modifiers.range)),
                        CardAttributeEvents::GainEnergy(count, _) => ECSSystemGameRequestManuever::dequeue_gain_energy(game_state, &count, &data[0]),
                        CardAttributeEvents::RefillEnergy(_) => ECSSystemGameRequestManuever::dequeue_refill_energy(game_state, data[0].as_entities()),
                    }
                }

                // clear stack
                game_state.edit::<StateCardAttributeModifierStack>(|x| {
                    x.clear_from_stack(AttributeClearFlag::Play);
                });
            }

            _ => {}
        }
    }
}
