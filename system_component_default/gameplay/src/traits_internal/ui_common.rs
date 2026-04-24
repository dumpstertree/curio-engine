use crate::context_2d::Context2D;
use curio_core::collections::{event_queue::EventQueue, game_state::Ledger};

pub trait UICommon {
    fn init(&mut self);
    /*
       let obj = context.spawn()
       let t = obj.addcomponent<Text>
       let a = obj.addcomponent<Audio>

        t.set_contents()

        obj.destroy()

    */
    fn present(&mut self, game_state: &mut Ledger, event_queue: &mut EventQueue, context: &mut Context2D);
    fn dismiss(&mut self, game_state: &mut Ledger, event_queue: &mut EventQueue, context: &mut Context2D);
    fn tick(&mut self, game_state: &mut Ledger, event_queue: &mut EventQueue, context: &mut Context2D);
}
