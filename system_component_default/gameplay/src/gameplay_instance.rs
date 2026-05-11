use curio_core::collections::{
    event_queue::{IGameEvent, Nerve},
    ledger::Ledger,
};
use hecs::World;
use std::{cell::RefCell, collections::HashMap, marker::PhantomData, rc::Rc, vec};

use crate::{
    built_in::impulse::ui_events::UIEvents,
    context_2d::Context2D,
    context_3d::Context3D,
    static_data::{global_ecs::get_global_ecs_instances, global_event_recievers::get_global_event_receivers},
    traits::{habit::Habit, impulse::Impulse, ui_events::IUIEvent, ui_panel::UIPanel},
};

pub struct GameplayInstance<T, U>
where
    T: IGameEvent + Clone + 'static,
    U: IUIEvent + Clone + 'static,
{
    phantom_u: PhantomData<U>,
    has_been_init: bool,
    context_32: Context3D,
    context_2d: Context2D,
    ecs_systems: Vec<(Box<dyn Habit>, bool)>,
    event_recievers: Vec<Box<dyn Impulse<T>>>,

    ui: HashMap<U, Box<dyn UIPanel>>,
}

impl<T, U> GameplayInstance<T, U>
where
    T: IGameEvent + Clone + 'static,
    U: IUIEvent + Clone + 'static,
{
    pub fn new() -> GameplayInstance<T, U> {
        // create the base world for our contexts
        let hecs_world = Rc::new(RefCell::new(World::new()));

        //create the world everything is in
        let context_3d = Context3D::new(hecs_world.clone());
        let context_2d = Context2D::new(hecs_world.clone());

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
            context_32: context_3d,
            context_2d,
            ecs_systems,
            event_recievers,
            has_been_init: false,
            ui: HashMap::new(),
            phantom_u: PhantomData::default(),
        }
    }
    pub fn tick(&mut self, ledger: &mut Ledger, event_queue: &mut Nerve) {
        // if not init -> init
        if !self.has_been_init {
            // flip flag
            self.has_been_init = true;

            // initialize each
            for x in self.ecs_systems.iter_mut() {
                x.0.init(ledger, &mut self.context_32, event_queue);
            }
        }

        // if not enabled -> enable
        for ecs_system in self.ecs_systems.iter_mut() {
            // do a network mode check
            let run_on_network_modes = ecs_system.0.run_on_instance(ledger);
            let this_network_mode = &ledger.network_capabilities.clone().unwrap().privilege;

            // ignore if doesnt run on this instance
            if !run_on_network_modes.contains(this_network_mode) {
                continue;
            }

            // get cur state of enable
            let is_enabled = ecs_system.1;
            let should_be_enabled = ecs_system.0.is_enabled(ledger);

            // enable
            if should_be_enabled && !is_enabled {
                ecs_system
                    .0
                    .enable(ledger, &mut self.context_32, event_queue);
            }

            // disable
            if !should_be_enabled && is_enabled {
                ecs_system
                    .0
                    .disable(ledger, &mut self.context_32, event_queue);
            }

            // save the new value
            ecs_system.1 = should_be_enabled;
        }

        let this_network_mode = &ledger.network_capabilities.clone().unwrap().privilege;

        // update any timed events to get added to the queu before we pull anything
        event_queue.update_timed_events();
        //
        let event = event_queue.drain_queued_events::<UIEvents<U>>();
        for e in event {
            match e {
                UIEvents::Open(x) => {
                    let mut i = x.new_instance();
                    i.init();
                    i.present(ledger, event_queue, &mut self.context_2d);
                    //
                    self.ui.insert(x, i);
                }
                UIEvents::Close(u) => {
                    if let Some(mut x) = self.ui.remove(&u) {
                        x.dismiss(ledger, event_queue, &mut self.context_2d);
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
                if !event_reciever.is_enabled(ledger) {
                    continue;
                }

                // make sure reciever can run on this instance type
                if !event_reciever
                    .run_on_instance(ledger)
                    .contains(this_network_mode)
                {
                    continue;
                }

                // apply the event to the reciever
                event_reciever.dequeue_event(ledger, &mut self.context_32, event_queue, this_event);
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
            let run_on_network_modes = ecs_system.0.run_on_instance(ledger);
            let this_network_mode = &ledger.network_capabilities.clone().unwrap().privilege;

            // ignore if doesnt run on this instance
            if !run_on_network_modes.contains(this_network_mode) {
                continue;
            }
            // debug
            ecs_system
                .0
                .debug(ledger, &mut self.context_32, event_queue);
            // tick
            ecs_system
                .0
                .will_tick(ledger, &mut self.context_32, event_queue);
            ecs_system.0.tick(ledger, &mut self.context_32, event_queue);
            ecs_system
                .0
                .did_tick(ledger, &mut self.context_32, event_queue);
        }

        for x in self.ui.iter_mut() {
            x.1.tick(ledger, event_queue, &mut self.context_2d);
        }
        // dequeue events
    }
}
