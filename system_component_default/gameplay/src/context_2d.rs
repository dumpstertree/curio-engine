use crate::traits_internal::world_context_common::ContextCommon;
use hecs::World;
use std::cell::RefCell;
use std::rc::Rc;

/// 2D access to the world
pub struct Context2D {
    pub world: Rc<RefCell<World>>,
}
impl ContextCommon for Context2D {
    fn hecs_world(&self) -> Rc<RefCell<World>> {
        self.world.clone()
    }
}
impl Context2D {
    /// Create a new context
    pub fn new(world: Rc<RefCell<World>>) -> Self {
        Self { world }
    }
}
