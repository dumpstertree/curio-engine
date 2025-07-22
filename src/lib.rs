mod dumpster_engine;
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
    pub(crate) mod camera_uniform;
    pub(crate) mod game_state;
    pub(crate) mod gizmo;
    pub(crate) mod input_button;
    pub(crate) mod key_state;
    pub(crate) mod material;
    pub(crate) mod matrix4x4;
    pub(crate) mod quaternion;
    pub(crate) mod vector3;
}
mod random;
mod Window {
    pub(crate) mod SystemWindow;
}
mod gameplay {
    pub mod ecs {
        pub mod component {
            pub(crate) mod component_camera;
            pub(crate) mod component_collider;
            pub(crate) mod component_renderer;
            pub(crate) mod component_transform;
            pub mod component_colliders {
                pub(crate) mod component_collider_box;
                pub(crate) mod component_collider_sphere;
            }
        }
        pub mod system {
            pub(crate) mod system_camera_fps;
            pub(crate) mod system_camera_update_state;
            pub(crate) mod system_collider_box_update_state;
            pub(crate) mod system_collider_sphere_update_state;
            pub(crate) mod system_renderer_update_state;
        }
    }
    pub(crate) mod game_events;
}
mod system {
    pub(crate) mod system_game_state;
    pub mod system_game_states {
        pub(crate) mod state_camera;
        pub(crate) mod state_colliders;
        pub(crate) mod state_collision;
        pub(crate) mod state_draw;
        pub(crate) mod state_gizmos;
        pub(crate) mod state_input;
        pub(crate) mod state_time;
    }
    pub(crate) mod system_component;
    pub mod system_components {
        pub(crate) mod gameplay_component;
        pub(crate) mod gameplay_component_factory;
        pub mod gameplay_components {
            pub(crate) mod gameplay_component_default;
        }
        pub(crate) mod time_component;
        pub(crate) mod time_component_factory;
        pub mod time_components {
            pub(crate) mod time_component;
        }
        pub(crate) mod graphics_component;
        pub(crate) mod graphics_component_factory;
        pub mod graphics_components {
            pub(crate) mod graphics_component_wgpu;
        }
        pub(crate) mod input_component;
        pub(crate) mod input_component_factory;
        pub mod input_components {
            pub(crate) mod input_component_default;
        }
        pub(crate) mod collision_component;
        pub(crate) mod collision_component_factory;
        pub mod collision_components {
            pub(crate) mod collision_component_default;
        }
    }
}
mod system_adapters {
    pub(crate) mod adapter_system_gpu;
}
mod my_game {
    pub mod ecs {
        pub mod system {
            pub(crate) mod system_ball_move;
            pub(crate) mod system_engine_commands;
            pub(crate) mod system_game_init;
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
}

use crate::dumpster_engine::DumpsterEngine;
use crate::dumpster_engine::WindowLayout;
use crate::my_game::ecs::system::system_ball_move::SystemBallMove;
use crate::my_game::ecs::system::system_paddle_move::SystemPaddleMove;
use crate::my_game::ecs::system::system_pong_init::SystemPongInit;

pub fn run() {
    // run the engine
    DumpsterEngine::run(
        WindowLayout::windowed_1080(),
        // vec![SystemGameInit::new(), SystemSpin::new(), SystemEngineCommands::new()],
        vec![SystemPongInit::new(), SystemPaddleMove::new(), SystemBallMove::new()],
    );
}
