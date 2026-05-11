use crate::{
    game_events::GameEvents,
    state::{
        host::{state_card_attribute_modifier_stack::StateCardAttributeModifierStack, state_heat::StateHeat},
        state_deck::StateDeck,
        state_energy::StateEnergy,
        state_teams::StateTeamAssignments,
        state_turn::StateTurn,
    },
};
use curio_core::{
    collections::{event_queue::Nerve, ledger::Ledger},
    network_modes::NetworkModes,
};
use gameplay::{
    context_3d::Context3D,
    traits::{impulse::Impulse, scope::Scope},
};
use impulse::impulse;

#[derive(Default)]
#[impulse(GameEvents)]
pub struct ECsystemGameTurnBegin {}
impl Scope for ECsystemGameTurnBegin {
    fn is_enabled(&mut self, _: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut Ledger) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
impl Impulse<GameEvents> for ECsystemGameTurnBegin {
    fn dequeue_event(&mut self, ledger: &mut Ledger, _: &mut Context3D, events: &mut Nerve, event: &GameEvents) {
        match event {
            GameEvents::TurnBegin(id) => {
                // end this turn
                println!("Instance: {}. Begin Turn {}", ledger.network.me().guid, id);

                ledger.write::<StateTurn>(|x| {
                    x.active_instance_id = *id;
                });

                for guid in ledger
                    .read::<StateTeamAssignments>()
                    .team_assignments
                    .get(id)
                    .unwrap()
                {
                    let state_modifiers = ledger.read::<StateCardAttributeModifierStack>();
                    let mod_stack = state_modifiers.get_flat_stack_for_entity(*guid);

                    let state_energy = ledger.read::<StateEnergy>();
                    let cur_energy = state_energy.all_players.get(guid).unwrap_or(&(0, 0));

                    println!("cur energy {}", cur_energy.0);

                    ledger.write::<StateDeck>(|x| {
                        if let Some(deck) = x.deck.get_mut(guid) {
                            deck.draw(1);
                        }
                    });
                    ledger.write::<StateHeat>(|x| {
                        if !x.all_players.contains_key(guid) {
                            x.all_players.insert(*guid, cur_energy.0);
                        } else {
                            let c = x.all_players[guid];
                            x.all_players.insert(*guid, c + cur_energy.0);
                        }
                    });

                    println!("heat {}", ledger.read::<StateHeat>().all_players[guid]);
                    // update energy
                    ledger.write::<StateEnergy>(|x| {
                        if let Some(y) = x.all_players.get_mut(guid) {
                            y.0 = y.1 + mod_stack.energy;
                        }
                    });
                }

                println!("send did turn begin");
                events.enqueue_event(GameEvents::DidTurnBegin(*id));
            }
            _ => {}
        }
    }
}
