use crate::form::Form;
use crate::form_ref::FormRef;
use crate::static_data::global_components::get_global_ecs_instances;
use curio_core::{Assets, Composition, Curio, Severity};
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

    fn hecs_world(&self) -> Rc<RefCell<World>>;
}

pub(crate) fn spawn_prefab_recursive_internal(world: Rc<RefCell<World>>, comp: &Composition, name: String) -> Form {
    let mut this_form: Option<Form> = None;
    if comp.base != String::new() {
        if let Ok(uid) = comp.base.parse::<i16>() {
            let asset = Assets::load_asset::<Composition>(&uid);

            // spawn parent
            this_form = Some(spawn_prefab_recursive_internal(world.clone(), &(*asset.clone()), asset.name.clone()));
        } else {
            // pull out the key - eventually remove this and just store the id in the compositiom
            if let Some(key) = Assets::try_lookup_key_for_name(&comp.base) {
                // retrieve the composition asset
                let asset = Assets::load_asset::<Composition>(&key);

                // spawn parent
                this_form = Some(spawn_prefab_recursive_internal(world.clone(), &(*asset.clone()), asset.name.clone()));
            } else {
                Curio::log(Severity::Warning, &format!("Unable to load Composition for ID: {}", comp.base.clone()));
            }
        }
    }

    // create children forms
    let child_forms: Vec<Form> = comp
        .children
        .iter()
        .map(|x| spawn_prefab_recursive_internal(world.clone(), x, x.name.clone()))
        .collect();

    // create a fallback form
    if this_form.is_none() {
        // create entity
        let entity = {
            // spawn the entity
            let mut world = world.borrow_mut();
            // spawn
            world.spawn(())
        };

        // create a new form
        this_form = Some(FormRef::new(&name, world, entity));
    }

    //
    let Some(mut this_form) = this_form else {
        panic!();
    };

    // create parent child relationship
    for child in child_forms {
        child.set_parent(Some(this_form.clone()));
    }

    // add all facets
    for facet in &comp.components {
        // get fn from global
        let facet_fn = get_global_ecs_instances(&facet.r#type);

        if let Some(f) = facet_fn {
            // create facet
            f(&mut this_form, &facet.fields);
        }
    }

    return this_form;

    // if name.starts_with("!") {
    //     let name = comp.name.clone();
    //     let split: Vec<&str> = name.split("::").into_iter().collect();
    //     let name = split[0].replace("!", "");

    //     // let Some(key) = AssetLoader::try_lookup_key_for_name(&name) else {
    //     //     panic!();
    //     // };

    //     let Some(key) = Assets::try_lookup_key_for_name(&name) else {
    //         panic!();
    //     };

    //     // create children forms
    //     let child_forms: Vec<Form> = comp
    //         .children
    //         .iter()
    //         .map(|x| spawn_prefab_recursive_internal(world.clone(), x, x.name.clone()))
    //         .collect();

    //     let asset = Assets::load_asset::<Composition>(&key);
    //     let mut parent_form = spawn_prefab_recursive_internal(world.clone(), &asset, split[1].to_owned());

    //     // create parent child relationship
    //     for child in child_forms {
    //         child.set_parent(Some(parent_form.clone()));
    //     }

    //     // add all facets
    //     for facet in &comp.components {
    //         // get fn from global
    //         let facet_fn = get_global_ecs_instances(&facet.r#type);

    //         if let Some(f) = facet_fn {
    //             // create facet
    //             f(&mut parent_form, &facet.fields);
    //         }
    //     }

    //     return parent_form;
    // }
    // // create entity
    // let entity = {
    //     // spawn the entity
    //     let mut world = world.borrow_mut();
    //     // spawn
    //     world.spawn(())
    // };

    // // create children forms
    // let child_forms: Vec<Form> = comp
    //     .children
    //     .iter()
    //     .map(|x| spawn_prefab_recursive_internal(world.clone(), x, x.name.clone()))
    //     .collect();
    // //
    // let mut parent_form = FormRef::new(&name, world, entity);

    // // create parent child relationship
    // for child in child_forms {
    //     child.set_parent(Some(parent_form.clone()));
    // }

    // // add all facets
    // for facet in &comp.components {
    //     // get fn from global
    //     let facet_fn = get_global_ecs_instances(&facet.r#type);

    //     if let Some(f) = facet_fn {
    //         // create facet
    //         f(&mut parent_form, &facet.fields);
    //     }
    // }

    // // return the parent form
    // parent_form
}
