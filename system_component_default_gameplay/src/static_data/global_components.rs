use std::{
    any::type_name,
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

use crate::{
    form::Form,
    traits::{facet_common::FacetCommon, field_override::FieldOverride},
};

/// Function that creates a boxed untyped value (what register stores)
type AddComponentFn = fn(&mut Form, &Vec<String>) -> bool;

struct ReceiverRegistry {
    add_component: HashMap<String, AddComponentFn>,
}

static COMPONENT_REGISTRY: LazyLock<RwLock<ReceiverRegistry>> = LazyLock::new(|| RwLock::new(ReceiverRegistry { add_component: HashMap::new() }));

pub fn register_global_component<T>()
where
    T: Default + FacetCommon + FieldOverride,
{
    let key = type_name::<T>()
        .split("::")
        .last()
        .filter(|x| true)
        .unwrap()
        .to_lowercase();
    let mut reg: std::sync::RwLockWriteGuard<'_, ReceiverRegistry> = COMPONENT_REGISTRY.write().expect("Registry poisoned");

    println!("TRY ADD component {}", type_name::<T>());

    reg.add_component.insert(key, |x, y| {
        println!("ADD component {}", type_name::<T>());
        let mut serialized = String::new();
        for key_val in y {
            serialized = format!("{}{}\n", serialized, key_val);
        }

        println!("ser: {}", serialized);
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
        return true;
    });
}
pub fn get_global_ecs_instances(name: &str) -> AddComponentFn
where {
    let reg = COMPONENT_REGISTRY.read().expect("Registry poisoned");
    let val = reg.add_component.get(&name.to_lowercase());

    println!("get component {}", name);
    // take any for testing
    return *val.unwrap();
}
