use crate::{
    game_events::GameEvents,
    state::{state_energy::StateEnergy, state_turn::StateTurn},
};
use built_in_state::state_time::TimeState;
use core::{
    collections::{event_queue::EventQueue, game_state::GameState},
    dumpster_engine::NetworkModes,
    gameplay::ecs::traits::{
        ecs_event_reciever::{self, InstanceLimiter},
        ecs_system::ECSSystemEventless,
    },
};
use ecs_event::global_ecs_system_event_reciever;
use ecs_system::global_ecs_system;
use hecs::World;

#[global_ecs_system]
#[global_ecs_system_event_reciever(GameEvents)]
pub struct ECSSystemGameTurnBegin {
    do_move: bool,
    lastmove: f64,
}
impl InstanceLimiter for ECSSystemGameTurnBegin {
    fn is_enabled(&mut self, _: &mut GameState) -> bool {
        true
    }
    fn run_on_instance(&mut self, _: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
}
impl ECSSystemEventless for ECSSystemGameTurnBegin {
    fn is_enabled(&mut self, _: &mut GameState, _: &mut World) -> bool {
        true
    }
    fn run_on_instance(&mut self, game_state: &mut GameState) -> Vec<core::dumpster_engine::NetworkModes> {
        vec![NetworkModes::LocalHost, NetworkModes::OnlineHost]
    }
    // fn tick(&mut self, game_state: &mut GameState, _: &mut World, events: &mut EventQueue) {
    //     if self.do_move && game_state.get_value2::<TimeState>().unscaled_time - self.lastmove > 1.0 {
    //         println!("start new move");
    //         let e = run_ai(game_state);

    //         match &e {
    //             GameEvents::RequestTurnEnd(_) => self.do_move = false,
    //             _ => {}
    //         }

    //         println!("send event");
    //         events.enqueue_event(e);
    //         self.lastmove = game_state.get_value2::<TimeState>().unscaled_time;
    //     }

    //     // println!("end new move");
    //     // println!("tick");
    //     // events.enqueue_event(GameEvents::TurnEnd(game_state.get_value2::<StateTurn>().active_instance_id));
    // }
}
impl ecs_event_reciever::EventReciever<GameEvents> for ECSSystemGameTurnBegin {
    fn dequeue_event(&mut self, game_state: &mut GameState, _: &mut World, events: &mut EventQueue, event: &GameEvents) {
        match event {
            GameEvents::TurnBegin(id) => {
                // end this turn
                println!("Instance: {}. Begin Turn {}", game_state.instance_id, id);

                game_state.edit::<StateTurn>(|x| {
                    x.active_instance_id = *id;
                });
                // update energy
                game_state.edit::<StateEnergy>(|x| {
                    let cur = x.all_players[id];
                    x.all_players.insert(*id, (cur.1, cur.1));
                });

                self.do_move = true;
                game_state.get::<TimeState>().unscaled_time;

                println!("send did turn begin");
                events.enqueue_event(GameEvents::DidTurnBegin(*id));
            }
            _ => {}
        }
    }
}
