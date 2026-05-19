use crate::context_2d::Context2D;
use curio_core::Nerve;
use curio_core::Ledger;

pub trait UICommon {
    fn init(&mut self);
    /*
       let obj = context.spawn()
       let t = obj.addcomponent<Text>
       let a = obj.addcomponent<Audio>

        t.set_contents()

        obj.destroy()

    */
    fn present(&mut self, ledger: &mut Ledger, event_queue: &mut Nerve, context: &mut Context2D);
    fn dismiss(&mut self, ledger: &mut Ledger, event_queue: &mut Nerve, context: &mut Context2D);
    fn tick(&mut self, ledger: &mut Ledger, event_queue: &mut Nerve, context: &mut Context2D);
}
