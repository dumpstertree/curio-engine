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
    pub(crate) mod event_queue;
    pub(crate) mod f32;
    pub(crate) mod game_state;
    pub(crate) mod gizmo;
    pub(crate) mod input_button;
    pub(crate) mod input_cursor;
    pub(crate) mod key_state;
    pub(crate) mod material;
    pub(crate) mod matrix4x4;
    pub(crate) mod quaternion;
    pub(crate) mod vector3;
    pub(crate) mod vector4;
}
mod random;
mod Window {
    pub(crate) mod SystemWindow;
}
mod gameplay {
    pub mod ecs {
        pub mod traits {
            pub(crate) mod ecs_event_reciever;
            pub(crate) mod ecs_system;
        }
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
            pub(crate) mod system_debug_camera;

            pub(crate) mod system_camera_update_state;
            pub(crate) mod system_collider_box_update_state;
            pub(crate) mod system_collider_sphere_update_state;
            pub(crate) mod system_debug_gui_colliders;
            pub(crate) mod system_debug_gui_collision;
            pub(crate) mod system_debug_gui_entity;
            pub(crate) mod system_debug_gui_screen;
            pub(crate) mod system_debug_gui_time;
            pub(crate) mod system_renderer_update_state;
        }
    }
}
mod events {
    pub(crate) mod engine_commands;
}
mod system {
    pub(crate) mod system_game_state;
    pub mod system_game_states {
        pub(crate) mod state_camera;
        pub(crate) mod state_colliders;
        pub(crate) mod state_collision;
        pub(crate) mod state_debug;
        pub(crate) mod state_draw;
        pub(crate) mod state_gizmos;
        pub(crate) mod state_gui;
        pub(crate) mod state_gui_debug;
        pub(crate) mod state_input;
        pub(crate) mod state_screeen;
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
}
pub(crate) mod egui_app_state;
pub(crate) mod egui_tools;

use crate::dumpster_engine::DumpsterEngine;
use crate::dumpster_engine::WindowLayout;
use crate::my_game::game_events::GameEvents;

pub fn run() {
    // run the engine
    DumpsterEngine::run::<GameEvents>(WindowLayout::windowed_1080());
}
