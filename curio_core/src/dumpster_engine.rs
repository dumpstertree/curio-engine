use egui_wgpu::wgpu::{Adapter, Device, Queue, Surface, SurfaceConfiguration};
use std::fmt::{self, Display};
use std::sync::Arc;
use winit::window::Window;

use crate::collections::vector2::Vector2;
use crate::graphics::graphics_mapping::GraphicsMapping;
use crate::input::input_mapping::InputMapping;
use crate::random::Random;

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub enum NetworkModes {
    LocalHost = 4,
    OnlineHost = 3,
    LocalPeer = 2,
    OnlinePeer = 1,
}
impl NetworkModes {
    pub fn all() -> Vec<NetworkModes> {
        vec![NetworkModes::OnlinePeer, NetworkModes::LocalPeer, NetworkModes::OnlineHost, NetworkModes::LocalHost]
    }
    pub fn all_peer() -> Vec<NetworkModes> {
        vec![NetworkModes::OnlinePeer, NetworkModes::LocalPeer]
    }
    pub fn all_host() -> Vec<NetworkModes> {
        vec![NetworkModes::OnlineHost, NetworkModes::LocalHost]
    }
}
impl Display for NetworkModes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkModes::LocalHost => f.write_str("local host"),
            NetworkModes::OnlineHost => f.write_str("online host"),
            NetworkModes::LocalPeer => f.write_str("local peer"),
            NetworkModes::OnlinePeer => f.write_str("online peer"),
        }
    }
}
pub struct GameMode {
    // pub input_mappings: Vec<InputMapping>,
    // pub graphics_mappings: Vec<GraphicsMapping>,
    // pub network_mode: NetworkModes,
    pub game_instances: Vec<GameInstance>,
}
impl GameMode {
    // pub fn new(input_mappings: Vec<InputMapping>, graphics_mappings: Vec<GraphicsMapping>, network_mode: NetworkModes) -> GameMode {
    //     GameMode { input_mappings, graphics_mappings, network_mode }
    // }
    pub fn new(game_instances: Vec<GameInstance>) -> GameMode {
        GameMode { game_instances }
    }

    pub fn new_local_single(input: InputMapping) -> GameMode {
        GameMode {
            game_instances: vec![
                GameInstance::new("peer_player_a", GraphicsMapping::new(Vector2::zero(), Vector2::one()), vec![input], NetworkModes::LocalPeer),
                GameInstance::new("host_logic", GraphicsMapping::new(Vector2::new(0.9, 0.9), Vector2::new(1.0, 1.0)), vec![], NetworkModes::LocalHost),
            ],
        }
    }
    pub fn new_local_splitscreen_2p_vertical(input_p1: InputMapping, input_p2: InputMapping) -> GameMode {
        GameMode {
            game_instances: vec![
                GameInstance::new("peer_player_a", GraphicsMapping::new(Vector2::new(0.5, 0.0), Vector2::new(1.0, 1.0)), vec![input_p1], NetworkModes::LocalPeer),
                GameInstance::new("peer_player_b", GraphicsMapping::new(Vector2::new(0.5, 0.0), Vector2::new(1.0, 1.0)), vec![input_p2], NetworkModes::LocalPeer),
                GameInstance::new("host_logic", GraphicsMapping::new(Vector2::new(0.0, 0.0), Vector2::new(0.5, 1.0)), vec![], NetworkModes::LocalHost),
            ],
        }
    }
    pub fn new_local_splitscreen_2p_horizontal(input_p1: InputMapping, input_p2: InputMapping) -> GameMode {
        GameMode {
            game_instances: vec![
                GameInstance::new("peer_player_a", GraphicsMapping::new(Vector2::new(0.0, 0.0), Vector2::new(1.0, 0.5)), vec![input_p1], NetworkModes::LocalPeer),
                GameInstance::new("peer_player_a", GraphicsMapping::new(Vector2::new(0.0, 0.5), Vector2::new(1.0, 1.0)), vec![input_p2], NetworkModes::LocalPeer),
                GameInstance::new("host_logic", GraphicsMapping::new(Vector2::new(0.9, 0.9), Vector2::new(1.0, 1.0)), vec![], NetworkModes::LocalHost),
            ],
        }
    }
}

pub struct GameInstance {
    pub name: String,
    pub graphics_mappings: GraphicsMapping,
    pub input_mappings: Vec<InputMapping>,
    pub network_mode: NetworkModes,
}
impl GameInstance {
    pub fn new(name: &str, graphics_mappings: GraphicsMapping, input_mappings: Vec<InputMapping>, network_mode: NetworkModes) -> GameInstance {
        GameInstance {
            name: String::from(name),
            graphics_mappings,
            input_mappings,
            network_mode,
        }
    }
}

pub struct GPUInstance {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface: Arc<Surface<'static>>,
    pub adapter: Arc<Adapter>,
    pub window: Arc<Window>,
    pub config: Arc<SurfaceConfiguration>,
} // now that the curio_engine is initialized use those values to populate the system

#[derive(Clone)]
pub struct CurioMetadata {
    pub name: String,
    pub icon: String,
    pub version: VersionNumber,
    pub instance: i32,
}
impl CurioMetadata {
    pub fn new(name: &str, icon: &str, version: VersionNumber) -> CurioMetadata {
        CurioMetadata {
            name: String::from(name),
            icon: String::from(icon),
            version,
            instance: Random::range_int(-9999999, 9999999),
        }
    }
}

impl CurioMetadata {
    pub const fn invalid() -> Self {
        Self {
            name: String::new(),
            icon: String::new(),
            version: VersionNumber::new(0, 0, 0),
            instance: -1,
        }
    }
}
#[derive(Clone)]
pub struct VersionNumber {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
}
impl VersionNumber {
    pub const fn new(major: i32, minor: i32, patch: i32) -> VersionNumber {
        VersionNumber { major, minor, patch }
    }
}

pub struct WindowLayout {
    pub width: i32,
    pub height: i32,
    pub fullscreen: bool,
    pub resizeable: bool,
    pub show_cursor: bool,
}

impl WindowLayout {
    pub fn fullscreen_1080() -> WindowLayout {
        WindowLayout {
            width: 1920,
            height: 1080,
            fullscreen: true,
            resizeable: false,
            show_cursor: true,
        }
    }
    pub fn fullscreen_720() -> WindowLayout {
        WindowLayout {
            width: 1280,
            height: 720,
            fullscreen: true,
            resizeable: false,
            show_cursor: true,
        }
    }
    pub fn windowed_1080() -> WindowLayout {
        WindowLayout {
            width: 1920,
            height: 1080,
            fullscreen: false,
            resizeable: false,
            show_cursor: true,
        }
    }
    pub fn windowed_720() -> WindowLayout {
        WindowLayout {
            width: 1280,
            height: 720,
            fullscreen: false,
            resizeable: false,
            show_cursor: true,
        }
    }
    pub fn custom(width: i32, height: i32, fullscreen: bool, resizeable: bool, show_cursor: bool) -> WindowLayout {
        WindowLayout {
            width: width,
            height: height,
            fullscreen: fullscreen,
            resizeable: resizeable,
            show_cursor: show_cursor,
        }
    }
}
