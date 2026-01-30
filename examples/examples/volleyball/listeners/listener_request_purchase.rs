use crate::game_events::GameEvents;
use crate::state::host::state_currency::StateCurrency;
use crate::state::host::state_deck_exploration::StateDeckExploration;
use crate::state::host::state_shop::{StateShop, StockItems};
use curio_core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use gameplay::context_3d::Context3D;
use gameplay::traits::{impulse::Impulse, scope::Scope};
use impulse::impulse;

#[derive(Default)]
#[impulse(GameEvents)]
pub struct Listener {}

impl Scope for Listener {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<NetworkModes> {
        NetworkModes::all_host()
    }
}
impl Impulse<GameEvents> for Listener {
    fn dequeue_event(&mut self, game_state: &mut GameState, _world: &mut Context3D, _event_queue: &mut EventQueue, event: &GameEvents) {
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
