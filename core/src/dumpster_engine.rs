use std::any::type_name;
use std::fmt;
use std::sync::Mutex;

use egui_wgpu::wgpu::naga::Type;
use winit::event_loop::EventLoop;

use crate::collections::vector2::Vector2;
use crate::events::engine_commands::EngineCommands;
use crate::gameplay::ecs::traits::ecs_system::ECSSystemEventless;
use crate::graphics::graphics_mapping::GraphicsMapping;
use crate::input::input_mapping::InputMapping;
use crate::system::system_components::system_component_gameplay::SystemComponentGameplay;
use crate::system::system_components::system_component_networking::SystemComponentNetworking;
use crate::system::system_components::system_component_physics::SystemComponentPhysics;
use crate::system::system_components::system_component_time::SystemComponentTime;
use crate::system::system_components::{system_component_graphics, system_component_input};
use crate::system::system_game_state::IState;
use crate::system_adapters::adapter_system_gpu::SystemGPU;
use crate::window::system_window::SystemWindow;

static REGISTERED_GLOBAL_ECS_SYSTEMS: Mutex<Vec<fn() -> Box<dyn ECSSystemEventless>>> = Mutex::new(Vec::new());
static REGISTERED_GLOBAL_STATES: Mutex<Vec<Type>> = Mutex::new(Vec::new());

#[derive(Clone, PartialEq)]
pub enum NetworkModes {
    LocalHost,
    LocalPeer,
    OnlineHost,
    OnlinePeer,
}
impl fmt::Display for NetworkModes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NetworkModes::LocalHost => write!(f, "LocalHost"),
            NetworkModes::LocalPeer => write!(f, "LocalPeer"),
            NetworkModes::OnlineHost => write!(f, "OnlineHost"),
            NetworkModes::OnlinePeer => write!(f, "OnlinePeer"),
        }
    }
}
pub struct GameMode {
    // pub input_mappings: Vec<InputMapping>,
    // pub graphics_mappings: Vec<GraphicsMapping>,
    // pub network_mode: NetworkModes,
    pub game_instances: Vec<GameInstance>,
}
impl GameMode {
    // pub fn new(input_mappings: Vec<InputMapping>, graphics_mappings: Vec<GraphicsMapping>, network_mode: NetworkModes) -> GameMode {
    //     GameMode { input_mappings, graphics_mappings, network_mode }
    // }
    pub fn new(game_instances: Vec<GameInstance>) -> GameMode {
        GameMode { game_instances }
    }

    pub fn new_local_single(input: InputMapping) -> GameMode {
        GameMode {
            game_instances: vec![GameInstance::new(GraphicsMapping::new(Vector2::zero(), Vector2::one()), vec![input], NetworkModes::LocalHost)],
        }
    }
    pub fn new_local_splitscreen_2p_vertical(input_p1: InputMapping, input_p2: InputMapping) -> GameMode {
        GameMode {
            game_instances: vec![
                GameInstance::new(GraphicsMapping::new(Vector2::new(0.0, 0.0), Vector2::new(0.5, 1.0)), vec![input_p1], NetworkModes::LocalHost),
                GameInstance::new(GraphicsMapping::new(Vector2::new(0.5, 0.0), Vector2::new(1.0, 1.0)), vec![input_p2], NetworkModes::LocalPeer),
            ],
        }
    }
    pub fn new_local_splitscreen_2p_horizontal(input_p1: InputMapping, input_p2: InputMapping) -> GameMode {
        GameMode {
            game_instances: vec![
                GameInstance::new(GraphicsMapping::new(Vector2::new(0.0, 0.0), Vector2::new(1.0, 0.5)), vec![input_p1], NetworkModes::LocalPeer),
                GameInstance::new(GraphicsMapping::new(Vector2::new(0.0, 0.5), Vector2::new(1.0, 1.0)), vec![input_p2], NetworkModes::LocalPeer),
                GameInstance::new(GraphicsMapping::new(Vector2::new(0.9, 0.9), Vector2::new(1.0, 1.0)), vec![], NetworkModes::LocalHost),
            ],
        }
    }
}

pub struct GameInstance {
    pub graphics_mappings: GraphicsMapping,
    pub input_mappings: Vec<InputMapping>,
    pub network_mode: NetworkModes,
}
impl GameInstance {
    pub fn new(graphics_mappings: GraphicsMapping, input_mappings: Vec<InputMapping>, network_mode: NetworkModes) -> GameInstance {
        GameInstance { graphics_mappings, input_mappings, network_mode }
    }
}
pub struct DumpsterEngine {}
impl DumpsterEngine {
    pub fn register_global_state<T>()
    where
        T: 'static + IState,
    {
        let type_id = type_name::<T>();
        println!("register {}", type_id);

        let Ok(guard) = REGISTERED_GLOBAL_STATES.lock() else {
            println!("failed to lock REGISTERED_STATE");
            return;
        };

        // guard.push(T);
    }
    pub fn register_global_ecs_system<T>()
    where
        T: 'static + ECSSystemEventless + Default + Clone,
    {
        let type_id = type_name::<T>();
        println!("register {}", type_id);

        let Ok(mut guard) = REGISTERED_GLOBAL_ECS_SYSTEMS.lock() else {
            println!("failed to lock REGISTERED_ECS_SYSTEMS");
            return;
        };

        let callback: fn() -> Box<dyn ECSSystemEventless> = || return Box::new(T::default());
        guard.push(callback);
    }
    pub fn run<TGameEvents>(
        event_loop: EventLoop<EngineCommands>,
        time: Box<dyn SystemComponentTime>,
        input: Box<dyn system_component_input::SystemComponentInput>,
        mut gameplay: Box<dyn SystemComponentGameplay>,
        physics: Box<dyn SystemComponentPhysics>,
        graphics: Box<dyn system_component_graphics::SystemComponentGraphics>,
        networking: Box<dyn SystemComponentNetworking>,
        window_layout: WindowLayout,
        game_modes: GameMode,
    ) where
        TGameEvents: 'static + Clone,
    {
        // set window layout values
        SystemGPU::set_resolution(window_layout.width, window_layout.height);
        SystemGPU::set_fullscreen(window_layout.fullscreen);
        SystemGPU::set_resizable(window_layout.resizeable);
        SystemGPU::set_cursor_visible(window_layout.show_cursor);

        // create built in systems
        let mut ecs_system_built_in: Vec<Box<dyn ECSSystemEventless>> = vec![];

        let mut ecs_system_built_in_constructors: Vec<fn() -> Box<dyn ECSSystemEventless>> = vec![];

        let Ok(guard) = REGISTERED_GLOBAL_ECS_SYSTEMS.lock() else {
            println!("failed to lock REGISTERED_ECS_SYSTEMS");
            return;
        };
        for x in guard.iter() {
            ecs_system_built_in.push(x());
            ecs_system_built_in_constructors.push(x.clone());
        }

        gameplay.set_systems(ecs_system_built_in_constructors);

        // create systems
        let mut system_window = SystemWindow::new(vec![time, input, gameplay, physics, graphics, networking], game_modes);

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
