use crate::{
    gameplay_instance::GameplayInstance,
    static_fns::{register_built_in_facets::register_built_in_component, register_built_in_habits::register_built_in_ecs},
    traits::ui_events::IUIEvent,
};
use curio_core::{
    AxisCode, ButtonCode, Vector3,
    collections::{
        event_queue::{EventQueue, IGameEvent},
        game_mode::GameMode,
        game_state::GameState,
        key_state::KeyState,
    },
    system::{system_component::SystemComponent, system_components::system_component_gameplay::SystemComponentGameplay},
};
use std::{fmt::Display, vec};

pub struct SystemComponentDefaultGameplay<T, U>
where
    T: IGameEvent + Clone + 'static,
    U: IUIEvent + 'static,
{
    game_instance: Vec<GameplayInstance<T, U>>,
}

impl<T, U> SystemComponentDefaultGameplay<T, U>
where
    T: IGameEvent + Clone + 'static,
    U: IUIEvent + 'static,
{
    pub fn new() -> Box<SystemComponentDefaultGameplay<T, U>> {
        // register any built in components to static data

        register_built_in_ecs();
        register_built_in_component();

        // return instance
        Box::new(SystemComponentDefaultGameplay::<T, U> { game_instance: vec![] })
    }
}
impl<T, U> SystemComponentGameplay for SystemComponentDefaultGameplay<T, U>
where
    T: IGameEvent + Display + 'static + Clone,
    U: IUIEvent + 'static,
{
    // fn set_systems(&mut self, _ecs_systems_eventless: Vec<fn() -> Box<dyn ECSSystemEventless>>) {}
}
impl<T, U> SystemComponent for SystemComponentDefaultGameplay<T, U>
where
    T: IGameEvent + Display + 'static + Clone,
    U: IUIEvent + 'static,
{
    fn input_button(&mut self, _game_state: &mut Vec<GameState>, _key_code: ButtonCode, _val: KeyState) {}
    fn input_axis(&mut self, _game_statee: &mut Vec<GameState>, _axis_code: AxisCode, _val: Vector3) {}
    fn order(&self) -> i32 {
        5000
    }
    fn init(&mut self, _: &mut Vec<GameState>) {}
    fn set_game_mode(&mut self, _game_state: &mut Vec<GameState>, game_mode: &GameMode) {
        for _ in &game_mode.game_instances {
            self.game_instance.push(GameplayInstance::new());
        }
    }
    fn debug(&mut self, _game_state: &mut Vec<GameState>, _system_queue: &mut Vec<EventQueue>) {}
    fn tick(&mut self, game_state: &mut Vec<GameState>, event_queue: &mut Vec<EventQueue>) {
        // iterate over each gamestate
        for i in 0..game_state.len() {
            // get this index values
            let game_state = &mut game_state[i];
            let event_queue = &mut event_queue[i];

            // tick the instance
            self.game_instance[i].tick(game_state, event_queue);
        }
    }
}
