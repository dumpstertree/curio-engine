use crate::world_context_2d::WorldContext2D;
use core::collections::{event_queue::EventQueue, game_state::GameState};

pub trait UICommon {
    fn init(&mut self);
    /*
       let obj = context.spawn()
       let t = obj.addcomponent<Text>
       let a = obj.addcomponent<Audio>

        t.set_contents()

        obj.destroy()

    */
    fn present(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut WorldContext2D);
    fn dismiss(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut WorldContext2D);
    fn tick(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut WorldContext2D);
}
