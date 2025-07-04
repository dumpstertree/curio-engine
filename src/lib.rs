mod dumpster_engine;
mod game_state;
mod texture;
mod IO {
    pub(crate) mod Asset;
    pub(crate) mod AssetLoader;
    pub(crate) mod model_asset;
    pub(crate) mod texture_asset;
}
mod Collections {
    pub(crate) mod Color;
    pub(crate) mod DrawCall;
    pub(crate) mod GraphicsBufferCache;
    pub(crate) mod Mesh;
    pub(crate) mod key_state;
    pub(crate) mod material;
    pub(crate) mod matrix4x4;
    pub(crate) mod vector3;
}

mod Window {
    pub(crate) mod CameraState;
    pub(crate) mod SystemWindow;
    pub(crate) mod state;
}
mod gameplay {
    pub mod ecs {
        pub mod component {
            pub(crate) mod component_camera;
            pub(crate) mod component_renderer;
            pub(crate) mod component_transform;
        }
        pub mod system {
            pub(crate) mod system_camera_fps;
            pub(crate) mod system_camera_update_state;
            pub(crate) mod system_renderer_update_state;
        }
    }
    pub(crate) mod game_events;
}
mod system {
    pub(crate) mod system_component;
    pub mod system_components {
        pub(crate) mod gameplay_component;
        pub mod gameplay_components {
            pub(crate) mod gameplay_component_default;
        }
        pub(crate) mod time_component;
        pub mod time_components {
            pub(crate) mod time_component;
        }
        pub(crate) mod graphics_component;
        pub mod graphics_components {
            pub(crate) mod graphics_component_wgpu;
        }
        pub(crate) mod input_component;
        pub mod input_components {
            pub(crate) mod input_component_default;
        }
    }
}
mod system_adapters {
    pub(crate) mod adapter_system_gpu;
}

use crate::{
    dumpster_engine::DumpsterEngine,
    gameplay::ecs::system::{
        system_camera_fps::FPSCameraECSSystem, system_camera_update_state::PostCameraECSSystem, system_renderer_update_state::TestECSSystem,
    },
};

use crate::Window::state;

pub fn run() {
    // run the engine
    DumpsterEngine::run(vec![
        Box::new(PostCameraECSSystem {}),
        Box::new(FPSCameraECSSystem {}),
        Box::new(TestECSSystem::new()),
    ]);
}
