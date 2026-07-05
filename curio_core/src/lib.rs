// math
pub use crate::math::color::Color;
pub use crate::math::frustrum::Frustrum;
pub use crate::math::matrix4x4::Matrix4x4;
pub use crate::math::quaternion::Quaternion;
pub use crate::math::random::Random;
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

// io
pub use crate::io::asset_cache::AssetCache;
pub use crate::io::asset_database::AssetDatabase;
pub use crate::io::asset_database_listing::AssetDatabaseListing;
pub use crate::io::asset_pipeline::AssetPipeline;
pub use crate::io::asset_pipeline::ASSET_UID_FONT_ASSET_DEFAULT;
pub use crate::io::asset_pipeline::ASSET_UID_SHADER_LIT;
pub use crate::io::asset_pipeline::ASSET_UID_SHADER_UNLIT;
pub use crate::io::asset_pipeline::ASSET_UID_TEXTURE_DEFAULT;
pub use crate::io::asset_pipeline::ASSET_UID_TEXTURE_FONT_ATLAS;
pub use crate::io::assets::Assets;
pub use crate::io::cache_metadata::CacheMetadata;
pub use crate::io::file::File;
pub use crate::io::logger::Logger;
pub use crate::io::severity::Severity;

// assets
pub use crate::assets::asset_common::AssetCommon;
pub use crate::assets::composition::Composition;
pub use crate::assets::composition_facet::CompositionFacet;
pub use crate::assets::texture_asset::TextureAsset;

// input
pub use crate::input::axis_code::AxisCode;
pub use crate::input::axis_state::AxisState;
pub use crate::input::button_code::ButtonCode;
pub use crate::input::button_pressed::ButtonPressed;
pub use crate::input::button_state::ButtonState;
pub use crate::input::input_mapped::InputMapped;
pub use crate::input::input_raw::InputRaw;

// graphics
pub use crate::graphics::gpu::Gpu;
pub use crate::graphics::shaders::Shaders;

// engine
pub use crate::engine::as_any::AsAny;
pub use crate::engine::commands::Commands;
pub use crate::engine::curio::Curio;
pub use crate::engine::curio_builder::CurioBuilder;
pub use crate::engine::curio_common::CurioCommon;
pub use crate::engine::curio_network::CurioNetwork;
pub use crate::engine::curio_network_participant::CurioNetworkParticipant;
pub use crate::engine::impulse_clone::ImpulseClone;
pub use crate::engine::impulse_common::ImpulseCommon;
pub use crate::engine::impulse_network_capabilities::ImpulseNetworkCapabilities;
pub use crate::engine::impulse_scope::ImpulseScope;
pub use crate::engine::impulse_synchronizer::ImpulseSynchronizer;
pub use crate::engine::ledger::Ledger;
pub use crate::engine::ledger_entry::LedgerEntry;
pub use crate::engine::metadata::formation::Formation;
pub use crate::engine::metadata::graphics_mapping::GraphicsMapping;
pub use crate::engine::metadata::identity::Identity;
pub use crate::engine::metadata::input_mapping::InputMapping;
pub use crate::engine::metadata::portal::Portal;
pub use crate::engine::metadata::seat::Seat;
pub use crate::engine::metadata::version::Version;
pub use crate::engine::nerve::Nerve;
pub use crate::engine::record_clone::RecordClone;
pub use crate::engine::record_common::RecordCommon;
pub use crate::engine::record_network_capabilities::RecordNetworkCapabilities;
pub use crate::engine::record_override::RecordOverride;
pub use crate::engine::record_scope::RecordScope;
pub use crate::engine::record_synchronizer::RecordSynchronizer;
pub use crate::engine::serialization::plugin_group_state::ComponentState;
pub use crate::engine::serialization::plugin_group_state::FieldState;
pub use crate::engine::serialization::plugin_group_state::ObjectState;
pub use crate::engine::serialization::plugin_group_state::PluginGroupState;
pub use crate::engine::serialization::plugin_group_state::PluginState;
pub use crate::engine::services::Services;

// network
pub use crate::network::network_modes::NetworkModes;

// extensions
pub use crate::extensions::extensions_f32::ExtensionsF32;
pub use crate::extensions::extensions_f64::ExtensionsF64;
pub use crate::extensions::extensions_i32::ExtensionsI32;

//system
pub use crate::system::record_id::RecordId;
pub use crate::system::system_component::SystemComponent;

//global
pub use crate::static_data::global_impulses::GlobalImpulses;
pub use crate::static_data::global_records::GlobalRecords;
pub use crate::static_data::impulse_registration::ImpulseRegistration;
pub use crate::static_data::record_registration::RecordRegistration;

//
pub mod graphics {
    pub(crate) mod gpu;
    pub(crate) mod shaders;
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
    pub(crate) mod random;
    pub(crate) mod vector2;
    pub(crate) mod vector2_int;
    pub(crate) mod vector3;
    pub(crate) mod vector3_int;
    pub(crate) mod vector4;
    pub(crate) mod vector4_int;
}
pub mod engine {
    pub mod c_bindings {
        mod peek_curio;
    }
    pub mod metadata {
        pub(crate) mod formation;
        pub(crate) mod graphics_mapping;
        pub(crate) mod identity;
        pub(crate) mod input_mapping;
        pub(crate) mod portal;
        pub(crate) mod seat;
        pub(crate) mod version;
    }
    pub mod serialization {
        pub(crate) mod plugin_group_state;
    }
    pub(crate) mod as_any;
    pub(crate) mod commands;
    pub(crate) mod curio;
    pub(crate) mod curio_builder;
    pub(crate) mod curio_common;
    pub(crate) mod curio_network;
    pub(crate) mod curio_network_participant;
    pub(crate) mod impulse_clone;
    pub(crate) mod impulse_common;
    pub(crate) mod impulse_network_capabilities;
    pub(crate) mod impulse_scope;
    pub(crate) mod impulse_synchronizer;
    pub(crate) mod ledger;
    pub(crate) mod ledger_entry;
    pub(crate) mod nerve;
    pub(crate) mod record_clone;
    pub(crate) mod record_common;
    pub(crate) mod record_network_capabilities;
    pub(crate) mod record_override;
    pub(crate) mod record_scope;
    pub(crate) mod record_synchronizer;
    pub(crate) mod services;
}
pub mod extensions {
    pub(crate) mod extensions_f32;
    pub(crate) mod extensions_f64;
    pub(crate) mod extensions_i32;
}
pub mod assets {
    pub(crate) mod asset_common;
    pub(crate) mod composition;
    pub(crate) mod composition_facet;
    pub(crate) mod texture_asset;
}
pub mod io {
    pub(crate) mod asset_cache;
    pub(crate) mod asset_database;
    pub(crate) mod asset_database_listing;
    pub(crate) mod asset_pipeline;
    pub(crate) mod assets;
    pub(crate) mod cache_metadata;
    pub(crate) mod file;
    pub(crate) mod logger;
    pub(crate) mod severity;
}
pub mod collections {
    pub(crate) mod any_map;
    pub(crate) mod any_queue;
    pub(crate) mod event_runner;
}
pub mod network {
    pub(crate) mod network_modes;
}
pub mod static_data {
    pub(crate) mod global_impulses;
    pub(crate) mod global_records;
    pub(crate) mod impulse_registration;
    pub(crate) mod record_registration;
}
pub mod system {
    pub(crate) mod record_id;
    pub(crate) mod system_component;
}
pub mod built_in {
    pub mod record {
        pub mod sys_record_debug;
        pub mod sys_record_debug_gui;
        pub mod sys_record_gui;
        pub mod sys_record_input;
        pub mod sys_record_network;
        pub mod sys_record_screen;
        pub mod sys_record_time;
    }
}
