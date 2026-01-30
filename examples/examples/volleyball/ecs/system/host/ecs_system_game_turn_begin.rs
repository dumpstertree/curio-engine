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
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
};
use gameplay::{
    context_3d::Context3D,
    traits::{habit::Habit, impulse::Impulse, scope::Scope},
};
use habit::habit;
use impulse::impulse;

#[derive(Default)]
#[impulse(GameEvents)]
pub struct ECSSystemGameTurnBegin {}
impl Scope for ECSSystemGameTurnBegin {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
impl Impulse<GameEvents> for ECSSystemGameTurnBegin {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut Context3D, events: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::TurnBegin(id) => {
                // end this turn
                println!("Instance: {}. Begin Turn {}", game_state.instance_id, id);

                game_state.edit::<StateTurn>(|x| {
                    x.active_instance_id = *id;
                });

                for guid in game_state
                    .get::<StateTeamAssignments>()
                    .team_assignments
                    .get(id)
                    .unwrap()
                {
                    let state_modifiers = game_state.get::<StateCardAttributeModifierStack>();
                    let mod_stack = state_modifiers.get_flat_stack_for_entity(*guid);

                    let state_energy = game_state.get::<StateEnergy>();
                    let cur_energy = state_energy.all_players.get(guid).unwrap_or(&(0, 0));

                    println!("cur energy {}", cur_energy.0);

                    game_state.edit::<StateDeck>(|x| {
                        if let Some(deck) = x.deck.get_mut(guid) {
                            deck.draw(1);
                        }
                    });
                    game_state.edit::<StateHeat>(|x| {
                        if !x.all_players.contains_key(guid) {
                            x.all_players.insert(*guid, cur_energy.0);
                        } else {
                            let c = x.all_players[guid];
                            x.all_players.insert(*guid, c + cur_energy.0);
                        }
                    });

                    println!("heat {}", game_state.get::<StateHeat>().all_players[guid]);
                    // update energy
                    game_state.edit::<StateEnergy>(|x| {
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
