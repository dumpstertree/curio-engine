use crate::{
    built_in::stimulant::engine_commands::EngineCommands,
    collections::{event_queue::EventQueue, game_mode::GameMode, ledger::Ledger},
    engine::curio_common::CurioCommon,
    input::{axis_code::AxisCode, key_code::ButtonCode, key_state::KeyState},
    network_modes::NetworkModes,
    random::Random,
    static_data::{global_events::get_global_event_constructor_all, global_states::get_global_state_constructor_all},
    system::system_component::SystemComponent,
    Application, Severity, Vector3,
};

/// An object that will be imbued with the logic of your application.
/// You can redesign Curio by implenting CurioCommon and passing it into the CurioCabinet'.
pub struct Curio {
    command_buffer: Vec<EngineCommands>,
    plugins: Vec<Box<dyn SystemComponent>>,
    nerves: Vec<EventQueue>,
    ledgers: Vec<Ledger>,
    game_mode: GameMode,
}

// impl - Public fns
impl Curio {
    /// Create a curio and imbue it with logic
    pub fn imbue(plugins: Vec<Box<dyn SystemComponent>>, game_mode: GameMode) -> Curio {
        // log
        Application::log(Severity::Info, "Imbuing Curio...");

        // create empty vecs with capacities based on number of game modes
        let mut all_ledgers = Vec::with_capacity(game_mode.game_instances.len());
        let mut all_nerves: Vec<EventQueue> = Vec::with_capacity(game_mode.game_instances.len());

        // log
        Curio::log_ledger();
        Curio::log_nerve();

        // // populate all ids
        let network_instances: Vec<_> = game_mode
            .game_instances
            .iter()
            .map(|inst| CurioNetworkInstance::new(Random::guid(6), inst.network_mode))
            .collect();

        // populate all ledgers and nerves
        for i in 0..game_mode.game_instances.len() {
            //create the network
            let network = CurioNetwork::new(network_instances.clone(), i);

            // create all ledgers
            all_ledgers.push(Ledger::new(network));

            // create all nerves
            all_nerves.push(EventQueue::new(&format!("event_queue__{}", game_mode.game_instances[i].name), game_mode.game_instances[i].network_mode));
        }

        // order components - this is only done on creation
        let mut sorted = Vec::new();
        sorted.extend(plugins);
        sorted.sort_by(|a, b| a.order().cmp(&b.order()));

        //
        Application::log(Severity::Info, "Sorting Plugins...");

        // create and return the instance
        Curio {
            command_buffer: vec![],
            plugins: sorted,
            ledgers: all_ledgers,
            nerves: all_nerves,
            game_mode,
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
        for c in &mut self.plugins {
            // log
            Application::log(Severity::Info, &format!("Init Plugin: {}", &c.name()));

            // init the state
            c.init(&mut self.ledgers);
        }

        // set all plugins
        for c in &mut self.plugins {
            // log
            Application::log(Severity::Info, &format!("Set Gamemode Plugin: {}", &c.name()));

            //set the gamemode the state will start with
            c.set_game_mode(&mut self.ledgers, &self.game_mode);
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
