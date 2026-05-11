use core::panic;
use std::path::Path;

use log::Metadata;

use crate::{
    built_in::stimulant::engine_commands::EngineCommands,
    collections::{curio_metadata::CurioMetadata, event_queue::Nerve, game_instance::GameInstance, game_mode::GameMode, ledger::Ledger, version_number::VersionNumber},
    engine::curio_common::CurioCommon,
    engine_services::EngineServices,
    input::{axis_code::AxisCode, key_code::ButtonCode, key_state::KeyState},
    network_modes::NetworkModes,
    plugin::{Plugin, PluginCreateFn},
    plugin_loader::load_library,
    random::Random,
    static_data::{global_events::get_global_event_constructor_all, global_states::get_global_state_constructor_all},
    system::system_component::SystemComponent,
    Application, GPUInstance, PluginVersionFn, Severity, Vector3,
};

pub struct CurioBuilder {
    metadata: CurioMetadata,
    plugins: Vec<Box<dyn SystemComponent>>,
    plugin_paths: Vec<String>,
    gamemode: GameMode,
}
impl CurioBuilder {
    pub fn set_game_mode(mut self, gamemode: GameMode) -> Self {
        self.gamemode = gamemode;
        self
    }
    pub fn add_plugin(mut self, plugin: Box<dyn SystemComponent>) -> Self {
        self.plugins.push(plugin);
        self
    }
    pub fn add_plugin_path(mut self, path: &str) -> Self {
        self.plugin_paths.push(path.to_string());
        self
    }
    pub fn set_metadata(mut self, metadata: CurioMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn imbue(self) -> Curio {
        println!("Imbuing");
        Curio::new(self)
    }
}
/// An object that will be imbued with the logic of your application.
/// You can redesign Curio by implenting CurioCommon and passing it into the CurioCabinet'.
pub struct Curio {
    pub meta: CurioMetadata,
    command_buffer: Vec<EngineCommands>,
    plugins: Vec<Box<dyn SystemComponent>>,
    nerves: Vec<Nerve>,
    ledgers: Vec<Ledger>,
    game_mode: GameMode,
}

// impl - Public fns
impl Curio {
    pub fn create() -> CurioBuilder {
        CurioBuilder {
            metadata: CurioMetadata::new("", "", VersionNumber::new(0, 0, 0)),
            plugins: Vec::new(),
            plugin_paths: Vec::new(),
            gamemode: GameMode::new(Vec::new()),
        }
    }

    fn new(builder: CurioBuilder) -> Self {
        // log
        Application::log(Severity::Info, "Imbuing Curio...");

        // create empty vecs with capacities based on number of game modes
        let mut all_ledgers = Vec::with_capacity(builder.gamemode.game_instances.len());
        let mut all_nerves: Vec<Nerve> = Vec::with_capacity(builder.gamemode.game_instances.len());

        // log
        Curio::log_ledger();
        Curio::log_nerve();

        // // populate all ids
        let network_instances: Vec<_> = builder
            .gamemode
            .game_instances
            .iter()
            .map(|inst| CurioNetworkInstance::new(Random::guid(6), inst.network_mode))
            .collect();

        // populate all ledgers and nerves
        for i in 0..builder.gamemode.game_instances.len() {
            //create the network
            let network = CurioNetwork::new(network_instances.clone(), i);

            // create all ledgers
            all_ledgers.push(Ledger::new(network.clone()));

            // create all nerves
            all_nerves.push(Nerve::new(network));
        }

        // order components - this is only done on creation
        let mut sorted = Vec::new();
        sorted.extend(builder.plugins);
        sorted.sort_by(|a, b| a.order().cmp(&b.order()));

        //
        Application::log(Severity::Info, "Sorting Plugins...");

        // create and return the instance
        Curio {
            meta: builder.metadata,
            command_buffer: vec![],
            plugins: sorted,
            ledgers: all_ledgers,
            nerves: all_nerves,
            game_mode: builder.gamemode,
        }
    }

    // Create a curio and imbue it with logic
    // pub fn imbue(plugins: Vec<Box<dyn SystemComponent>>, game_mode: GameMode) -> Curio {
    //     // log
    //     Application::log(Severity::Info, "Imbuing Curio...");

    //     // create empty vecs with capacities based on number of game modes
    //     let mut all_ledgers = Vec::with_capacity(game_mode.game_instances.len());
    //     let mut all_nerves: Vec<Nerve> = Vec::with_capacity(game_mode.game_instances.len());

    //     // log
    //     Curio::log_ledger();
    //     Curio::log_nerve();

    //     // // populate all ids
    //     let network_instances: Vec<_> = game_mode
    //         .game_instances
    //         .iter()
    //         .map(|inst| CurioNetworkInstance::new(Random::guid(6), inst.network_mode))
    //         .collect();

    //     // populate all ledgers and nerves
    //     for i in 0..game_mode.game_instances.len() {
    //         //create the network
    //         let network = CurioNetwork::new(network_instances.clone(), i);

    //         // create all ledgers
    //         all_ledgers.push(Ledger::new(network.clone()));

    //         // create all nerves
    //         all_nerves.push(Nerve::new(network));
    //     }

    //     // order components - this is only done on creation
    //     let mut sorted = Vec::new();
    //     sorted.extend(plugins);
    //     sorted.sort_by(|a, b| a.order().cmp(&b.order()));

    //     //
    //     Application::log(Severity::Info, "Sorting Plugins...");

    //     // create and return the instance
    //     // Curio {
    //     //     curiomne
    //     //     command_buffer: vec![],
    //     //     plugins: sorted,
    //     //     ledgers: all_ledgers,
    //     //     nerves: all_nerves,
    //     //     game_mode,
    //     // }
    // }
}
// impl -Private fns
impl Curio {
    fn log_ledger() {
        // get all being added
        let all = get_global_state_constructor_all();

        //create empty string
        let mut ledger_record_log = String::new();

        //append all update
        for x in &all {
            ledger_record_log += &format!("     Found 'Record' for: {}\n", x.0);
        }
        ledger_record_log += &format!("'Ledger(s)' built with {} 'Record(s)'", all.len());

        //log
        Application::log(Severity::Info, &ledger_record_log);
    }
    fn log_nerve() {
        // get all being added
        let all = get_global_event_constructor_all();

        // create empty string
        let mut nerve_impulse_log = String::new();

        // append all update
        nerve_impulse_log += "Initializing 'Nerve(s)'...\n";
        for x in &all {
            nerve_impulse_log += &format!("     Found 'Impulse' for: {}\n", x.0);
        }
        nerve_impulse_log += &format!("'Nerve(s)' built with {} 'Impulse(s)'", all.len());

        // log
        Application::log(Severity::Info, &nerve_impulse_log);
    }
}
// impl - CurioCommon fns
impl CurioCommon for Curio {
    fn application_refresh(&mut self) {
        // clear buffer before use
        self.command_buffer.clear();

        // iterate over each component calling refresh and gathering the returned commands and adding them to the buffer
        for c in &mut self.plugins {
            self.command_buffer
                .extend(c.refresh(&mut self.ledgers, &mut self.nerves));
        }

        // call fn on each command in the buffer
        for command in &self.command_buffer {
            match command {
                EngineCommands::Tick => {
                    // iterate over each component
                    for c in &mut self.plugins {
                        // init the state
                        c.tick(&mut self.ledgers, &mut self.nerves);
                    }
                }
                _ => {}
            }
        }
    }
    fn input_axis(&mut self, axis: AxisCode, state: Vector3) {
        // log
        Application::log(Severity::Info, "Input: Axis");

        for c in &mut self.plugins {
            c.input_axis(&mut self.ledgers, axis, state);
        }
    }
    fn input_button(&mut self, button: ButtonCode, state: KeyState) {
        // log
        Application::log(Severity::Info, "Input: Button");

        for c in &mut self.plugins {
            c.input_button(&mut self.ledgers, button, state);
        }
    }
    fn window_opened(&mut self) {
        // log
        Application::log(Severity::Info, "Window: Opened");

        // init all plugins
        for plugin in &mut self.plugins {
            // log
            Application::log(Severity::Info, &format!("Init Plugin: {}", &plugin.name()));

            // init the state
            plugin.init(&mut self.ledgers);
        }

        // set all plugins
        for plugin in &mut self.plugins {
            // log
            Application::log(Severity::Info, &format!("Set Gamemode Plugin: {}", &plugin.name()));

            //set the gamemode the state will start with
            plugin.set_game_mode(&mut self.ledgers, &self.game_mode);
        }
    }
    fn window_closed(&mut self) {
        // log
        Application::log(Severity::Info, "Window: Closed");

        // alert plugins
        for plugin in &mut self.plugins {
            plugin.application_quit();
        }
    }
    fn window_resized(&mut self) {
        // log
        Application::log(Severity::Info, "Window: Resized");

        // alert plugins
        for plugin in &mut self.plugins {
            plugin.application_resize(0.0, 0.0);
        }
    }
    fn window_moved(&mut self) {
        // log
        Application::log(Severity::Warning, "Window: Moved - Not yet implemented");
    }
}

#[derive(Clone)]
pub struct CurioNetwork {
    all: Vec<CurioNetworkInstance>,
    me_index: usize,
}
impl CurioNetwork {
    pub fn new(all: Vec<CurioNetworkInstance>, me: usize) -> CurioNetwork {
        CurioNetwork { all: all, me_index: me }
    }
    pub fn all(&self) -> &[CurioNetworkInstance] {
        &self.all
    }
    pub fn me(&self) -> &CurioNetworkInstance {
        &self.all[self.me_index]
    }
}

#[derive(Clone)]
pub struct CurioNetworkInstance {
    pub guid: i32,
    pub mode: NetworkModes,
}
impl CurioNetworkInstance {
    pub fn new(guid: i32, mode: NetworkModes) -> CurioNetworkInstance {
        CurioNetworkInstance { guid, mode }
    }
}

// matches your curio_init signature exactly
type InitCurioFn = unsafe extern "C" fn(gpu: *const EngineServices) -> *mut Curio;

pub struct LoadedCurio {
    pub curio: Box<Curio>,
    _lib: Library,
}

pub fn load_curio(folder: &Path, gpu: *const EngineServices) -> LoadedCurio {
    let entries = std::fs::read_dir(folder).expect("plugins folder not found");

    for entry in entries.flatten() {
        let path = entry.path();

        let is_plugin = match path.extension().and_then(|e| e.to_str()) {
            Some("so") | Some("dll") | Some("dylib") => true,
            _ => false,
        };

        if !is_plugin {
            continue;
        }

        let lib = match load_library(&path) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("failed to load {:?}: {}", path, e);
                continue;
            }
        };

        let curio = unsafe {
            // look for curio_init — if not found this .so isn't a curio game
            let init_fn: Symbol<InitCurioFn> = match lib.get(b"curio_init") {
                Ok(f) => f,
                Err(_) => continue,
            };

            let raw = init_fn(gpu);

            if raw.is_null() {
                eprintln!("curio_init returned null for {:?}", path);
                continue;
            }

            Box::from_raw(raw)
        };

        eprintln!("loaded: {}", curio.meta.name);

        return LoadedCurio { curio, _lib: lib };
    }

    panic!("");
}
use libloading::{Library, Symbol};

pub struct LoadedPlugin {
    plugin: Box<dyn Plugin>,
    _lib: Library, // keep the .so alive — drop order matters
}

// pub fn load_curio(folder: &Path) -> Vec<*mut Curio> {
//     let mut loaded = Vec::new();

//     let lib = load_library(folder).unwrap();
//     let entries = std::fs::read_dir(folder).expect("plugins folder not found");

//     for entry in entries.flatten() {
//         let path = entry.path();

//         // pick up .so on Linux, .dll on Windows, .dylib on Mac
//         let is_plugin = match path.extension().and_then(|e| e.to_str()) {
//             Some("so") | Some("dll") | Some("dylib") => true,
//             _ => false,
//         };

//         if !is_plugin {
//             continue;
//         }

//         let init_curio: Symbol<InitCurioFn> = unsafe { lib.get(b"curio_init").unwrap() };
//         let raw = unsafe { init_curio() };
//         let app = raw;

//         loaded.push(app);
//     }

//     loaded
// }
// fn load_plugin(path: &Path) -> Result<LoadedPlugin, Box<dyn std::error::Error>> {
//     unsafe {
//         let lib = load_library(path)?;

//         // check version before doing anything else
//         // let version_fn: Symbol<PluginVersionFn> = lib.get(b"_plugin_api_version")?;
//         // let version = version_fn();
//         // if version != engine::ENGINE_API_VERSION {
//         //     return Err(format!("plugin version mismatch: expected {}, got {}", engine::ENGINE_API_VERSION, version).into());
//         // }

//         let create_fn: Symbol<PluginCreateFn> = lib.get(b"_plugin_create")?;
//         let raw = create_fn();
//         let plugin = Box::from_raw(raw);

//         Ok(LoadedPlugin { plugin, _lib: lib })
//     }
// }

// pub type InitCurioFn = unsafe extern "C" fn() -> *mut Curio;
