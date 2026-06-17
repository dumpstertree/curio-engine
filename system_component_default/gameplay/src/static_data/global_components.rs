use std::{
    any::type_name,
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

use curio_core::{ComponentState, FieldState, Severity};

use crate::{
    form::Form,
    traits::{facet_common::FacetCommon, field_override::FieldOverride},
};

/// Function that creates a boxed untyped value (what register stores)
type AddComponentFn = fn(&mut Form, &Vec<String>) -> bool;
type GetStateFn = fn(&Form) -> Option<ComponentState>;

pub struct ReceiverRegistry {
    pub add_component: HashMap<String, AddComponentFn>,
    pub get_state: HashMap<String, GetStateFn>,
}

pub static COMPONENT_REGISTRY: LazyLock<RwLock<ReceiverRegistry>> = LazyLock::new(|| {
    RwLock::new(ReceiverRegistry {
        add_component: HashMap::new(),
        get_state: HashMap::new(),
    })
});

pub fn register_global_component<T>()
where
    T: Default + Clone + FacetCommon + FieldOverride,
{
    let key = type_name::<T>()
        .split("::")
        .last()
        .filter(|_x| true)
        .unwrap()
        .to_lowercase();
    let mut reg: std::sync::RwLockWriteGuard<'_, ReceiverRegistry> = COMPONENT_REGISTRY.write().expect("Registry poisoned");

    // Curio::log(Severity::Info, &format!("Registered Global Habit: {}", type_name::<T>()));

    reg.get_state.insert(key.clone(), |x| {
        let key = type_name::<T>()
            .split("::")
            .last()
            .filter(|_x| true)
            .unwrap()
            .to_lowercase();
        if let Some(f) = x.get_facet::<T>() {
            return Some(ComponentState { component_name: key, fields: f.get_state() });
        }
        return None;
    });
    reg.add_component.insert(key, |x, y| {
        if x.has_facet::<T>() {
            x.edit_facet::<T>(|f| {
                for e in y {
                    let mut s = e.split(":");
                    f.apply(s.next().unwrap(), s.next().unwrap());
                }
            });
        } else {
            let mut val = T::default();
            for e in y {
                let mut s = e.split(":");
                val.apply(s.next().unwrap(), s.next().unwrap());
            }
            // let asset = serde_yaml::from_str::<T>(&serialized);
            // if let Err(e) = asset {
            //     panic!("{}", e);
            // }

            x.clone().add_facet::<T>(val);
        }

        return true;
    });
}
pub fn get_global_ecs_instances(name: &str) -> AddComponentFn
where {
    let reg = COMPONENT_REGISTRY.read().expect("Registry poisoned");
    let val = reg.add_component.get(&name.to_lowercase());

    let Some(val) = val else {
        panic!("Unknown Habit with name '{}'", name);
    };
    // take any for testing
    return *val;
}
