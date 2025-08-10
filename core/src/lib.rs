pub mod dumpster_engine;

pub mod io {
    pub mod asset;
    pub mod asset_loader;
    pub mod model_asset;
    pub mod texture_asset;
}
pub mod collections {
    pub mod camera_uniform;
    pub mod color;
    pub mod draw_call;
    pub mod event_queue;
    pub mod f32;
    pub mod game_state;
    pub mod gizmo;
    pub mod input_button;
    pub mod input_cursor;
    pub mod key_state;
    pub mod material;
    pub mod matrix4x4;
    pub mod mesh;
    pub mod projection;
    pub mod quaternion;
    pub mod vector3;
    pub mod vector4;
}
pub mod random;

mod window {
    pub(crate) mod system_window;
}
pub mod gameplay {
    pub mod ecs {
        pub mod traits {
            pub mod ecs_event_reciever;
            pub mod ecs_system;
        }
        pub mod component {
            pub mod component_collider;
        }
    }
}
pub mod events {
    pub mod engine_commands;
}
pub mod system {
    pub mod system_game_state;
    pub mod system_game_states {
        pub mod state_camera;
        pub mod state_colliders;
        pub mod state_collision;
        pub mod state_debug;
        pub mod state_draw;
        pub mod state_gizmos;
        pub mod state_gui;
        pub mod state_gui_debug;
        pub mod state_input;
        pub mod state_screeen;
        pub mod state_time;
    }
    pub mod system_component;
    pub mod system_components {
        pub mod system_component_gameplay;
        pub mod system_component_graphics;
        pub mod system_component_input;
        pub mod system_component_physics;
        pub mod system_component_time;
    }
}
pub mod system_adapters {
    pub mod adapter_system_gpu;
}

pub fn main() {}
