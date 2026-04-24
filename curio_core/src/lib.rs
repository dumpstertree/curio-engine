// math
pub use crate::math::color::Color;
pub use crate::math::matrix4x4::Matrix4x4;
pub use crate::math::quaternion::Quaternion;
pub use crate::math::vector2::Vector2;
pub use crate::math::vector2_int::Vector2Int;
pub use crate::math::vector3::Vector3;
pub use crate::math::vector3_int::Vector3Int;
pub use crate::math::vector4::Vector4;
pub use crate::math::vector4_int::Vector4Int;

// utils
pub use crate::log::log;
pub use crate::log::Severity;
pub use crate::random::Random;

// assets
pub use crate::assets::asset::AssetCommon;
pub use crate::assets::font_asset::FontAsset;
pub use crate::assets::font_asset::FontDesc;
pub use crate::assets::model_asset::ModelAsset;
pub use crate::assets::prefab_asset::PrefabGameObject;
pub use crate::assets::texture_asset::TextureAsset;
pub use crate::graphics::material::ShaderDesc;

// input
pub use crate::input::axis_code::AxisCode;
pub use crate::input::input_button::InputButtonState;
pub use crate::input::input_cursor::InputAxisState;
pub use crate::input::input_mapping::InputMapping;
pub use crate::input::input_snapshot_mapped::PlayerInputSnapshot;
pub use crate::input::input_snapshot_raw::RawInputSnapshot;
pub use crate::input::key_code::ButtonCode;

// graphics
pub use crate::graphics::draw_call::DrawCall;
pub use crate::graphics::gizmo::Gizmo;
pub use crate::graphics::gpu_instance::GPUInstance;
pub use crate::graphics::graphics_mapping::GraphicsMapping;
pub use crate::graphics::light_uniform::DrawCallLight;
pub use crate::graphics::light_uniform::LightSystem;
pub use crate::graphics::light_uniform::LightType;
pub use crate::graphics::material::Material;
pub use crate::graphics::mesh::Mesh;
pub use crate::graphics::mesh::Vertex;

//
pub mod graphics {
    pub(crate) mod draw_call;
    pub(crate) mod gizmo;
    pub(crate) mod gpu_instance;
    pub(crate) mod graphics_mapping;
    pub(crate) mod light_uniform;
    pub(crate) mod material;
    pub(crate) mod mesh;
}
pub mod input {
    pub(crate) mod axis_code;
    pub(crate) mod input_button;
    pub(crate) mod input_cursor;
    pub(crate) mod input_mapping;
    pub(crate) mod input_snapshot_mapped;
    pub(crate) mod input_snapshot_raw;
    pub(crate) mod key_code;
}
pub mod math {
    pub(crate) mod color;
    pub(crate) mod matrix4x4;
    pub(crate) mod quaternion;
    pub(crate) mod vector2;
    pub(crate) mod vector2_int;
    pub(crate) mod vector3;
    pub(crate) mod vector3_int;
    pub(crate) mod vector4;
    pub(crate) mod vector4_int;
}
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

pub mod extensions {
    pub mod extensions_f32;
    pub mod extensions_f64;
    pub mod extensions_i32;
}
pub mod assets {
    pub(crate) mod asset;
    pub(crate) mod font_asset;
    pub(crate) mod model_asset;
    pub(crate) mod prefab_asset;
    pub(crate) mod texture_asset;
}
pub mod io {
    pub mod asset_cache;
    pub mod asset_database;
    pub mod asset_loader;
    pub mod file;

    pub mod model_asset_animated;
}

pub mod collections {
    pub mod any_map;
    pub mod camera_uniform;
    pub mod curio_metadata;
    pub mod event_queue;
    pub mod event_runner;
    pub mod f32;
    pub mod game_instance;
    pub mod game_mode;
    pub mod game_state;
    pub mod key_state;
    pub mod network_capabilities;
    pub mod network_modes;
    pub mod projection;
    pub mod state_map;
    pub mod state_ownerships;
    pub mod state_sync_event;
    pub mod version_number;
    pub mod window_layout;
}
mod log;
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
