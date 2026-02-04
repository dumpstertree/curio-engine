use crate::{
    built_in::stimulant::engine_commands::EngineCommands,
    collections::{event_queue::EventQueue, game_mode::GameMode, game_state::GameState, key_state::KeyState},
    engine::curio_common::CurioCommon,
    input::{axis_code::AxisCode, key_code::ButtonCode},
    random::Random,
    system::system_component::SystemComponent,
    Vector3,
};

/// An object that will be imbued with the logic of your application.
/// You can redesign Curio by implenting CurioCommon and passing it into the CurioCabinet'.
pub struct Curio {
    command_buffer: Vec<EngineCommands>,
    components: Vec<Box<dyn SystemComponent>>,
    game_events: Vec<EventQueue>,
    game_state: Vec<GameState>,
    game_mode: GameMode,
}

// impl - Public fns
impl Curio {
    /// Create a curio and imbue it with logic
    pub fn imbue(components: Vec<Box<dyn SystemComponent>>, game_mode: GameMode) -> Curio {
        // create empty vecs with capacities based on number of game modes
        let mut ids = Vec::with_capacity(game_mode.game_instances.len());
        let mut states = Vec::with_capacity(game_mode.game_instances.len());
        let mut events: Vec<EventQueue> = Vec::with_capacity(game_mode.game_instances.len());

        // populate all ids
        for _ in &game_mode.game_instances {
            ids.push(Random::range_int(-999, 999));
        }

        // populate all states and events
        for i in 0..game_mode.game_instances.len() {
            states.push(GameState::new(&format!("game_state__{}", game_mode.game_instances[i].name), game_mode.game_instances[i].network_mode, ids[i], ids.clone()));
            events.push(EventQueue::new(&format!("event_queue__{}", game_mode.game_instances[i].name), game_mode.game_instances[i].network_mode));
        }

        // order components - this is only done on creation
        let mut sorted = Vec::new();
        sorted.extend(components);
        sorted.sort_by(|a, b| a.order().cmp(&b.order()));

        // create and return the instance
        Curio {
            command_buffer: vec![],
            components: sorted,
            game_state: states,
            game_events: events,
            game_mode,
        }
    }
}

// impl - CurioCommon fns
impl CurioCommon for Curio {
    fn application_refresh(&mut self) {
        // clear buffer before use
        self.command_buffer.clear();

        // iterate over each component calling refresh and gathering the returned commands and adding them to the buffer
        for c in &mut self.components {
            self.command_buffer
                .extend(c.refresh(&mut self.game_state, &mut self.game_events));
        }

        // call fn on each command in the buffer
        for command in &self.command_buffer {
            match command {
                EngineCommands::Tick => {
                    // iterate over each component
                    for c in &mut self.components {
                        // init the state
                        c.tick(&mut self.game_state, &mut self.game_events);
                    }
                }
                _ => {}
            }
        }
    }
    fn input_axis(&mut self, axis: AxisCode, state: Vector3) {
        for c in &mut self.components {
            c.input_axis(&mut self.game_state, axis, state);
        }
    }
    fn input_button(&mut self, button: ButtonCode, state: KeyState) {
        for c in &mut self.components {
            c.input_button(&mut self.game_state, button, state);
        }
    }
    fn window_opened(&mut self) {
        for c in &mut self.components {
            // init the state
            c.init(&mut self.game_state);

            //set the gamemode the state will start with
            c.set_game_mode(&mut self.game_state, &self.game_mode);
        }
    }
    fn window_closed(&mut self) {
        for c in &mut self.components {
            c.application_quit();
        }
    }
    fn window_resized(&mut self) {
        for c in &mut self.components {
            c.application_resize(0.0, 0.0);
        }
    }
    fn window_moved(&mut self) {}
}
