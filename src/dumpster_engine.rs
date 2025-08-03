use std::any::TypeId;

use hecs::{Entity, World};
use pollster::FutureExt;
use serde_json::Value;

use crate::system::system_components::collision_component_factory::SystemComponentCollisionFactory;
use crate::system::system_components::gameplay_component_factory::SystemComponentGameplayFactory;
use crate::system::system_components::gameplay_components::gameplay_component_default::{ECSSystem, ECSSystemEventless};
use crate::system::system_components::graphics_component_factory::SystemComponentGraphicsFactory;
use crate::system::system_components::input_component_factory::SystemComponentInputFactory;
use crate::system::system_components::time_component_factory::SystemComponentTimeFactory;
use crate::system_adapters::adapter_system_gpu::SystemGPU;
use crate::Window::SystemWindow::SystemWindow;

// static REGISTRY: Mutex<Vec<(TypeId, ComponentInfo)>> = Mutex::new(Vec::new());

static mut REGISTRY: Vec<(TypeId, ComponentInfo)> = Vec::new();

struct ComponentInfo<'a> {
    name: &'static str,
    serializer: fn(&'a World, Entity) -> Option<Value>,
}

pub fn register_component<T>(name: &'static str)
where
    T: 'static + serde::Serialize + Clone + Send + Sync + hecs::Component,
{
    fn serialize<T>(world: &hecs::World, entity: hecs::Entity) -> Option<serde_json::Value>
    where
        T: 'static + serde::Serialize + Clone + Send + Sync + hecs::Component,
    {
        world
            .get::<&T>(entity)
            .ok()
            .and_then(|comp| serde_json::to_value(&*comp).ok())
    }

    // unsafe {
    //     REGISTRY.push((
    //         TypeId::of::<T>(),
    //         ComponentInfo {
    //             name,
    //             serializer: serialize::<T>,
    //         },
    //     ));
    // }
}

// Serialize all attached components for a given entity
// pub fn serialize_entity<'a>(world: &'a World, entity: Entity) -> Value {
//     unsafe {
//         let mut out = Map::new();
//         // let reg = REGISTRY; //.lock().unwrap();
//         // for info in reg.iter() {
//         for info in REGISTRY {
//             if let Some(val) = (info.1.serializer)(world, entity) {
//                 out.insert(info.1.name.to_string(), val);
//             }
//         }
//         Value::Object(out)
//     }
// }

static mut REGISTERED_ECS_SYSTEMS: Vec<fn() -> Box<dyn ECSSystemEventless>> = Vec::new();

pub struct DumpsterEngine {}
impl DumpsterEngine {
    pub fn register_ecs_system<T>()
    where
        T: 'static + ECSSystemEventless + Default,
    {
        unsafe {
            let x: fn() -> Box<dyn ECSSystemEventless> = || return Box::new(T::default());
            REGISTERED_ECS_SYSTEMS.push(x);
        }
    }
    pub fn run<TGameEvents>(window_layout: WindowLayout, ecs_systems: Vec<Box<dyn ECSSystem<TGameEvents>>>)
    where
        TGameEvents: 'static,
        TGameEvents: Clone,
    {
        let event_loop = SystemGPU::init().block_on();

        // set window layout values
        SystemGPU::set_resolution(window_layout.width, window_layout.height);
        SystemGPU::set_fullscreen(window_layout.fullscreen);
        SystemGPU::set_resizable(window_layout.resizeable);
        SystemGPU::set_cursor_visible(window_layout.show_cursor);

        // create built in systems
        let mut ecs_system_built_in: Vec<Box<dyn ECSSystemEventless>> = vec![];

        unsafe {
            for x in REGISTERED_ECS_SYSTEMS.iter() {
                ecs_system_built_in.push(x());
            }
        }
        // create systems
        let mut system_window = SystemWindow::new(vec![
            SystemComponentTimeFactory::create(),
            SystemComponentInputFactory::create(),
            SystemComponentGraphicsFactory::create(),
            SystemComponentCollisionFactory::create(),
            SystemComponentGameplayFactory::create(ecs_systems, ecs_system_built_in),
        ]);

        // run the window
        system_window.run(event_loop);
    }
}

pub struct WindowLayout {
    width: i32,
    height: i32,
    fullscreen: bool,
    resizeable: bool,
    show_cursor: bool,
}

impl WindowLayout {
    pub fn fullscreen_1080() -> WindowLayout {
        WindowLayout {
            width: 1920,
            height: 1080,
            fullscreen: true,
            resizeable: false,
            show_cursor: true,
        }
    }
    pub fn fullscreen_720() -> WindowLayout {
        WindowLayout {
            width: 1280,
            height: 720,
            fullscreen: true,
            resizeable: false,
            show_cursor: true,
        }
    }
    pub fn windowed_1080() -> WindowLayout {
        WindowLayout {
            width: 1920,
            height: 1080,
            fullscreen: false,
            resizeable: false,
            show_cursor: true,
        }
    }
    pub fn windowed_720() -> WindowLayout {
        WindowLayout {
            width: 1280,
            height: 720,
            fullscreen: false,
            resizeable: false,
            show_cursor: true,
        }
    }
    pub fn custom(width: i32, height: i32, fullscreen: bool, resizeable: bool, show_cursor: bool) -> WindowLayout {
        WindowLayout {
            width: width,
            height: height,
            fullscreen: fullscreen,
            resizeable: resizeable,
            show_cursor: show_cursor,
        }
    }
}
