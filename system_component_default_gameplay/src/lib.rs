use core::{
    collections::{
        event_queue::{EventQueue, IGameEvent},
        game_state::GameState,
    },
    gameplay::ecs::traits::{ecs_event_reciever::EventReciever, ecs_system::ECSSystemEventless},
    static_data::{global_ecs::get_global_ecs_instances, global_event_recievers::get_global_event_receivers},
    system::{system_component::SystemComponent, system_components::system_component_gameplay::SystemComponentGameplay},
};
use hecs::World;
use std::{fmt::Display, vec};

pub struct GameplayInstance<T>
where
    T: IGameEvent + Clone + 'static,
{
    // network_mode: NetworkModes,
    has_been_init: bool,
    // game_state: GameState,
    world: World,
    ecs_systems: Vec<(Box<dyn ECSSystemEventless>, bool)>,
    event_recievers: Vec<Box<dyn EventReciever<T>>>,
}

impl<T> GameplayInstance<T>
where
    T: IGameEvent + Clone + 'static,
{
    pub fn new() -> GameplayInstance<T> {
        //create the world everything is in
        let world = World::new();

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
        GameplayInstance {
            world,
            ecs_systems,
            event_recievers,
            has_been_init: false,
        }
    }
    pub fn tick(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue)
    where
        T: IGameEvent + Display + 'static + Clone,
    {
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

        // dequeue events
    }
}

pub struct SystemComponentDefaultGameplay<T>
where
    T: IGameEvent + Clone + 'static,
{
    game_instance: Vec<GameplayInstance<T>>,
}

impl<T> SystemComponentDefaultGameplay<T>
where
    T: IGameEvent + Clone + 'static,
{
    pub fn new() -> Box<SystemComponentDefaultGameplay<T>> {
        Box::new(SystemComponentDefaultGameplay::<T> { game_instance: vec![] })
    }
}
impl<T> SystemComponentGameplay for SystemComponentDefaultGameplay<T>
where
    T: IGameEvent + Display + 'static + Clone,
{
    fn set_systems(&mut self, _ecs_systems_eventless: Vec<fn() -> Box<dyn ECSSystemEventless>>) {}
}
impl<T> SystemComponent for SystemComponentDefaultGameplay<T>
where
    T: IGameEvent + Display + 'static + Clone,
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
