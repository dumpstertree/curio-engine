use core::{
    collections::{
        event_queue::EventQueue,
        game_state::GameState,
    },
    gameplay::ecs::traits::{ecs_event_reciever::EventReciever, ecs_system::ECSSystemEventless},
    io::asset_loader::AssetLoader,
    system::{system_component::SystemComponent, system_components::system_component_gameplay::SystemComponentGameplay},
};
use hecs::World;
use intertrait::cast::CastMut;

pub struct GameplayInstance {
    // network_mode: NetworkModes,
    has_been_init: bool,
    // game_state: GameState,
    world: World,
    ecs_systems: Vec<(Box<dyn ECSSystemEventless>, bool)>,
}

impl GameplayInstance {
    pub fn new(ecs_systems_constructors: &Vec<fn() -> Box<dyn ECSSystemEventless>>) -> GameplayInstance {
        let world = World::new();
        // let game_state = GameState::new(network_mode.clone(), game_state_id, all_game_state_ids);
        let mut ecs_systems = vec![];
        for constructor in ecs_systems_constructors {
            let ecs_system = constructor();
            ecs_systems.push((ecs_system, false));
        }

        GameplayInstance { world, ecs_systems, has_been_init: false }

        // let ecs_systems:
    }
    pub fn tick<T>(&mut self, game_state: &mut GameState, event_queue: &mut EventQueue)
    where
        T: 'static + Clone,
    {
        // if not init -> init
        if !self.has_been_init {
            // flip flag
            self.has_been_init = true;

            // initialize each
            for x in self.ecs_systems.iter_mut() {
                x.0.init(game_state, &mut self.world, event_queue, &mut AssetLoader::new());
            }
        }

        // if not enabled -> enable
        for ecs_system in self.ecs_systems.iter_mut() {
            // do a network mode check
            let run_on_network_modes = ecs_system.0.run_on_instance(game_state);
            let this_network_mode = &game_state.network_mode;

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

        //
        let mut event = event_queue.drain_queued_events::<T>();
        //
        if event.len() > 0 {
            // println!("num of events {}", event.len());
        }
        while event.len() > 0 {
            // events to add based on current triggered event
            let mut to_append = vec![];
            // dequeue the next event
            let Some(this_event) = event.get(0) else {
                continue;
            };

            // iteerate over each system
            for ecs_system in self.ecs_systems.iter_mut() {
                // if enabled tick
                if ecs_system.1 {
                    // do a network mode check
                    let run_on_network_modes = ecs_system.0.run_on_instance(game_state);
                    let this_network_mode = &game_state.network_mode;

                    // ignore if doesnt run on this instance
                    if !run_on_network_modes.contains(this_network_mode) {
                        continue;
                    }

                    // guard - try cast the system to event reciever
                    let event_reciever = (*ecs_system.0).cast::<dyn EventReciever<T>>();
                    if let Some(event_reciever) = event_reciever {
                        // invoke event
                        event_reciever.dequeue_event(game_state, &mut self.world, event_queue, &this_event);

                        // add any new events to the list of iterated events
                        to_append.extend(event_queue.drain_queued_events::<T>());
                    }
                }
            }

            // remove
            event.remove(0);
            // add any new
            event.extend(to_append);
        }
        for ecs_system in self.ecs_systems.iter_mut() {
            if !ecs_system.1 {
                continue;
            }
            // do a network mode check
            let run_on_network_modes = ecs_system.0.run_on_instance(game_state);
            let this_network_mode = &game_state.network_mode;

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
    T: Clone,
{
    ecs_systems_eventless: Vec<(Box<dyn ECSSystemEventless>, bool)>,
    scene: World,
    asset_loader: AssetLoader,
    event_queue: EventQueue,
    phantom_data: PhantomData<T>,
    child_init: bool,
    game_instance: Vec<GameplayInstance>,
    constructors: Vec<fn() -> Box<dyn ECSSystemEventless>>,
}

impl<T> SystemComponentDefaultGameplay<T>
where
    T: Clone,
{
    pub fn new() -> Box<SystemComponentDefaultGameplay<T>> {
        Box::new(SystemComponentDefaultGameplay::<T> {
            game_instance: vec![],
            ecs_systems_eventless: vec![],
            scene: World::new(),
            asset_loader: AssetLoader::new(),
            event_queue: EventQueue::new(),
            phantom_data: PhantomData,
            child_init: false,
            constructors: vec![],
        })
    }
}
impl<T> SystemComponentGameplay for SystemComponentDefaultGameplay<T>
where
    T: 'static + Clone,
{
    fn set_systems(&mut self, ecs_systems_eventless: Vec<fn() -> Box<dyn ECSSystemEventless>>) {
        // let mut systems_eventless_enabled: Vec<(Box<dyn ECSSystemEventless>, bool)> = Vec::new();
        // for x in &ecs_systems_eventless {
        //     systems_eventless_enabled.push((x.clone(), false));
        // }
        for x in ecs_systems_eventless {
            self.constructors.push(x);
        }

        // self.ecs_systems_eventless = systems_eventless_enabled;
    }
}
impl<T> SystemComponent for SystemComponentDefaultGameplay<T>
where
    T: 'static + Clone,
{
    fn order(&self) -> i32 {
        5000
    }
    fn init(&mut self, _: &mut Vec<GameState>) {
        println!("init gameplay");
    }
    fn set_game_mode(&mut self, game_state: &mut Vec<GameState>, game_mode: &core::dumpster_engine::GameMode) {
        for _ in &game_mode.game_instances {
            self.game_instance
                .push(GameplayInstance::new(&self.constructors));
        }
    }
    fn debug(&mut self, game_state: &mut Vec<GameState>, system_queue: &mut Vec<EventQueue>) {
        for i in 0..game_state.len() {
            let game_state = &mut game_state[i];
            let system_queue = &mut system_queue[i];
            // for game_state in game_state {
            for s in self.ecs_systems_eventless.iter_mut() {
                if !s.1 {
                    continue;
                }
                s.0.as_mut()
                    .debug(game_state, &mut self.scene, system_queue);
            }
        }
    }
    fn tick(&mut self, game_state: &mut Vec<GameState>, event_queue: &mut Vec<EventQueue>) {
        for i in 0..game_state.len() {
            // self.game_instance[i].tick::<T>(&mut game_state[i], event_queue);
        }
        for i in 0..game_state.len() {
            let game_state = &mut game_state[i];
            let event_queue = &mut event_queue[i];
            self.game_instance[i].tick::<T>(game_state, event_queue);
        }

        // if !self.child_init {
        //     self.child_init = true;
        //     for gs in game_state.iter_mut() {
        //         for s in self.ecs_systems_eventless.iter_mut() {
        //             s.0.as_mut()
        //                 .init(gs, &mut self.scene, &mut self.event_queue, &mut self.asset_loader);
        //         }
        //     }
        // }
        // // for game_state in game_state {
        // // clear old
        // // self.gameplay_event_queue.evnt_queue.clear();

        // // this needs to be cloned to avoid sharing issues
        // // let mut gameplay_queue = self.gameplay_event_queue.clone();
        // let scene = &self.scene;
        // // sort systems

        // // self.ecs_systems_eventless.sort_by(|a, b| {
        // //     a.0.order(&game_state, &scene)
        // //         .cmp(&b.0.order(&game_state, &scene))
        // // });

        // // create a temp queue to pass in
        // let mut tmp_queue = EventQueue::new();

        // // iterate over each event we queued last frame
        // let mut queued_events = self.event_queue.get_queued_events::<T>().to_vec();
        // while queued_events.len() > 0 {
        //     // dequeue first
        //     let event = &queued_events[0];

        //     // iterate over each system
        //     println!(" num of reciever {}", self.ecs_systems_eventless.len());

        //     for game_state in game_state.iter_mut() {
        //         for boxed_system in self.ecs_systems_eventless.iter_mut() {
        //             if !boxed_system
        //                 .0
        //                 .run_on_instance(game_state)
        //                 .contains(&game_state.network_mode)
        //             {
        //                 // println!(" does not contain for {},  {}", game_state.instance_id, game_state.network_mode);
        //                 continue;
        //             }
        //             // guard - try cast the system to event reciever
        //             let event_reciever = (*boxed_system.0).cast::<dyn EventReciever<T>>();
        //             if let Some(event_reciever) = event_reciever {
        //                 // post the event
        //                 // println!(" does contain for {},  {}", game_state.instance_id, game_state.network_mode);
        //                 event_reciever.dequeue_event(game_state, &mut self.scene, &mut tmp_queue, event);
        //             };
        //         }
        //     }

        //     // remove first
        //     queued_events.remove(0);

        //     // iterate over each new event in tmp queue
        //     for response_event in tmp_queue.get_queued_events::<T>() {
        //         queued_events.push(response_event.clone());
        //     }
        //     // clear old
        //     tmp_queue.clear_queued_events::<T>();
        // }
        // self.event_queue.clear_queued_events::<T>();
        // for game_state in game_state.iter_mut() {
        //     // enable/disable componentes
        //     for s in self.ecs_systems_eventless.iter_mut() {
        //         if !s
        //             .0
        //             .run_on_instance(game_state)
        //             .contains(&game_state.network_mode)
        //         {
        //             continue;
        //         }
        //         let was_enabled = s.1;
        //         let is_enabled = s.0.is_enabled(game_state, &mut self.scene);
        //         if was_enabled && !is_enabled {
        //             s.0.disable(game_state, &mut self.scene, &mut self.event_queue);
        //         }
        //         if !was_enabled && is_enabled {
        //             s.0.enable(game_state, &mut self.scene, &mut self.event_queue);
        //         }
        //         s.1 = is_enabled;
        //     }

        //     // run loops
        //     for s in self.ecs_systems_eventless.iter_mut() {
        //         if !s.1 {
        //             continue;
        //         }
        //         if !s
        //             .0
        //             .run_on_instance(game_state)
        //             .contains(&game_state.network_mode)
        //         {
        //             continue;
        //         }
        //         s.0.as_mut()
        //             .will_tick(game_state, &mut self.scene, &mut self.event_queue);
        //     }
        //     for s in self.ecs_systems_eventless.iter_mut() {
        //         if !s.1 {
        //             continue;
        //         }
        //         if !s
        //             .0
        //             .run_on_instance(game_state)
        //             .contains(&game_state.network_mode)
        //         {
        //             continue;
        //         }
        //         s.0.as_mut()
        //             .tick(game_state, &mut self.scene, &mut self.event_queue);
        //     }
        //     for s in self.ecs_systems_eventless.iter_mut() {
        //         if !s.1 {
        //             continue;
        //         }
        //         if !s
        //             .0
        //             .run_on_instance(game_state)
        //             .contains(&game_state.network_mode)
        //         {
        //             continue;
        //         }
        //         s.0.as_mut()
        //             .did_tick(game_state, &mut self.scene, &mut self.event_queue);
        //     }

        //     // save queue for next frame
        //     // self.gameplay_event_queue = gameplay_queue;

        //     // dequeue commands
        //     // return self.system_event_queue.evnt_queue.as_slices().0;
        // }
    }
}

use std::{marker::PhantomData, vec};
