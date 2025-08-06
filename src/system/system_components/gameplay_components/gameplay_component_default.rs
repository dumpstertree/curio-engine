use crate::{
    dumpster_engine::EventReciever,
    system::{system_component::ISystemComponent, system_components::gameplay_component::IGameplayComponent},
    Collections::{
        game_state::{AnyMap, GameState},
        vector3::Vector3,
    },
    IO::AssetLoader::AssetLoader,
};

use egui::util::id_type_map::TypeId;
use hecs::World;
use intertrait::{cast::CastMut, CastFrom};

pub struct GameplayComponentDefault<T>
where
    T: Clone,
{
    ecs_systems_eventless: Vec<(Box<dyn ECSSystemEventless>, bool)>,
    scene: World,
    gameplay_event_queue: EventQueue<T>,
    asset_loader: AssetLoader,
    event_queue: EventQueue2,
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

        GameplayComponentDefault {
            ecs_systems_eventless: systems_eventless_enabled,
            scene: World::new(),
            gameplay_event_queue: EventQueue::new(),
            asset_loader: AssetLoader::new(),
            event_queue: EventQueue2::new(),
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
                .init(gs, &mut self.scene, &mut self.asset_loader);
        }
    }
    fn debug(&mut self, game_state: &mut GameState, system_queue: &mut EventQueue<EngineCommands>) {
        for s in self.ecs_systems_eventless.iter_mut() {
            if !s.1 {
                continue;
            }
            s.0.as_mut()
                .debug(game_state, &mut self.scene, system_queue);
        }
    }
    fn tick(&mut self, game_state: &mut GameState, system_queue: &mut EventQueue<EngineCommands>) {
        // clear old
        self.gameplay_event_queue.evnt_queue.clear();

        // this needs to be cloned to avoid sharing issues
        let mut gameplay_queue = self.gameplay_event_queue.clone();
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
        for s in self.ecs_systems_eventless.iter_mut() {
            let was_enabled = s.1;
            let is_enabled = s.0.is_enabled(game_state, &mut self.scene);
            if was_enabled && !is_enabled {
                s.0.disable(game_state, &mut self.scene);
            }
            if !was_enabled && is_enabled {
                s.0.enable(game_state, &mut self.scene);
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
            s.0.as_mut().tick(game_state, &mut self.scene);
        }
        for s in self.ecs_systems_eventless.iter_mut() {
            if !s.1 {
                continue;
            }
            s.0.as_mut().did_tick(game_state, &mut self.scene);
        }

        // save queue for next frame
        self.gameplay_event_queue = gameplay_queue;

        // dequeue commands
        // return self.system_event_queue.evnt_queue.as_slices().0;
    }
}

pub trait ECSSystemEventless: CastFrom {
    fn order(&self, game_state: &GameState, world: &World) -> i32 {
        0
    }
    fn debug(&mut self, game_state: &mut GameState, world: &mut World, system_event_queue: &mut EventQueue<EngineCommands>) {}
    fn is_enabled(&mut self, game_state: &mut GameState, world: &mut World) -> bool;
    fn enable(&mut self, game_state: &mut GameState, world: &mut World) {}
    fn disable(&mut self, game_state: &mut GameState, world: &mut World) {}
    fn init(&mut self, game_state: &mut GameState, world: &mut World, asset_loader: &mut AssetLoader) {}
    fn will_tick(&mut self, game_state: &mut GameState, world: &mut World, queue: &mut EventQueue2) {}
    fn tick(&mut self, game_state: &mut GameState, world: &mut World) {}
    fn did_tick(&mut self, game_state: &mut GameState, world: &mut World) {}
}

use std::{
    any::{type_name, Any},
    collections::VecDeque,
    hash::{Hash, Hasher},
};
#[derive(Clone)]
pub enum EngineCommands {
    Redraw,
    Tick,
    Exit,
    Resize(Vector3),
    Fullscreen(bool),
    Resizable(bool),
    Cursor(bool),
    SetDebugMode(bool),
    SetPauseMode(bool),
}
#[derive(Clone)]
pub struct EventQueue<T>
where
    T: Clone,
{
    pub evnt_queue: VecDeque<T>,
}
impl<T> EventQueue<T>
where
    T: Clone,
{
    fn dequeue_events(&mut self) -> Option<T> {
        self.evnt_queue.pop_front()
    }
    pub fn enqueue_event(&mut self, event: T) {
        self.evnt_queue.push_back(event);
    }

    pub fn new() -> EventQueue<T> {
        EventQueue { evnt_queue: VecDeque::new() }
    }
}

// pub struct EventQueue2 {
//     queue
// }
// impl EventQueue2 {

//     pub fn enqueue<T>() {}
// }
use std::collections::hash_map::DefaultHasher;

pub struct EventQueue2 {
    cache: AnyMap<i32>,
    hasher: DefaultHasher,
}
impl EventQueue2 {
    pub fn new() -> EventQueue2 {
        EventQueue2 {
            cache: AnyMap::<i32>::default(),
            hasher: DefaultHasher::new(),
        }
    }

    fn type_id_to_i32<T: 'static>() -> i32 {
        let mut hasher = DefaultHasher::new();
        TypeId::of::<T>().hash(&mut hasher);
        (hasher.finish() & 0xFFFF_FFFF) as i32 // Safe truncation
    }

    pub fn enqueue_event<T: 'static>(&mut self, val: T)
    where
        T: Clone,
    {
        let id = EventQueue2::type_id_to_i32::<T>();
        if let Some(vec) = self.cache.get_mut::<Vec<T>, i32>(&id) {
            vec.push(val);
        } else {
            self.cache.insert::<Vec<T>>(id, vec![val]);
        }
    }
    pub fn get_queued_events<T: 'static>(&self) -> &[T]
    where
        T: Clone,
    {
        let id = EventQueue2::type_id_to_i32::<T>();
        if let Some(x) = self.cache.get::<Vec<T>, i32>(&id) {
            x.as_slice()
        } else {
            &[]
        }
    }
    pub fn clear_queued_events<T: 'static>(&mut self) {
        let id = EventQueue2::type_id_to_i32::<T>();
        if let Some(x) = self.cache.get_mut::<Vec<T>, i32>(&id) {
            x.clear();
        }
    }
}
