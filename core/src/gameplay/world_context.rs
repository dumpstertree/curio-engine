use std::cell::RefCell;
use std::rc::Rc;

use hecs::{Component, Entity, QueryBorrow, QueryMut, World};

pub struct WorldContext {
    pub world: Rc<RefCell<World>>,
}

impl WorldContext {
    pub fn new() -> Self {
        Self { world: Rc::new(RefCell::new(World::new())) }
    }

    /// Removes all entities and components.
    pub fn clear(&mut self) {
        self.world.borrow_mut().clear();
    }
    /// Borrow a query from the world.
    pub fn query<Q>(&self, f: impl FnOnce(QueryBorrow<Q>))
    where
        Q: hecs::Query,
    {
        let world_ref = self.world.borrow();
        let q = world_ref.query::<Q>();
        f(q);
    }
    /// Borrow a query from the world.
    pub fn query_mut<Q>(&self, f: impl FnOnce(QueryMut<Q>))
    where
        Q: hecs::Query,
    {
        let mut world_ref = self.world.borrow_mut();
        let q = world_ref.query_mut::<Q>();
        f(q);
    }

    /// Create a new empty GameObject (Unity-style "Instantiate").
    pub fn instantiate(&mut self) -> GameObject {
        let entity: Entity = self.world.borrow_mut().spawn(());

        GameObject::new(self.world.clone(), entity)
    }

    /// Destroy a GameObject (Unity-style "Destroy").
    pub fn destroy(&mut self, go: GameObject) {
        let _ = self.world.borrow_mut().despawn(go.entity);
    }
}
/// A Unity-style wrapper for an ECS entity.
#[derive(Clone)]
pub struct GameObject {
    world: Rc<RefCell<World>>,
    pub entity: Entity,
    pub name: String,
}
impl PartialEq for GameObject {
    fn eq(&self, other: &Self) -> bool {
        // currently doesnt check world
        self.entity == other.entity && self.name == other.name
    }
}

impl GameObject {
    pub fn new(world: Rc<RefCell<World>>, entity: Entity) -> Self {
        Self { world, entity, name: String::new() }
    }

    // -------------------------------
    // Component Management
    // -------------------------------

    /// Add a component T using its default value.
    pub fn add_component_default<T>(&self) -> &GameObject
    where
        T: Component + Default,
    {
        self.world
            .borrow_mut()
            .insert_one(self.entity, T::default())
            .expect("Failed to insert component");

        self
    }

    /// Add a specific component instance.
    pub fn add_component_value<T>(&self, value: T) -> &GameObject
    where
        T: Component,
    {
        self.world
            .borrow_mut()
            .insert_one(self.entity, value)
            .expect("Failed to insert component");

        self
    }

    /// Modify a component in-place.
    pub fn edit_component<T: Component + 'static>(&self, edit_fn: impl FnOnce(&mut T)) {
        let world = self.world.borrow_mut();
        let mut borrow = world
            .get::<&mut T>(self.entity)
            .unwrap_or_else(|_| panic!("GameObject '{}' does not contain component {}", self.name, std::any::type_name::<T>(),));
        edit_fn(&mut *borrow);
    }

    /// Get a cloned component value (Unity style).
    pub fn get_component<T: Component + Clone + 'static>(&self) -> Option<T> {
        let world = self.world.borrow();
        let borrow = world.get::<&T>(self.entity).ok()?;
        Some((*borrow).clone())
    }

    /// Returns true if the object contains component T.
    pub fn has_component<T: Component + 'static>(&self) -> bool {
        self.world.borrow().get::<&T>(self.entity).is_ok()
    }

    pub fn destroy(&self) {
        let _ = self.world.borrow_mut().despawn(self.entity);
    }
}
