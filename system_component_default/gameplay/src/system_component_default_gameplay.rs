use crate::{
    gameplay_instance::GameplayInstance,
    static_data::global_components::COMPONENT_REGISTRY,
    static_fns::{register_built_in_facets::register_built_in_component, register_built_in_habits::register_built_in_ecs},
    traits::ui_events::IUIEvent,
};
use curio_core::{AxisCode, ButtonCode, ButtonPressed, Formation, ImpulseCommon, Ledger, Nerve, PluginCommon, PluginState, Vector3};
use std::vec;

pub struct SystemComponentDefaultGameplay<T, U>
where
    T: ImpulseCommon + 'static,
    U: IUIEvent + 'static,
{
    game_instance: Vec<GameplayInstance<T, U>>,
}

impl<T, U> SystemComponentDefaultGameplay<T, U>
where
    T: ImpulseCommon + Clone + 'static,
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
// impl<T, U> SystemComponentGameplay for SystemComponentDefaultGameplay<T, U>
// where
//     T: IGameEvent + Display + 'static + Clone,
//     U: IUIEvent + 'static,
// {
//     // fn set_systems(&mut self, _ecs_systems_eventless: Vec<fn() -> Box<dyn ECsystemEventless>>) {}
// }
impl<T, U> PluginCommon for SystemComponentDefaultGameplay<T, U>
where
    T: ImpulseCommon + 'static + Clone,
    U: IUIEvent + 'static,
{
    fn name(&self) -> String {
        "Gameplay".to_owned()
    }
    fn input_button(&mut self, _ledger: &mut Vec<Ledger>, _key_code: ButtonCode, _val: ButtonPressed) {}
    fn input_axis(&mut self, _ledgere: &mut Vec<Ledger>, _axis_code: AxisCode, _val: Vector3) {}
    fn order(&self) -> i32 {
        5000
    }
    fn init(&mut self, _: &mut Vec<Ledger>) {}
    fn set_formation(&mut self, _ledger: &mut Vec<Ledger>, game_mode: &Formation) {
        for _ in &game_mode.seats {
            self.game_instance.push(GameplayInstance::new());
        }
    }
    fn tick(&mut self, ledger: &mut Vec<Ledger>, event_queue: &mut Vec<Nerve>) {
        // iterate over each gamestate
        for i in 0..ledger.len() {
            // get this index values
            let ledger = &mut ledger[i];
            let event_queue = &mut event_queue[i];

            // tick the instance
            self.game_instance[i].tick(ledger, event_queue);
        }
    }
    fn peek(&self) -> Vec<curio_core::ComponentState> {
        let reg = COMPONENT_REGISTRY.read().expect("Registry poisoned");
        reg.get_def_state.iter().map(|x| x.1.clone()).collect()
    }
    fn serializable(&self, ledger: &Vec<Ledger>) -> Vec<(String, PluginState)> {
        let mut result = Vec::new();
        for i in 0..self.game_instance.len() {
            let name = format!("{}-{}", ledger[i].network.me().mode, ledger[i].network.me().guid.to_string());
            let state = self.game_instance[i].get_state();
            result.push((name, state));
        }
        result
    }
}
