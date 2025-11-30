pub mod dumpster_engine;

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
    pub mod draw_call;
    pub mod event_queue;
    pub mod event_runner;
    pub mod f32;
    pub mod game_state;
    pub mod gizmo;
    pub mod input_button;
    pub mod input_cursor;
    pub mod key_state;
    pub mod light_uniform;
    pub mod material;
    pub mod matrix4x4;
    pub mod mesh;
    pub mod network_capabilities;
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
}
pub mod prefab;
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
pub mod static_data {
    pub mod global_ecs;
    pub mod global_event_recievers;
    pub mod global_events;
    pub mod global_states;
}
pub mod events {
    pub mod engine_commands;
}
pub mod system {
    pub mod system_game_state;
    // pub mod system_game_states {
    //     pub mod state_camera;
    //     pub mod state_colliders;
    //     pub mod state_collision;
    //     pub mod state_debug;
    //     pub mod state_draw;
    //     pub mod state_gizmos;
    //     pub mod state_gui;
    //     pub mod state_gui_debug;
    //     pub mod state_input;
    //     pub mod state_screeen;
    //     pub mod state_time;
    // }
    pub mod system_component;
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

// pub fn main() {}

use message_io::network::{NetEvent, Transport};
use message_io::node::{self};

pub fn main() {
    // Create a node, the main message-io entity. It is divided in 2 parts:
    // The 'handler', used to make actions (connect, send messages, signals, stop the node...)
    // The 'listener', used to read events from the network or signals.
    let (handler, listener) = node::split::<()>();

    // Listen for TCP, UDP and WebSocket messages at the same time.
    handler
        .network()
        .listen(Transport::FramedTcp, "0.0.0.0:3042")
        .unwrap();
    handler
        .network()
        .listen(Transport::Udp, "0.0.0.0:3043")
        .unwrap();
    handler
        .network()
        .listen(Transport::Ws, "0.0.0.0:3044")
        .unwrap();

    // Read incoming network events.
    listener.for_each(move |event| match event.network() {
        NetEvent::Connected(_, _) => unreachable!(),                              // Used for explicit connections.
        NetEvent::Accepted(_endpoint, _listener) => println!("Client connected"), // Tcp or Ws
        NetEvent::Message(endpoint, data) => {
            println!("Received: {}", String::from_utf8_lossy(data));
            handler.network().send(endpoint, data);
        }
        NetEvent::Disconnected(_endpoint) => println!("Client disconnected"), //Tcp or Ws
    });
}
