use crate::gameobject::GameObject;
use crate::static_data::global_components::get_global_ecs_instances;
use core::io::asset_loader::PrefabGameObject;
use hecs::{QueryMut, World};
use std::cell::RefCell;
use std::rc::Rc;

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
