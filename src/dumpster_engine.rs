use intertrait::cast::CastRef;
use pollster::FutureExt;

use crate::system::system_components::collision_component_factory::SystemComponentCollisionFactory;
use crate::system::system_components::gameplay_component_factory::SystemComponentGameplayFactory;
use crate::system::system_components::gameplay_components::gameplay_component_default::{ECSSystem, ECSSystemEventless, EngineCommands};
use crate::system::system_components::graphics_component_factory::SystemComponentGraphicsFactory;
use crate::system::system_components::input_component_factory::SystemComponentInputFactory;
use crate::system::system_components::time_component_factory::SystemComponentTimeFactory;
use crate::system_adapters::adapter_system_gpu::SystemGPU;
use crate::Window::SystemWindow::SystemWindow;

pub trait EventReciever<T> {
    fn recieve(&self, event: T);
}

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

        for system in ecs_system_built_in.iter() {
            let r = system.as_ref().cast::<dyn EventReciever<TGameEvents>>();

            // let maybe_receiver = system.as_any().downcast_ref::<Box<dyn EventReciever>>();
            if let Some(receiver) = r {
                receiver.recieve(EngineCommands::Exit);
            }

            // if let Some(receiver) = system
            //     .as_any()
            //     .downcast_mut::<dyn EventReciever<TGameEvents>>()
            // {
            //     // receiver.handle_event(event);
            // }
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
