use crate::built_in::facet::transform::transform2d::Transform2D;
use crate::form::Form;
use crate::form_ref::FormRef;
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

    /// Spawn a Form inside the Context
    pub fn spawn(&mut self, name: &str, t: Transform2D) -> Form {
        // spawn a new entity inside the hecs_world
        let hecs_world = self.hecs_world();
        let entity = {
            // borrow
            let mut world = hecs_world.borrow_mut();
            // spawn - dont know how to spawn with only a single tranform
            world.spawn(())
        };

        // create the form
        let form = FormRef::new(name, hecs_world, entity).add_facet(t);
        // return the form
        form
    }
}
