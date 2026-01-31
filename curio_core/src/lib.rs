pub mod built_in {
    pub mod stimulant {
        pub mod engine_commands;
    }
    pub mod facet {
        pub mod component_collider;
    }
    pub mod record {
        pub mod sys_record_camera;
        pub mod sys_record_colliders;
        pub mod sys_record_collision;
        pub mod sys_record_debug;
        pub mod sys_record_debug_gui;
        pub mod sys_record_gizmos;
        pub mod sys_record_gui;
        pub mod sys_record_input;
        pub mod sys_record_lights;
        pub mod sys_record_network;
        pub mod sys_record_rendering;
        pub mod sys_record_screen;
        pub mod sys_record_skybox;
        pub mod sys_record_sun;
        pub mod sys_record_time;
    }
}
pub mod engine {
    pub mod curio;
    pub mod curio_cabinet;
    pub mod curio_common;
}
pub mod graphics {
    pub mod graphics_mapping;
}
pub mod extensions {
    pub mod extensions_f32;
    pub mod extensions_f64;
    pub mod extensions_i32;
}
pub mod input {
    pub mod axis_code;
    pub mod input_mapping;
    pub mod input_snapshot_mapped;
    pub mod input_snapshot_raw;
    pub mod key_code;
}
pub mod io {
    pub mod asset;
    pub mod asset_cache;
    pub mod asset_database;
    pub mod asset_loader;
    pub mod file;
    pub mod font_asset;
    pub mod model_asset;
    pub mod model_asset_animated;
    pub mod texture_asset;
}
pub mod collections {
    pub mod any_map;
    pub mod camera_uniform;
    pub mod color;
    pub mod curio_metadata;
    pub mod draw_call;
    pub mod event_queue;
    pub mod event_runner;
    pub mod f32;
    pub mod game_instance;
    pub mod game_mode;
    pub mod game_state;
    pub mod gizmo;
    pub mod gpu_instance;
    pub mod input_button;
    pub mod input_cursor;
    pub mod key_state;
    pub mod light_uniform;
    pub mod material;
    pub mod matrix4x4;
    pub mod mesh;
    pub mod network_capabilities;
    pub mod network_modes;
    pub mod projection;
    pub mod quaternion;
    pub mod state_map;
    pub mod state_ownerships;
    pub mod state_sync_event;
    pub mod tween;
    pub mod vector2;
    pub mod vector2_int;
    pub mod vector3;
    pub mod vector3_int;
    pub mod vector4;
    pub mod vector4_int;
    pub mod version_number;
    pub mod window_layout;
}
pub mod random;

pub mod static_data {
    pub mod global_events;
    pub mod global_states;
}
pub mod system {
    pub mod system_component;
    pub mod system_game_state;
    pub mod system_components {
        pub mod system_component_gameplay;
        pub mod system_component_graphics;
        pub mod system_component_input;
        pub mod system_component_networking;
        pub mod system_component_physics;
        pub mod system_component_time;
    }
}
pub mod system_adapters {
    pub mod adapter_system_gpu;
}
