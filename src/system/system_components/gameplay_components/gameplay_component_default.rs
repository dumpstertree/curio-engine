use crate::system::{
    system_component::ISystemComponent,
    system_components::gameplay_component::{self, IGameplayComponent},
};

use hecs::World;

use crate::{game_state::GameState, Window::state::State};

pub struct GameplayComponentDefault<T>
where
    T: Clone,
{
    ecs_systems: Vec<(Box<dyn ECSSystem<T>>, bool)>,
    scene: World,
    event_queue: EventQueue<T>,
}

impl<T> GameplayComponentDefault<T>
where
    T: Clone,
{
    pub fn new(ecs_systems: Vec<Box<dyn ECSSystem<T>>>) -> GameplayComponentDefault<T> {
        let mut systems_enabled: Vec<(Box<dyn ECSSystem<T>>, bool)> = Vec::new();
        for x in ecs_systems {
            systems_enabled.push((x, false));
        }

        GameplayComponentDefault {
            ecs_systems: systems_enabled,
            scene: World::new(),
            event_queue: EventQueue::new(),
        }
    }
}
impl<T> IGameplayComponent for GameplayComponentDefault<T> where T: Clone {}
impl<T> ISystemComponent for GameplayComponentDefault<T>
where
    T: Clone,
{
    fn order(&self) -> i32 {
        5000
    }
    fn init(&mut self, state: &mut crate::Window::state::State, gs: &mut crate::game_state::GameState) {
        println!("init gameplay");
        for s in self.ecs_systems.iter_mut() {
            s.0.as_mut().init(gs, &mut self.scene, &mut self.event_queue);
        }
    }
    fn render(&mut self, state: &mut State, game_state: &mut GameState) {
        // get time state
        let t = game_state.get_time();

        // is elapsed
        if t.should_update {
            // this needs to be cloned to avoid sharing issues
            let mut queue = self.event_queue.clone();

            // dequeue events from previous frame
            while queue.evnt_queue.len() > 0 {
                let Some(event) = queue.dequeue_events() else {
                    break;
                };
                for s in self.ecs_systems.iter_mut() {
                    s.0.as_mut().dequeue_event(game_state, &mut self.scene, &mut queue, &event);
                }
            }
            for s in self.ecs_systems.iter_mut() {
                let was_enabled = s.1;
                let is_enabled = s.0.is_enabled(game_state, &mut self.scene, &mut queue);
                if was_enabled && !is_enabled {
                    s.0.disable(game_state, &mut self.scene, &mut queue);
                }
                if !was_enabled && is_enabled {
                    s.0.enable(game_state, &mut self.scene, &mut queue);
                }
                s.1 = is_enabled;
            }
            // run loops
            for s in self.ecs_systems.iter_mut() {
                if !s.1 {
                    continue;
                }
                s.0.as_mut().will_tick(game_state, &mut self.scene, &mut queue);
            }
            for s in self.ecs_systems.iter_mut() {
                if !s.1 {
                    continue;
                }
                s.0.as_mut().tick(game_state, &mut self.scene, &mut queue);
            }
            for s in self.ecs_systems.iter_mut() {
                if !s.1 {
                    continue;
                }
                s.0.as_mut().did_tick(game_state, &mut self.scene, &mut queue);
            }
            // dequeue commands
            while queue.cmd_queue.len() > 0 {
                let Some(event) = queue.dequeue_commands() else {
                    break;
                };
                match event {
                    EngineCommands::Exit => println!("exit"),
                    EngineCommands::Resize => println!("resize"),
                }
            }

            // save queue for next frame
            self.event_queue = queue;
        }
    }
}

pub trait ECSSystem<T>
where
    T: Clone,
{
    fn is_enabled(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue<T>) -> bool;
    fn enable(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue<T>) {}
    fn disable(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue<T>) {}
    fn init(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue<T>) {}
    fn will_tick(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue<T>) {}
    fn tick(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue<T>) {}
    fn did_tick(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue<T>) {}
    fn dequeue_event(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue<T>, event: &T) {}
}

use std::collections::{HashMap, VecDeque};
#[derive(Clone)]
pub enum EngineCommands {
    Exit,
    Resize,
}
#[derive(Clone)]
pub struct EventQueue<T>
where
    T: Clone,
{
    evnt_queue: VecDeque<T>,
    cmd_queue: VecDeque<EngineCommands>,
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
    fn dequeue_commands(&mut self) -> Option<EngineCommands> {
        self.cmd_queue.pop_front()
    }
    pub fn enqueue_command(&mut self, command: EngineCommands) {
        self.cmd_queue.push_back(command);
    }
    pub fn new() -> EventQueue<T> {
        EventQueue {
            evnt_queue: VecDeque::new(),
            cmd_queue: VecDeque::new(),
        }
    }
}
