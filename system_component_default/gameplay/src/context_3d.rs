use crate::built_in::facet::transform::transform3d::Transform3D;
use crate::form::Form;
use crate::form_ref::FormRef;
use crate::traits_internal::world_context_common::ContextCommon;
use hecs::World;
use std::cell::RefCell;
use std::rc::Rc;

/// 3D access to the world
pub struct Context3D {
    pub world: Rc<RefCell<World>>,
}

impl ContextCommon for Context3D {
    fn hecs_world(&self) -> Rc<RefCell<World>> {
        self.world.clone()
    }
}
impl Context3D {
    /// Create a new context
    pub fn new(world: Rc<RefCell<World>>) -> Self {
        Self { world }
    }

    /// Spawn a Form inside the Context
    pub fn spawn(&mut self, name: &str, t: Transform3D) -> Form {
        // spawn a new entity inside the hecs_world
        let hecs_world = self.hecs_world();
        let entity = {
            // borrow
            let mut world = hecs_world.borrow_mut();
            // spawn - dont know how to spawn with only a single tranform
            world.spawn(())
        };

        // spawn the form
        let form = FormRef::new(name, hecs_world, entity).add_facet(t);
        // return
        form
    }
}
