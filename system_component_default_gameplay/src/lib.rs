use core::{
    collections::{
        event_queue::{self, EventQueue, EventScope, IGameEvent},
        game_state::{self, GameState},
        input_button::InputButtonState,
        input_cursor::InputAxisState,
    },
    gameplay::{
        ecs::traits::{ecs_event_reciever::EventReciever, ecs_system::ECSSystemEventless},
        world_context::WorldContext,
    },
    input::{
        axis_code::{self, AxisCode},
        key_code::ButtonCode,
    },
    static_data::{global_ecs::get_global_ecs_instances, global_event_recievers::get_global_event_receivers},
    system::{system_component::SystemComponent, system_components::system_component_gameplay::SystemComponentGameplay},
};
use hecs::{Component, ComponentRef, DynamicBundle, Entity, QueryBorrow, World};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, marker::PhantomData, sync::Arc, vec};

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
    ecs_systems: Vec<(Box<dyn ECSSystemEventless>, bool)>,
    event_recievers: Vec<Box<dyn EventReciever<T>>>,

    ui: Vec<Box<dyn UI>>,
}

impl<T, U> GameplayInstance<T, U>
where
    T: IGameEvent + Clone + 'static,
    U: IUIEvent + Clone + 'static,
{
    pub fn new() -> GameplayInstance<T, U> {
        //create the world everything is in
        let world = WorldContext::new();

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
            ecs_systems,
            event_recievers,
            has_been_init: false,
            ui: Vec::new(),
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
        //
        let event = event_queue.drain_queued_events::<UIEvents<U>>();
        for e in event {
            match e {
                UIEvents::Open(x) => {
                    let mut i = x.new_instance();
                    i.init();
                    i.present(game_state, event_queue, &mut self.world);
                    //
                    self.ui.push(i)
                }
                UIEvents::Close(_) => todo!("close"),
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
            x.tick(game_state, event_queue, &mut self.world);
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
        Box::new(SystemComponentDefaultGameplay::<T, U> { game_instance: vec![] })
    }
}
impl<T, U> SystemComponentGameplay for SystemComponentDefaultGameplay<T, U>
where
    T: IGameEvent + Display + 'static + Clone,
    U: IUIEvent + 'static,
{
    fn set_systems(&mut self, _ecs_systems_eventless: Vec<fn() -> Box<dyn ECSSystemEventless>>) {}
}
impl<T, U> SystemComponent for SystemComponentDefaultGameplay<T, U>
where
    T: IGameEvent + Display + 'static + Clone,
    U: IUIEvent + 'static,
{
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
    fn present(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut WorldContext);
    fn dismiss(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut WorldContext);
    fn tick(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue, context: &mut WorldContext);
}

pub trait UIHud: UI {}

pub trait UIPanel: UI {
    fn input_button(button: ButtonCode, state: InputButtonState);
    fn input_axis(axis: AxisCode, state: InputAxisState);
}

pub trait UIDialog: UI {
    fn input_button(button: ButtonCode, state: InputButtonState);
    fn input_axis(axis: AxisCode, state: InputAxisState);
}

pub trait IUIEvent: Clone + Copy + Display + Sync {
    fn new_instance(&self) -> Box<dyn UI>;
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
