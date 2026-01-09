use std::{cell::RefCell, hash::Hash, rc::Rc};

use hecs::{Component, Entity, World};

/// A Unity-style wrapper for an ECS entity.
#[derive(Clone)]
pub struct GameObject {
    world: Rc<RefCell<World>>,
    pub entity: Entity,
    pub name: String,
    pub children: Vec<GameObject>,
}
impl PartialEq for GameObject {
    fn eq(&self, other: &Self) -> bool {
        // currently doesnt check world
        self.entity == other.entity && self.name == other.name
    }
}
impl Hash for GameObject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // self.world.hash(state);
        self.entity.hash(state);
        self.name.hash(state);
    }
}

impl GameObject {
    pub fn new(name: &str, world: Rc<RefCell<World>>, entity: Entity, children: Vec<GameObject>) -> Self {
        Self {
            world,
            entity,
            name: name.to_string(),
            children: children,
        }
    }

    // -------------------------------
    // Component Management
    // -------------------------------

    /// Add a component T using its default value.
    pub fn add_component_default<T>(self) -> Self
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
    pub fn add_component_value<T>(self, value: T) -> Self
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
        let cloned = {
            let world = self.world.borrow();
            let borrow = world.get::<&T>(self.entity).ok()?;
            (*borrow).clone()
        }; // borrow ends here

        Some(cloned)
    }
    /// Returns true if the object contains component T.
    pub fn has_component<T: Component + 'static>(&self) -> bool {
        self.world.borrow().get::<&T>(self.entity).is_ok()
    }

    pub fn destroy(&self) {
        let _ = self.world.borrow_mut().despawn(self.entity);
    }
}
