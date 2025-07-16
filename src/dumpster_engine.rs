use pollster::FutureExt;

use crate::gameplay::ecs::system::system_camera_fps::FPSCameraECSSystem;
use crate::gameplay::ecs::system::system_camera_update_state::PostCameraECSSystem;
use crate::gameplay::ecs::system::system_collider_box_update_state::SystemColliderSphereUpdateState;
use crate::gameplay::ecs::system::system_collider_sphere_update_state::SystemColliderBoxUpdateState;
use crate::gameplay::ecs::system::system_renderer_update_state::TestECSSystem;
use crate::system::system_components::collision_component_factory::SystemComponentCollisionFactory;
use crate::system::system_components::gameplay_component_factory::SystemComponentGameplayFactory;
use crate::system::system_components::gameplay_components::gameplay_component_default::{ECSSystem, ECSSystemEventless};
use crate::system::system_components::graphics_component_factory::SystemComponentGraphicsFactory;
use crate::system::system_components::input_component_factory::SystemComponentInputFactory;
use crate::system::system_components::time_component_factory::SystemComponentTimeFactory;
use crate::system_adapters::adapter_system_gpu::{SystemGPU, SYS_GPU};
use crate::Window::SystemWindow::SystemWindow;

pub struct DumpsterEngine {}
impl DumpsterEngine {
    pub fn run<TGameEvents>(window_layout: WindowLayout, ecs_systems: Vec<Box<dyn ECSSystem<TGameEvents>>>)
    where
        TGameEvents: 'static,
        TGameEvents: Clone,
    {
        let mut guard_sys_gpu = SYS_GPU.lock().unwrap();
        let event_loop = guard_sys_gpu.init().block_on();
        drop(guard_sys_gpu);

        // set window layout values
        SystemGPU::set_resolution(window_layout.width, window_layout.height);
        SystemGPU::set_fullscreen(window_layout.fullscreen);
        SystemGPU::set_resizable(window_layout.resizeable);
        SystemGPU::set_cursor_visible(window_layout.show_cursor);

        // create built in systems
        let ecs_system_built_in: Vec<Box<dyn ECSSystemEventless>> = vec![
            Box::new(PostCameraECSSystem {}),
            Box::new(FPSCameraECSSystem::new()),
            TestECSSystem::new(),
            SystemColliderBoxUpdateState::new(),
            SystemColliderSphereUpdateState::new(),
        ];
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
