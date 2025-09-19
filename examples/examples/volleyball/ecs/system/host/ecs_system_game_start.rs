use built_in_state::{state_camera::CameraState, state_network::StateNetwork};
use ecs_system::global_ecs_system;
use hecs::World;

use core::{
    collections::{
        event_queue::EventQueue,
        game_state::GameState,
    },
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};

use crate::{
    cards::card_instance::CardInstance,
    game_events::GameEvents,
    state::{
        state_deck::{Deck, StateDeck},
        state_energy::StateEnergy,
        state_position_player::StatePositionPlayer,
        state_teams::{StateTeamAssignments, Teams},
    },
};

#[global_ecs_system]
pub struct ECSSystemGameStart {}
impl ECSSystemEventless for ECSSystemGameStart {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
    fn enable(&mut self, game_state: &mut GameState, _: &mut World, event_queue: &mut EventQueue) {
        println!("Instance: {}. Host Startup", game_state.instance_id);

        // set resolution
        game_state.edit::<CameraState>(|x| {
            x.resolution_width = 1920 / 1;
            x.resolution_height = 1080 / 1;
        });

        let mut assignment = Teams::Red;
        for instance in game_state.get_value2::<StateNetwork>().peer_instance_ids() {
            game_state.edit::<StateTeamAssignments>(|x| {
                if !x.team_assignments.contains_key(&assignment) {
                    x.team_assignments.insert(assignment.clone(), vec![]);
                }

                x.team_assignments
                    .get_mut(&assignment)
                    .unwrap()
                    .push(*instance);
            });
            assignment = assignment.next_team();
        }
        for instance in game_state.get_value2::<StateNetwork>().peer_instance_ids() {
            game_state.edit::<StateDeck>(|x| {
                x.deck.insert(*instance, Deck::default());

                let deck = x.deck.get_mut(instance).unwrap();

                // add all cards
                deck.hand_persistent = vec![CardInstance::new("rest")];
                deck.pile_draw = vec![
                    CardInstance::new("bump"),
                    CardInstance::new("bump"),
                    CardInstance::new("bump"),
                    CardInstance::new("set+move"),
                    CardInstance::new("set+move"),
                    CardInstance::new("set+move"),
                    CardInstance::new("spike"),
                    CardInstance::new("spike"),
                    CardInstance::new("spike"),
                    CardInstance::new("extra_oomph"),
                    CardInstance::new("hold_back"),
                    CardInstance::new("curse"),
                    CardInstance::new("blessing"),
                    CardInstance::new("deep_breath"),
                ];
            });
        }
        for instance in game_state.get_value2::<StateNetwork>().peer_instance_ids() {
            // setup player positions
            game_state.edit::<StatePositionPlayer>(|x| {
                x.positions.insert(*instance, (0, 0));
            });
        }
        for instance in game_state.get_value2::<StateNetwork>().peer_instance_ids() {
            // setup player positions
            game_state.edit::<StateEnergy>(|x| {
                x.all_players.insert(*instance, (0, 0));
            });
        }

        event_queue.enqueue_event(GameEvents::ResetBoard(Teams::Red));
    }
}
