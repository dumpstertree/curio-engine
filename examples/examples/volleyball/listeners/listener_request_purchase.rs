use crate::game_events::GameEvents;
use crate::listeners::listener_ui_set_mode::UITypes;
use crate::state::host::state_currency::{self, StateCurrency};
use crate::state::host::state_deck_exploration::StateDeckExploration;
use crate::state::host::state_heat::StateHeat;
use crate::state::host::state_shop::{StateShop, StockItems};
use crate::state::peer::state_peer_entity_ids::{EntityIDTypes, StateEntityIDs};
use crate::state::state_deck::StateDeck;
use crate::state::state_score::StateScore;
use crate::state::state_teams::StateTeamAssignments;
use built_in_state::state_debug::StateDebug;
use core::collections::game_state;
use core::gameplay::ecs::traits::ecs_event_reciever::{self, InstanceLimiter};
use core::gameplay::world_context::WorldContext;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use ecs_event::global_ecs_system_event_reciever;
use hecs::World;

#[derive(Default)]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct Listener {}

impl InstanceLimiter for Listener {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        NetworkModes::all_host()
    }
}
impl ecs_event_reciever::EventReciever<GameEvents> for Listener {
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut WorldContext, event_queue: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::RequestPurchase(user_id, instance_id) => {
                println!("purchase requested");
                let state_shop = game_state.get::<StateShop>();
                let state_currency = game_state.get::<StateCurrency>();

                let mut matching_stock = None;

                for stock in &state_shop.shop.stock {
                    // not matching
                    if &stock.instance_id != instance_id {
                        continue;
                    }
                    matching_stock = Some(stock.clone());
                }

                let Some(matching_stock) = matching_stock else {
                    println!("No matching item in stock for instance_id: {}", instance_id);
                    return;
                };

                // no count
                if !matching_stock.count >= 1 {
                    println!("Item count is <= 0 for instance_id: {}", instance_id);
                    return;
                }

                // not enough money
                if !state_currency.currency >= matching_stock.count {
                    println!("Not enought currency for instance_id: {}", instance_id);
                    return;
                }

                // add item
                match matching_stock.item {
                    StockItems::Card(card_uid) => {
                        game_state.edit::<StateDeckExploration>(|x| {
                            // get deck for user id
                            let Some(deck) = x.deck.get_mut(user_id) else {
                                println!("Deck not found for user_id: {}", user_id);
                                return;
                            };
                            // add card to deck
                            deck.add_card_to_deck(&card_uid, false);
                        });

                        //
                    }
                    StockItems::Relic(_) => {
                        todo!("Relics not yet supported");
                    }
                }
                // reduce currency
                game_state.edit::<StateCurrency>(|x| {
                    x.currency -= matching_stock.cost;
                });

                // reduce inventory
                game_state.edit::<StateShop>(|x| {
                    for stock in x.shop.stock.iter_mut() {
                        // not matching
                        if &stock.instance_id != instance_id {
                            continue;
                        }
                        stock.count = stock.count - 1;
                    }
                });

                println!("purchas success");
            }

            _ => {}
        }
    }
}
