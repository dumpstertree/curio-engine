use crate::form::Form;
use crate::form_ref::FormRef;
use crate::static_data::global_components::get_global_ecs_instances;
use curio_core::io::asset_loader::PrefabGameObject;
use hecs::{QueryMut, World};
use std::cell::RefCell;
use std::rc::Rc;

pub trait ContextCommon {
    /// Removes all entities and components.
    fn clear(&mut self) {
        self.hecs_world().borrow_mut().clear();
    }

    /// Get Facets-Form combinations in context
    fn get<Q>(&self) -> Vec<Q>
    where
        Q: hecs::Component + Clone,
    {
        let world_ref = self.hecs_world();
        let world = world_ref.borrow(); // Immutable borrow
        let mut out = Vec::new();

        for (_entity, component) in world.query::<&Q>().iter() {
            out.push(component.clone()); // Clone Q
        }

        out
    }

    /// Edit Facets-Form combinations in context
    fn edit<Q>(&self, f: impl FnOnce(QueryMut<Q>))
    where
        Q: hecs::Query,
    {
        let w = self.hecs_world();
        let mut world_ref = w.borrow_mut();
        let q = world_ref.query_mut::<Q>();
        f(q);
    }

    /// instantiate a prefab into the context
    fn spawn_prefab_recursive(&mut self, prefab: &PrefabGameObject) -> Form {
        let hecs_world = self.hecs_world();
        // create entity
        let entity = {
            // spawn the entity
            let mut world = hecs_world.borrow_mut();
            // spawn
            world.spawn(())
        };

        // create children forms
        let child_forms: Vec<Form> = prefab
            .children
            .iter()
            .map(|x| self.spawn_prefab_recursive(x))
            .collect();
        println!("SPAWN PREFAB RECURSIVE CHILDREN LEN {}", child_forms.len());
        //
        let mut parent_form = FormRef::new(&prefab.name, hecs_world, entity);

        // create parent child relationship
        for mut child in child_forms {
            child.set_parent(Some(parent_form.clone()));
        }

        // add all facets
        for facet in &prefab.components {
            // get fn from global
            let facet_fn = get_global_ecs_instances(&facet.r#type);
            // create facet
            facet_fn(&mut parent_form, &facet.fields);
        }

        // return the parent form
        parent_form
    }

    fn hecs_world(&self) -> Rc<RefCell<World>>;
}
