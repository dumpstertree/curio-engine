use core::io::asset_loader::PrefabGameObject;
use std::rc::Rc;
use std::{cell::RefCell, hash::Hash};

use hecs::{Component, Entity, QueryMut, World};

use crate::component::component_transform::Transform;
use crate::component::component_transform2d::Transform2D;
use crate::static_data::global_components::get_global_ecs_instances;

pub trait WorldContextCommon {
    /// Removes all entities and components.
    fn clear(&mut self) {
        self.get_world().borrow_mut().clear();
    }

    fn get<Q>(&self) -> Vec<Q>
    where
        Q: hecs::Component + Clone,
    {
        let world_ref = self.get_world();
        let world = world_ref.borrow(); // Immutable borrow
        let mut out = Vec::new();

        for (_entity, component) in world.query::<&Q>().iter() {
            out.push(component.clone()); // Clone Q
        }

        out
    }
    fn instantiate_prefab(&mut self, prefab: &PrefabGameObject) -> GameObject {
        // create entity
        let entity = {
            let world = self.get_world();
            let mut world = world.borrow_mut();
            world.spawn(())
        };

        println!("instantiated {}", prefab.name);

        let mut go = GameObject::new(
            &prefab.name,
            self.get_world().clone(),
            entity,
            prefab
                .children
                .iter()
                .map(|x| self.instantiate_prefab(x))
                .collect(),
        );
        for component in &prefab.components {
            let x = get_global_ecs_instances(&component.r#type);
            x(&mut go, &component.fields);
        }

        go
    }

    // /// Collect query results into an owned Vec so the borrow ends inside the fn.
    // fn query<Q>(&self) -> Vec<(Entity, <Q as Query>::Item<'static>)>
    // where
    //     Q: hecs::Query + Clone,
    // {
    //     let world = self.get_world();
    //     let world_ref = world.borrow();

    //     let mut out: Vec<(Entity, <Q as Query>::Item<'_>)> = Vec::new();

    //     // iterate produces Q::Item<'a>
    //     for item in world_ref.query::<Q>().iter() {
    //         out.push(item.clone());
    //     }

    //     out
    // }

    /// Borrow a query from the world.
    fn query_mut<Q>(&self, f: impl FnOnce(QueryMut<Q>))
    where
        Q: hecs::Query,
    {
        let w = self.get_world();
        let mut world_ref = w.borrow_mut();
        let q = world_ref.query_mut::<Q>();
        f(q);
    }

    /// Destroy a GameObject (Unity-style "Destroy").
    fn destroy(&mut self, go: GameObject) {
        let _ = self.get_world().borrow_mut().despawn(go.entity);
    }

    fn get_world(&self) -> Rc<RefCell<World>>;
}
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
