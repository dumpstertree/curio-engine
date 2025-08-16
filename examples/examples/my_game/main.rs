use crate::game_events::GameEvents;

pub(crate) mod constants;
pub(crate) mod game_events;
pub mod ecs {
    pub mod system {
        pub(crate) mod system_ball_move;
        pub(crate) mod system_paddle_move;
        pub(crate) mod system_pong_init;
        pub(crate) mod system_spin;
    }
    pub mod component {
        pub(crate) mod component_ball;
        pub(crate) mod component_paddle;
        pub(crate) mod component_spin;
    }
}

use core::{
    collections::vector2::Vector2,
    dumpster_engine::{DumpsterEngine, GameMode, WindowLayout},
    graphics::graphics_mapping::GraphicsMapping,
    input::{input_mapping::InputMapping, key_code::KeyCode},
    system_adapters::adapter_system_gpu::SystemGPU,
};
use pollster::FutureExt;
use system_component_default_gameplay::SystemComponentDefaultGameplay;
use system_component_default_input::SystemComponentDefaultInput;
use system_component_default_physics::SystemComponentDefaultPhysics;
use system_component_default_rendering::SystemComponentDefaultGraphics;
use system_component_default_time::SystemComponentDefaultTime;
fn main() {
    let input_mapping_0 = InputMapping::new(
        vec![
            (String::from("move_forward"), KeyCode::KeyW),
            (String::from("move_back"), KeyCode::KeyS),
            (String::from("move_left"), KeyCode::KeyA),
            (String::from("move_right"), KeyCode::KeyD),
        ],
        vec![],
    );
    let input_mapping_1 = InputMapping::new(
        vec![
            (String::from("move_forward"), KeyCode::KeyI),
            (String::from("move_back"), KeyCode::KeyK),
            (String::from("move_left"), KeyCode::KeyJ),
            (String::from("move_right"), KeyCode::KeyL),
        ],
        vec![],
    );
    let graphics_mapping_0 = GraphicsMapping::new(Vector2::new(0.0, 0.0), Vector2::new(0.5, 1.0));
    let graphics_mapping_1 = GraphicsMapping::new(Vector2::new(0.5, 0.0), Vector2::new(1.0, 1.0));
    DumpsterEngine::run::<GameEvents>(
        //
        // loop for engine
        SystemGPU::init().block_on(),
        //
        // components
        SystemComponentDefaultTime::new(),
        SystemComponentDefaultInput::new(),
        SystemComponentDefaultGameplay::<GameEvents>::new(),
        SystemComponentDefaultPhysics::new(),
        SystemComponentDefaultGraphics::new(),
        //
        // window settings
        WindowLayout::windowed_1080(),
        //

        // create game states
        GameMode::new(vec![input_mapping_0, input_mapping_1], vec![graphics_mapping_0, graphics_mapping_1]),
    );
}
