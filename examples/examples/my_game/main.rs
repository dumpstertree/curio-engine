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
    dumpster_engine::{DumpsterEngine, WindowLayout},
    system_adapters::adapter_system_gpu::SystemGPU,
};
use pollster::FutureExt;
use system_component_default_gameplay::SystemComponentDefaultGameplay;
use system_component_default_input::SystemComponentDefaultInput;
use system_component_default_physics::SystemComponentDefaultPhysics;
use system_component_default_rendering::SystemComponentDefaultGraphics;
use system_component_default_time::SystemComponentDefaultTime;

fn main() {
    DumpsterEngine::run::<GameEvents>(
        // loop for engine
        SystemGPU::init().block_on(),
        // components
        SystemComponentDefaultTime::new(),
        SystemComponentDefaultInput::new(),
        SystemComponentDefaultGameplay::<GameEvents>::new(),
        SystemComponentDefaultPhysics::new(),
        SystemComponentDefaultGraphics::new(),
        // window settings
        WindowLayout::windowed_1080(),
    );
}
