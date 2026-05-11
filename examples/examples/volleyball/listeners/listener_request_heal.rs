use crate::game_events::GameEvents;
use crate::state::host::state_currency::StateCurrency;
use crate::state::state_score::StateScore;
use crate::state::state_teams::StateTeamAssignments;
use curio_core::collections::{event_queue::Nerve, ledger::Ledger};
use curio_core::network_modes::NetworkModes;
use gameplay::context_3d::Context3D;
use gameplay::traits::{impulse::Impulse, scope::Scope};
use impulse::impulse;

#[derive(Default)]
#[impulse(GameEvents)]
pub struct Listener {}

impl Scope for Listener {
    fn is_enabled(&mut self, _: &mut Ledger) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut Ledger) -> Vec<NetworkModes> {
        NetworkModes::all_host()
    }
}
impl Impulse<GameEvents> for Listener {
    fn dequeue_event(&mut self, ledger: &mut Ledger, _world: &mut Context3D, _event_queue: &mut Nerve, event: &GameEvents) {
        match event {
            GameEvents::RequestHeal(user_guid) => {
                let state_currency = ledger.read::<StateCurrency>();
                if state_currency.currency < 100 {
                    println!("Not enough Currency. Require 100 have {}", state_currency.currency);
                    return;
                }

                let state_teams = ledger.read::<StateTeamAssignments>();
                ledger.write::<StateCurrency>(|x| {
                    x.currency -= 100;
                });
                ledger.write::<StateScore>(|x| {
                    if let Some(team) = state_teams.team_for(user_guid) {
                        if let Some(score) = x.all_scores.get_mut(&team) {
                            *score += 1;
                        }
                    }
                });
            }
            _ => {}
        }
    }
}
