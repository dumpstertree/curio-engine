use crate::built_in::facet::facet_transform::component_transform2d::Transform2D;
use crate::gameobject::GameObject;
use crate::traits_internal::world_context_common::WorldContextCommon;
use hecs::World;
use std::cell::RefCell;
use std::rc::Rc;

pub struct WorldContext2D {
    pub world: Rc<RefCell<World>>,
}
impl WorldContextCommon for WorldContext2D {
    fn get_world(&self) -> Rc<RefCell<World>> {
        self.world.clone()
    }
}
impl WorldContext2D {
    pub fn new(world: Rc<RefCell<World>>) -> Self {
        Self { world }
    }

    /// Create a new empty GameObject (Unity-style "Instantiate").
    pub fn instantiate(&mut self, name: &str, t: Transform2D) -> GameObject {
        let entity = {
            let world = self.get_world();
            let mut world = world.borrow_mut();
            world.spawn(())
        };
        let go = GameObject::new(name, self.get_world().clone(), entity, vec![]).add_component_value::<Transform2D>(t);
        go
    }
}
