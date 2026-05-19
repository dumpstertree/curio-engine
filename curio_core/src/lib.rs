// application
pub use crate::application::Application;

// math
pub use crate::math::color::Color;
pub use crate::math::frustrum::Frustrum;
pub use crate::math::matrix4x4::Matrix4x4;
pub use crate::math::quaternion::Quaternion;
pub use crate::math::vector2::Vector2;
pub use crate::math::vector2_int::Vector2Int;
pub use crate::math::vector3::Vector3;
pub use crate::math::vector3_int::Vector3Int;
pub use crate::math::vector4::Vector4;
pub use crate::math::vector4_int::Vector4Int;

// collections
pub use crate::collections::any_map::AnyMap;
pub use crate::collections::any_queue::AnyQueue;
pub use crate::collections::event_runner::EventRunner;
pub use crate::collections::ledger::Ledger;

// utils
pub use crate::log::log;
pub use crate::log::Severity;
pub use crate::random::Random;

// assets
pub use crate::assets::asset::AssetCommon;
pub use crate::assets::asset::AssetCommonFromBits;
pub use crate::assets::font_asset::FontAsset;
pub use crate::assets::font_asset::FontDesc;
pub use crate::assets::model_asset::ModelAsset;
pub use crate::assets::prefab_asset::PrefabGameObject;
pub use crate::assets::texture_asset::TextureAsset;
pub use crate::graphics::material::ShaderDesc;

// input
pub use crate::input::axis_code::AxisCode;
pub use crate::input::axis_state::AxisState;
pub use crate::input::button_code::ButtonCode;
pub use crate::input::button_pressed::ButtonPressed;
pub use crate::input::button_state::ButtonState;
pub use crate::input::input_mapped::InputMapped;
pub use crate::input::input_raw::InputRaw;

// graphics
pub use crate::graphics::camera_uniform::CameraSnapshot;
pub use crate::graphics::camera_uniform::CameraUniform;
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

// engine
pub use crate::engine::curio::load_curio;
pub use crate::engine::curio::ComponentState;
pub use crate::engine::curio::Curio;
pub use crate::engine::curio::FieldState;
pub use crate::engine::curio::FormsSnapshot;
pub use crate::engine::curio::LedgerSnapshot;
pub use crate::engine::curio::LoadedCurio;
pub use crate::engine::curio::ObjectState;
pub use crate::engine::curio::TabGroupState;
pub use crate::engine::curio::TabState;
pub use crate::engine::curio_common::CurioCommon;
pub use crate::engine::curio_metadata::CurioMetadata;
pub use crate::engine::event_sync_event::EventSyncEvent;
pub use crate::engine::formation::Formation;
pub use crate::engine::ievent_clone::IEventClone;
pub use crate::engine::igame_event::IGameEvent;
pub use crate::engine::input_mapping::InputMapping;
pub use crate::engine::nerve::AsAny;
pub use crate::engine::nerve::EventScope;
pub use crate::engine::nerve::Nerve;
pub use crate::engine::portal::Portal;
pub use crate::engine::seat::Seat;
pub use crate::engine::version::Version;

// network
pub use crate::network::event_network_capabilities::EventNetworkCapabilities;
pub use crate::network::network_modes::NetworkModes;
pub use crate::network::state_network_capabilities::StateNetworkCapabilities;
pub use crate::network::state_ownerships::StateOwnerships;
pub use crate::network::state_sync_event::StateSyncEvent;

// extensions
pub use crate::extensions::extensions_f32::ExtensionsF32;
pub use crate::extensions::extensions_f64::ExtensionsF64;
pub use crate::extensions::extensions_i32::ExtensionsI32;

//system
pub use crate::system::record_common::RecordCommon;
pub use crate::system::record_common_clone::RecordCommonClone;
pub use crate::system::record_id::RecordId;
pub use crate::system::system_component::SystemComponent;

//
pub mod graphics {
    pub(crate) mod camera_uniform;
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
    pub(crate) mod axis_state;
    pub(crate) mod button_code;
    pub(crate) mod button_pressed;
    pub(crate) mod button_state;
    pub(crate) mod input_mapped;
    pub(crate) mod input_raw;
}
pub mod math {
    pub(crate) mod color;
    pub(crate) mod frustrum;
    pub(crate) mod matrix4x4;
    pub(crate) mod quaternion;
    pub(crate) mod vector2;
    pub(crate) mod vector2_int;
    pub(crate) mod vector3;
    pub(crate) mod vector3_int;
    pub(crate) mod vector4;
    pub(crate) mod vector4_int;
}

pub mod engine {
    pub(crate) mod curio;
    pub(crate) mod curio_common;
    pub(crate) mod curio_metadata;
    pub(crate) mod event_sync_event;
    pub(crate) mod formation;
    pub(crate) mod ievent_clone;
    pub(crate) mod igame_event;
    pub(crate) mod input_mapping;
    pub(crate) mod iui_event;
    pub(crate) mod nerve;
    pub(crate) mod portal;
    pub(crate) mod seat;
    pub(crate) mod version;
}

pub mod extensions {
    pub(crate) mod extensions_f32;
    pub(crate) mod extensions_f64;
    pub(crate) mod extensions_i32;
    pub(crate) mod f32;
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
}
pub mod collections {
    pub(crate) mod any_map;
    pub(crate) mod any_queue;
    pub(crate) mod event_runner;
    pub(crate) mod ledger;
}
pub mod network {
    pub(crate) mod event_network_capabilities;
    pub(crate) mod network_modes;
    pub(crate) mod state_network_capabilities;
    pub(crate) mod state_ownerships;
    pub(crate) mod state_sync_event;
}
pub(crate) mod application;
pub(crate) mod log;
pub(crate) mod random;

pub mod static_data {
    pub(crate) mod global_events;
    pub(crate) mod global_states;
}
pub mod system {
    pub(crate) mod record_common;
    pub(crate) mod record_common_clone;
    pub(crate) mod record_id;
    pub(crate) mod system_component;
}
pub mod system_adapters {
    pub mod adapter_system_gpu;
}
pub mod engine_services;
pub mod plugin_loader;

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
