use crate::component::component_transform::Transform;
use crate::gameobject::GameObject;
use crate::world_context_common::WorldContextCommon;
use hecs::World;
use std::cell::RefCell;
use std::rc::Rc;

pub struct WorldContext {
    pub world: Rc<RefCell<World>>,
}

impl WorldContextCommon for WorldContext {
    fn get_world(&self) -> Rc<RefCell<World>> {
        self.world.clone()
    }
}
impl WorldContext {
    pub fn new(world: Rc<RefCell<World>>) -> Self {
        Self { world }
    }

    ///
    pub fn instantiate(&mut self, name: &str, t: Transform) -> GameObject {
        let entity = {
            let world = self.get_world();
            let mut world = world.borrow_mut();
            world.spawn(())
        };

        let go = GameObject::new(name, self.get_world().clone(), entity, vec![]).add_component_value(t);
        go
    }
}
