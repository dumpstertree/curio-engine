use built_in_state::{state_camera::CameraState, state_network::StateNetwork};
use ecs_system::global_ecs_system;
use hecs::World;

use core::{
    collections::{
        event_queue::{self, EventQueue},
        game_state::GameState,
    },
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::ecs_system::ECSSystemEventless,
};

use crate::{
    game_events::GameEvents,
    state::{
        state_ball_mode::{BallModes, StateBallMode},
        state_deck::{Card, CardTypes, Deck, StateDeck},
        state_energy::StateEnergy,
        state_position_ball::StatePositionBall,
        state_position_player::StatePositionPlayer,
        state_teams::{StateTeamAssignments, Teams},
        state_turn::StateTurn,
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
                deck.hand_persistent = vec![Card::new("Rest", "card_bump.glb", CardTypes::Rest, 0)];
                deck.pile_draw = vec![
                    Card::new("Bump 0", "card_bump.glb", CardTypes::Bump, 1),
                    Card::new("Bump 1", "card_bump.glb", CardTypes::Bump, 1),
                    Card::new("Bump 2", "card_bump.glb", CardTypes::Bump, 1),
                    Card::new("Bump 3", "card_bump.glb", CardTypes::Bump, 1),
                    Card::new("Bump 4", "card_bump.glb", CardTypes::Bump, 1),
                    Card::new("Set 0", "card_set.glb", CardTypes::Set, 1),
                    Card::new("Set 1", "card_set.glb", CardTypes::Set, 1),
                    Card::new("Set 2", "card_set.glb", CardTypes::Set, 1),
                    Card::new("Set 3", "card_set.glb", CardTypes::Set, 1),
                    Card::new("Set 4", "card_set.glb", CardTypes::Set, 1),
                    Card::new("Spike 0", "card_spike.glb", CardTypes::Spike, 3),
                    Card::new("Spike 1", "card_spike.glb", CardTypes::Spike, 3),
                    Card::new("Spike 2", "card_spike.glb", CardTypes::Spike, 3),
                    Card::new("Spike 3", "card_spike.glb", CardTypes::Spike, 3),
                    Card::new("Spike 4", "card_spike.glb", CardTypes::Spike, 3),
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
