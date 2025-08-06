use crate::{
    gameplay::ecs::traits::{ecs_event_reciever::EventReciever, ecs_system::ECSSystemEventless},
    system::{system_component::ISystemComponent, system_components::gameplay_component::IGameplayComponent},
    Collections::{event_queue::EventQueue2, game_state::GameState, vector3::Vector3},
    IO::AssetLoader::AssetLoader,
};
use hecs::World;
use intertrait::{cast::CastMut, CastFrom};

pub struct GameplayComponentDefault<T>
where
    T: Clone,
{
    ecs_systems_eventless: Vec<(Box<dyn ECSSystemEventless>, bool)>,
    scene: World,
    asset_loader: AssetLoader,
    event_queue: EventQueue2,
    phantom_data: PhantomData<T>,
}

impl<T> GameplayComponentDefault<T>
where
    T: Clone,
{
    pub fn new(ecs_systems_eventless: Vec<Box<dyn ECSSystemEventless>>) -> GameplayComponentDefault<T> {
        let mut systems_eventless_enabled: Vec<(Box<dyn ECSSystemEventless>, bool)> = Vec::new();
        for x in ecs_systems_eventless {
            systems_eventless_enabled.push((x, false));
        }

        GameplayComponentDefault::<T> {
            ecs_systems_eventless: systems_eventless_enabled,
            scene: World::new(),
            asset_loader: AssetLoader::new(),
            event_queue: EventQueue2::new(),
            phantom_data: PhantomData,
        }
    }
}
impl<T> IGameplayComponent for GameplayComponentDefault<T> where T: 'static + Clone {}
impl<T> ISystemComponent for GameplayComponentDefault<T>
where
    T: 'static + Clone,
{
    fn order(&self) -> i32 {
        5000
    }
    fn init(&mut self, gs: &mut GameState) {
        println!("init gameplay");
        for s in self.ecs_systems_eventless.iter_mut() {
            s.0.as_mut()
                .init(gs, &mut self.scene, &mut self.event_queue, &mut self.asset_loader);
        }
    }
    fn debug(&mut self, game_state: &mut GameState, system_queue: &mut EventQueue2) {
        for s in self.ecs_systems_eventless.iter_mut() {
            if !s.1 {
                continue;
            }
            s.0.as_mut()
                .debug(game_state, &mut self.scene, system_queue);
        }
    }
    fn tick(&mut self, game_state: &mut GameState, system_queue: &mut EventQueue2) {
        // clear old
        // self.gameplay_event_queue.evnt_queue.clear();

        // this needs to be cloned to avoid sharing issues
        // let mut gameplay_queue = self.gameplay_event_queue.clone();
        let scene = &self.scene;
        // sort systems

        self.ecs_systems_eventless.sort_by(|a, b| {
            a.0.order(&game_state, &scene)
                .cmp(&b.0.order(&game_state, &scene))
        });

        // create a temp queue to pass in
        let mut tmp_queue = EventQueue2::new();

        // iterate over each event we queued last frame
        let mut queued_events = self.event_queue.get_queued_events::<T>().to_vec();
        while queued_events.len() > 0 {
            // dequeue first
            let event = &queued_events[0];

            // iterate over each system
            for boxed_system in self.ecs_systems_eventless.iter_mut() {
                // guard - try cast the system to event reciever
                let event_reciever = (*boxed_system.0).cast::<dyn EventReciever<T>>();
                if let Some(event_reciever) = event_reciever {
                    // post the event
                    event_reciever.dequeue_event(game_state, &mut self.scene, &mut tmp_queue, event);
                };
            }

            // remove first
            queued_events.remove(0);

            // iterate over each new event in tmp queue
            for response_event in tmp_queue.get_queued_events::<T>() {
                queued_events.push(response_event.clone());
            }
            // clear old
            tmp_queue.clear_queued_events::<T>();
        }
        self.event_queue.clear_queued_events::<T>();

        // enable/disable componentes
        for s in self.ecs_systems_eventless.iter_mut() {
            let was_enabled = s.1;
            let is_enabled = s.0.is_enabled(game_state, &mut self.scene);
            if was_enabled && !is_enabled {
                s.0.disable(game_state, &mut self.scene, &mut self.event_queue);
            }
            if !was_enabled && is_enabled {
                s.0.enable(game_state, &mut self.scene, &mut self.event_queue);
            }
            s.1 = is_enabled;
        }

        // run loops
        for s in self.ecs_systems_eventless.iter_mut() {
            if !s.1 {
                continue;
            }
            s.0.as_mut()
                .will_tick(game_state, &mut self.scene, &mut self.event_queue);
        }
        for s in self.ecs_systems_eventless.iter_mut() {
            if !s.1 {
                continue;
            }
            s.0.as_mut()
                .tick(game_state, &mut self.scene, &mut self.event_queue);
        }
        for s in self.ecs_systems_eventless.iter_mut() {
            if !s.1 {
                continue;
            }
            s.0.as_mut()
                .did_tick(game_state, &mut self.scene, &mut self.event_queue);
        }

        // save queue for next frame
        // self.gameplay_event_queue = gameplay_queue;

        // dequeue commands
        // return self.system_event_queue.evnt_queue.as_slices().0;
    }
}

use std::{
    collections::VecDeque,
    hash::{Hash, Hasher},
    marker::PhantomData,
};
