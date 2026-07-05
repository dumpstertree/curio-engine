use crate::Services;
use crate::{engine::metadata::identity::Identity, ButtonCode, ButtonPressed, CurioBuilder, Commands, CurioNetwork, CurioNetworkParticipant, Nerve, PluginGroupState, Random};
use crate::{
    engine::{curio_common::CurioCommon, serialization::plugin_group_state::CurioState},
    input::axis_code::AxisCode,
    static_data::{global_events::get_global_event_constructor_all, global_states::get_global_state_constructor_all},
    system::system_component::SystemComponent,
    Formation, Ledger, Severity, Vector3, Version,
};
use egui_wgpu::wgpu::{Texture, TextureView};
use std::collections::HashMap;

/// An object that will be imbued with the logic of your application.
/// You can redesign Curio by implenting CurioCommon and passing it into the CurioCabinet'.
pub struct Curio {
    pub identity: Identity,
    pub plugins: Vec<Box<dyn SystemComponent>>,
    pub nerves: Vec<Nerve>,
    pub ledgers: Vec<Ledger>,
    pub formation: Formation,
    command_buffer: Vec<Commands>,
}

// impl -Pub Crate fns
impl Curio {
    pub(crate) fn new(builder: CurioBuilder) -> Self {
        // log
        Curio::log(Severity::Info, "Imbuing Curio...");

        // log ledger starting values
        Curio::log_ledger();

        // log nerve starting values
        Curio::log_nerve();

        // // populate all ids
        let network_instances: Vec<_> = builder
            .gamemode
            .seats
            .iter()
            .map(|inst| CurioNetworkParticipant::new(Random::guid(6), inst.network))
            .collect();

        // create empty vecs with capacities based on number of game modes
        let mut all_ledgers = Vec::with_capacity(builder.gamemode.seats.len());
        let mut all_nerves: Vec<Nerve> = Vec::with_capacity(builder.gamemode.seats.len());

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
        Curio::log(Severity::Info, "Sorting Plugins...");

        // create and return the instance
        Curio {
            identity: builder.metadata,
            command_buffer: vec![],
            plugins: sorted,
            ledgers: all_ledgers,
            nerves: all_nerves,
            formation: builder.gamemode,
        }
    }
}

// impl - Public fns
impl Curio {
    /// Log a system wide message
    pub fn log(severity: Severity, contents: &str) {
        Services::get()
            .logger()
            .log(0, severity, &format!("[SYS]: {}", contents));
    }

    /// Create a new Curio by editing a CurioBuilder
    pub fn create() -> CurioBuilder {
        CurioBuilder {
            metadata: Identity::new("", "", Version::new(0, 0, 0)),
            plugins: Vec::new(),
            gamemode: Formation::custom(Vec::new()),
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
        Curio::log(Severity::Info, &ledger_record_log);
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
        Curio::log(Severity::Info, &nerve_impulse_log);
    }
}
// impl - CurioCommon fns
impl CurioCommon for Curio {
    /// Render the Curio to the provided Texuture. Render will be called on all plugins in order and can be edited/overwritten.
    fn render(&mut self, output_texture: &Texture, output_view: &TextureView, mut encoder: &mut egui_wgpu::wgpu::CommandEncoder) {
        for x in self.plugins.iter_mut() {
            x.render(output_texture, output_view, &mut encoder, &mut self.ledgers, &mut self.nerves);
        }
    }

    fn serializable(&self) -> CurioState {
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
        CurioState {
            identity: self.identity.clone(),
            formation: self.formation.clone(),
            plugin_group_state: PluginGroupState { id_for_tabs: id_for_tabs },
        }
    }
    fn update(&mut self) {
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
                Commands::Tick => {
                    // iterate over each component
                    for c in &mut self.plugins {
                        c.tick(&mut self.ledgers, &mut self.nerves);
                    }
                }
                Commands::Exit => todo!(),
                Commands::Resize(_) => todo!(),
                Commands::Fullscreen(_) => todo!(),
                Commands::Resizable(_) => todo!(),
                Commands::Cursor(_) => todo!(),
            }
        }
    }
    fn input_axis(&mut self, axis: AxisCode, state: Vector3) {
        // log
        Curio::log(Severity::Info, "Input: Axis");

        for c in &mut self.plugins {
            c.input_axis(&mut self.ledgers, axis, state);
        }
    }
    fn input_button(&mut self, button: ButtonCode, state: ButtonPressed) {
        // log
        Curio::log(Severity::Info, "Input: Button");

        for c in &mut self.plugins {
            c.input_button(&mut self.ledgers, button, state);
        }
    }
    fn window_opened(&mut self) {
        // log
        Curio::log(Severity::Info, "Window: Opened");

        // init all plugins
        for plugin in &mut self.plugins {
            // log
            Curio::log(Severity::Info, &format!("Init Plugin: {}", &plugin.name()));

            // init the state
            plugin.init(&mut self.ledgers);
        }

        // set all plugins
        for plugin in &mut self.plugins {
            // log
            Curio::log(Severity::Info, &format!("Set Gamemode Plugin: {}", &plugin.name()));

            //set the gamemode the state will start with
            plugin.set_game_mode(&mut self.ledgers, &self.formation);
        }
    }
    fn window_closed(&mut self) {
        // log
        Curio::log(Severity::Info, "Window: Closed");

        // alert plugins
        for plugin in &mut self.plugins {
            plugin.application_quit();
        }
    }
    fn window_resized(&mut self) {
        Curio::log(Severity::Warning, "Window: Resized - Not yet implemented");
    }
    fn window_moved(&mut self) {
        Curio::log(Severity::Warning, "Window: Moved - Not yet implemented");
    }
}
