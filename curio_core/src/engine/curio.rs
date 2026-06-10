use crate::{plugin_loader, ButtonCode, ButtonPressed, Nerve, NetworkModes};
use core::panic;
use std::{collections::HashMap, path::Path, time::Instant};

use crate::{
    built_in::stimulant::engine_commands::EngineCommands,
    engine::{curio_common::CurioCommon, curio_metadata::CurioMetadata},
    engine_services::EngineServices,
    input::axis_code::AxisCode,
    plugin_loader::load_library,
    random::Random,
    static_data::{global_events::get_global_event_constructor_all, global_states::get_global_state_constructor_all},
    system::system_component::SystemComponent,
    Application, Formation, GPUInstance, Ledger, Severity, Vector3, Version,
};

pub struct CurioBuilder {
    metadata: CurioMetadata,
    plugins: Vec<Box<dyn SystemComponent>>,
    plugin_paths: Vec<String>,
    gamemode: Formation,
}
impl CurioBuilder {
    pub fn set_game_mode(mut self, gamemode: Formation) -> Self {
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
    pub plugins: Vec<Box<dyn SystemComponent>>,
    pub nerves: Vec<Nerve>,
    pub ledgers: Vec<Ledger>,
    pub game_mode: Formation,
}

// impl - Public fns
impl Curio {
    pub fn tab_snapshot(&self) -> TabGroupState {
        let mut id_for_tabs = HashMap::new();

        // get all the ledger tabs
        for x in &self.ledgers {
            let s = x.to_state();
            let key = &s.0;
            if !id_for_tabs.contains_key(key) {
                id_for_tabs.insert(key.clone(), Vec::new());
            }
            if let Some(rr) = id_for_tabs.get_mut(key) {
                rr.push(s.1);
            }
        }

        // get all the plugin tabs
        for x in &self.plugins {
            let y = x.get_state(&self.ledgers);
            for r in y {
                if !id_for_tabs.contains_key(&r.0) {
                    id_for_tabs.insert(r.0.clone(), Vec::new());
                }
                if let Some(rr) = id_for_tabs.get_mut(&r.0) {
                    rr.push(r.1);
                }
            }
        }

        TabGroupState { id_for_tabs }
    }

    pub fn context_snapshot(&self) -> FormsSnapshot {
        FormsSnapshot { forms: vec![] }
    }
    pub fn create() -> CurioBuilder {
        CurioBuilder {
            metadata: CurioMetadata::new("", "", Version::new(0, 0, 0)),
            plugins: Vec::new(),
            plugin_paths: Vec::new(),
            gamemode: Formation::custom(Vec::new()),
        }
    }

    fn new(builder: CurioBuilder) -> Self {
        // log
        Application::log(Severity::Info, "Imbuing Curio...");

        // create empty vecs with capacities based on number of game modes
        let mut all_ledgers = Vec::with_capacity(builder.gamemode.seats.len());
        let mut all_nerves: Vec<Nerve> = Vec::with_capacity(builder.gamemode.seats.len());

        // log
        Curio::log_ledger();
        Curio::log_nerve();

        // // populate all ids
        let network_instances: Vec<_> = builder
            .gamemode
            .seats
            .iter()
            .map(|inst| CurioNetworkInstance::new(Random::guid(6), inst.network))
            .collect();

        // populate all ledgers and nerves
        for i in 0..builder.gamemode.seats.len() {
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
                        // let now = Instant::now();
                        // init the state
                        c.tick(&mut self.ledgers, &mut self.nerves);

                        // println!("{}: plugin took: {}",c.name(), now.elapsed().as_nanos() as f32 * 0.000001);
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
    fn input_button(&mut self, button: ButtonCode, state: ButtonPressed) {
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

#[derive(Default, Clone, Serialize)]
pub struct LedgerSnapshot {
    pub instances: Vec<EditorLedgerState>,
}

#[derive(Default, Clone, Serialize)]
pub struct EditorLedgerState {
    pub owner: i32,
    pub mode: String,
    pub records: Vec<EditorRecordState>,
}

#[derive(Default, Clone, Serialize)]
pub struct EditorRecordState {
    pub typeid: String,
    pub value: serde_json::Value,
}
#[derive(Default, Clone, Serialize)]
pub struct FormsSnapshot {
    pub forms: Vec<EditorSceneState>,
}
#[derive(Default, Clone, Serialize)]

pub struct EditorSceneState {
    pub forms: Vec<EditorFormState>,
}
#[derive(Default, Clone, Serialize)]
pub struct TabGroupState {
    //populates the left menu. ids are used for the dropdown and value are all the tabs in the tabgroup
    pub id_for_tabs: HashMap<String, Vec<TabState>>,
}

#[derive(Default, Clone, Serialize)]
pub struct TabState {
    // name of the tab
    pub tab_name: String,
    // all the objects to display vertically
    pub objects: Vec<ObjectState>,
}

#[derive(Default, Clone, Serialize)]
pub struct ObjectState {
    // name of object
    pub object_name: String,
    // objects can be recusive but dont have to be
    pub children: Vec<ObjectState>,
    // when clicked this data is populated into the inspector
    pub components: Vec<ComponentState>,
}

#[derive(Default, Clone, Serialize)]
pub struct ComponentState {
    // name of component
    pub component_name: String,
    // all the actual data in the component
    pub fields: Vec<FieldState>,
}

#[derive(Default, Clone, Serialize)]
pub struct FieldState {
    // name of the field
    pub field_name: String,
    // serialized data in the field
    pub data: serde_json::Value,
}
impl FieldState {
    pub fn new<T: Serialize>(field_name: &str, value: T) -> FieldState {
        FieldState {
            field_name: field_name.to_string(),
            data: serde_json::to_value(value).unwrap(),
        }
    }
}

#[derive(Default, Clone, Serialize)]
pub struct EditorFormState {
    pub guid: i32,
    pub name: String,
    pub facets: Vec<EditorFacetState>,
    pub children: Vec<EditorSceneState>,
}
#[derive(Default, Clone, Serialize)]
pub struct EditorFacetState {
    pub guid: i32,
    pub name: String,
    pub value: serde_json::Value,
}

pub struct LoadedCurio {
    pub curio: Box<Curio>,
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

        let _ = load_library(&path);

        let l2 = plugin_loader::library_slot().lock();
        let lib = match l2 {
            Ok(l) => l,
            Err(e) => {
                panic!("failed to load {:?}: {}", path, e);
            }
        };

        // let lib = match lib {
        //     Some(x) => {
        //         x
        //     },
        //     None => {
        //         panic!("failed to load {:?}: {}", path, e)
        //     }
        // };

        let lib = lib.as_ref().unwrap();

        let curio = unsafe {
            // look for curio_init — if not found this .so isn't a curio game
            let init_fn: Symbol<InitCurioFn> = if let Ok(f) = lib.get(b"curio_init") { f } else { continue };

            let raw = init_fn(gpu);

            if raw.is_null() {
                eprintln!("curio_init returned null for {:?}", path);
                continue;
            }

            Box::from_raw(raw)
        };

        eprintln!("loaded: {}", curio.meta.name);

        return LoadedCurio { curio };
    }

    panic!("");
}
use libloading::{Library, Symbol};
use serde::Serialize;
