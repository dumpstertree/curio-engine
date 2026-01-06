pub mod ecs_event_reciever;
pub mod ecs_system;
pub mod prefab;
pub mod world_context;

pub mod static_data {
    pub mod global_components;
    pub mod global_ecs;
    pub mod global_event_recievers;
}
pub mod component {
    pub mod component_camera;
    pub mod component_input_index;
    pub mod component_renderer_animated;
    pub mod component_renderer_static;
    pub mod component_renderer_text;
    pub mod component_colliders {
        pub mod component_collider_box;
        pub mod component_collider_sphere;
    }
    pub mod component_light;
    pub mod component_transform;
    pub mod component_transform2d;
}
pub mod system {
    pub mod system_camera_update_state;
    // pub mod system_collider_box_update_state;
    // pub mod system_collider_sphere_update_state;
    // pub mod system_debug_camera;
    // pub mod system_debug_gui_colliders;
    // pub mod system_debug_gui_collision;
    // pub mod system_debug_gui_entity;
    pub mod system_debug_gui_screen;
    pub mod system_debug_gui_time;
    pub mod system_debug_toggle;
    pub mod system_renderer_update_light_state;
    pub mod system_renderer_update_state;
}
pub mod field_override;

use core::{
    collections::{
        event_queue::{EventQueue, EventScope, IGameEvent},
        game_state::GameState,
        input_button::InputButtonState,
        input_cursor::InputAxisState,
        key_state::KeyState,
    },
    input::{
        axis_code::AxisCode,
        key_code::ButtonCode,
    },
    system::{system_component::SystemComponent, system_components::system_component_gameplay::SystemComponentGameplay},
};
use hecs::World;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap, fmt::Display, hash::Hash, marker::PhantomData, rc::Rc, vec};

use crate::{
    component::{component_camera::Camera, component_light::ComponentLight, component_renderer_animated::RendererAnimated, component_renderer_static::Renderer, component_renderer_text::ComponentRendererText, component_transform::Transform, component_transform2d::Transform2D},
    ecs_event_reciever::EventReciever,
    ecs_system::ECSSystemEventless,
    static_data::{
        global_components::register_global_component,
        global_ecs::{get_global_ecs_instances, register_global_ecs},
        global_event_recievers::get_global_event_receivers,
    },
    system::{
        system_camera_update_state::PostCameraECSSystem,
        system_debug_gui_screen::SystemDebugGuiScreen,
        system_debug_gui_time::SystemDebugGuiTime,
        system_debug_toggle::SystemDebugToggle,
        system_renderer_update_light_state::{self},
        system_renderer_update_state,
    },
    world_context::{WorldContext, WorldContext2D},
};

pub struct GameplayInstance<T, U>
where
    T: IGameEvent + Clone + 'static,
    U: IUIEvent + Clone + 'static,
{
    phantom_u: PhantomData<U>,
    // network_mode: NetworkModes,
    has_been_init: bool,
    // game_state: GameState,
    world: WorldContext,
    world_2d: WorldContext2D,
    ecs_systems: Vec<(Box<dyn ECSSystemEventless>, bool)>,
    event_recievers: Vec<Box<dyn EventReciever<T>>>,

    ui: HashMap<U, Box<dyn UIPanel>>,
}

pub fn register_built_in_component() {
    register_global_component::<Transform>();
    register_global_component::<Transform2D>();
    register_global_component::<Camera>();
    register_global_component::<ComponentLight>();
    register_global_component::<Renderer>();
    register_global_component::<RendererAnimated>();
    register_global_component::<ComponentRendererText>();
}
pub fn register_built_in_ecs() {
    register_global_ecs::<PostCameraECSSystem>();
    register_global_ecs::<SystemDebugGuiScreen>();
    register_global_ecs::<SystemDebugGuiTime>();
    register_global_ecs::<SystemDebugToggle>();
    register_global_ecs::<system_renderer_update_light_state::SystemRendererUpdateState>();
    register_global_ecs::<system_renderer_update_state::SystemRendererUpdateState>();
}

impl<T, U> GameplayInstance<T, U>
where
    T: IGameEvent + Clone + 'static,
    U: IUIEvent + Clone + 'static,
{
    pub fn new() -> GameplayInstance<T, U> {
        //
        let w = Rc::new(RefCell::new(World::new()));
        //create the world everything is in
        let world = WorldContext::new(w.clone());
        let world_2d = WorldContext2D::new(w.clone());

        // get all ecs instances
        let mut ecs_systems = vec![];
        for x in get_global_ecs_instances() {
            ecs_systems.push((x, false));
        }

        //get all reciever instances
        let mut event_recievers = vec![];
        for x in get_global_event_receivers::<T>() {
            event_recievers.push(x);
        }

        // create the instance
        GameplayInstance::<T, U> {
            world,
            world_2d,
            ecs_systems,
            event_recievers,
            has_been_init: false,
            ui: HashMap::new(),
            phantom_u: PhantomData::default(),
        }
    }
    pub fn tick(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue) {
        // if not init -> init
        if !self.has_been_init {
            // flip flag
            self.has_been_init = true;

            // initialize each
            for x in self.ecs_systems.iter_mut() {
                x.0.init(game_state, &mut self.world, event_queue);
            }
        }

        // if not enabled -> enable
        for ecs_system in self.ecs_systems.iter_mut() {
            // do a network mode check
            let run_on_network_modes = ecs_system.0.run_on_instance(game_state);
            let this_network_mode = &game_state.network_capabilities.clone().unwrap().privilege;

            // ignore if doesnt run on this instance
            if !run_on_network_modes.contains(this_network_mode) {
                continue;
            }

            // get cur state of enable
            let is_enabled = ecs_system.1;
            let should_be_enabled = ecs_system.0.is_enabled(game_state, &mut self.world);

            // enable
            if should_be_enabled && !is_enabled {
                ecs_system
                    .0
                    .enable(game_state, &mut self.world, event_queue);
            }

            // disable
            if !should_be_enabled && is_enabled {
                ecs_system
                    .0
                    .disable(game_state, &mut self.world, event_queue);
            }

            // save the new value
            ecs_system.1 = should_be_enabled;
        }

        let this_network_mode = &game_state.network_capabilities.clone().unwrap().privilege;

        // update any timed events to get added to the queu before we pull anything
        event_queue.update_timed_events();
        //
        let event = event_queue.drain_queued_events::<UIEvents<U>>();
        for e in event {
            match e {
                UIEvents::Open(x) => {
                    let mut i = x.new_instance();
                    i.init();
                    i.present(game_state, event_queue, &mut self.world_2d);
                    //
                    self.ui.insert(x, i);
                }
                UIEvents::Close(u) => {
                    if let Some(mut x) = self.ui.remove(&u) {
                        x.dismiss(game_state, event_queue, &mut self.world_2d);
                    }
                }
            }
        }

        //
        let mut event = event_queue.drain_queued_events::<T>();

        while event.len() > 0 {
            // dequeue the next event
            let Some(this_event) = event.get(0) else {
                continue;
            };

            // apply the event to each of our recievers
            for event_reciever in &mut self.event_recievers {
                // make sure reciever is enabled
                if !event_reciever.is_enabled(game_state) {
                    continue;
                }

                // make sure reciever can run on this instance type
                if !event_reciever
                    .run_on_instance(game_state)
                    .contains(this_network_mode)
                {
                    continue;
                }

                // apply the event to the reciever
                event_reciever.dequeue_event(game_state, &mut self.world, event_queue, this_event);
            }

            // remove
            event.remove(0);

            // add any new events that were added during the application of events
            event.extend(event_queue.drain_queued_events::<T>());
            // event.splice(0..0, event_queue.drain_queued_events::<T>());

            // let mut e = event_queue.drain_queued_events();
            // e.reverse();
            // for x in e {
            //     event.insert(0, x);
            // }
        }

        for ecs_system in self.ecs_systems.iter_mut() {
            if !ecs_system.1 {
                continue;
            }
            // do a network mode check
            let run_on_network_modes = ecs_system.0.run_on_instance(game_state);
            let this_network_mode = &game_state.network_capabilities.clone().unwrap().privilege;

            // ignore if doesnt run on this instance
            if !run_on_network_modes.contains(this_network_mode) {
                continue;
            }
            // debug
            ecs_system.0.debug(game_state, &mut self.world, event_queue);
            // tick
            ecs_system
                .0
                .will_tick(game_state, &mut self.world, event_queue);
            ecs_system.0.tick(game_state, &mut self.world, event_queue);
            ecs_system
                .0
                .did_tick(game_state, &mut self.world, event_queue);
        }

        for x in self.ui.iter_mut() {
            x.1.tick(game_state, event_queue, &mut self.world_2d);
        }
        // dequeue events
    }
}

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
    fn input_button(&mut self, _game_state: &mut Vec<GameState>, key_code: ButtonCode, val: core::collections::key_state::KeyState) {
        // for x in self.game_instance.iter_mut() {
        //     for y in x.ui.iter_mut() {
        //         y.input_button(key_code, val);
        //     }
        // }
    }
    fn input_axis(&mut self, _game_statee: &mut Vec<GameState>, axis_code: AxisCode, val: core::collections::vector3::Vector3) {
        // for x in self.game_instance {
        //     for y in x.ui {
        //         // y.input_axis(axis_code, inputac);
        //     }
        // }
    }
    fn order(&self) -> i32 {
        5000
    }
    fn init(&mut self, _: &mut Vec<GameState>) {}
    fn set_game_mode(&mut self, _game_state: &mut Vec<GameState>, game_mode: &core::dumpster_engine::GameMode) {
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

pub trait UI {
    fn init(&mut self);
    /*
       let obj = context.spawn()
       let t = obj.addcomponent<Text>
       let a = obj.addcomponent<Audio>

        t.set_contents()

        obj.destroy()

    */
    fn present(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut WorldContext2D);
    fn dismiss(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut WorldContext2D);
    fn tick(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut WorldContext2D);
}

pub trait UIHud: UI {}

pub trait UIPanel: UI {
    fn input_button(&mut self, button: ButtonCode, state: KeyState);
    fn input_axis(&mut self, axis: AxisCode, state: InputAxisState);
}

pub trait UIDialog: UI {
    fn input_button(button: ButtonCode, state: InputButtonState);
    fn input_axis(axis: AxisCode, state: InputAxisState);
}

pub trait IUIEvent: Clone + Copy + Display + Sync + PartialEq + Eq + Hash {
    fn new_instance(&self) -> Box<dyn UIPanel>;
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum UIEvents<T>
where
    T: Clone + Sync + IUIEvent + 'static,
{
    Open(T),
    Close(T),
}
impl<T> Display for UIEvents<T>
where
    T: Clone + Sync + IUIEvent + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
impl<T> IGameEvent for UIEvents<T>
where
    T: Clone + Sync + IUIEvent + 'static,
{
    fn id() -> i32
    where
        Self: Sized + 'static,
    {
        1
    }

    fn ownership(&self) -> EventScope
    where
        Self: Sized + 'static,
    {
        EventScope::ConnectedPeers
    }
}
