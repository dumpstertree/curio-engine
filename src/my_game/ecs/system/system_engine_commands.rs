use hecs::World;

use crate::{
    gameplay::game_events::GameEvents,
    system::{
        system_components::gameplay_components::gameplay_component_default::{ECSSystem, EngineCommands, EventQueue},
        system_game_states::state_input::InputState,
    },
    Collections::game_state::GameState,
    Collections::vector3::Vector3,
};

pub struct SystemEngineCommands {
    fullscreen: bool,
    was_down: bool,
}
impl SystemEngineCommands {
    pub fn new() -> Box<SystemEngineCommands> {
        Box::new(SystemEngineCommands {
            fullscreen: false,
            was_down: false,
        })
    }
}
impl ECSSystem<GameEvents> for SystemEngineCommands {
    fn is_enabled(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue<GameEvents>) -> bool {
        true
    }
    fn tick(&mut self, game_state: &mut GameState, world: &mut World, event_queue: &mut EventQueue<GameEvents>) {
        let input = game_state.get_value2::<InputState>();
        if input.esc.is_down {
            event_queue.enqueue_command(EngineCommands::Exit);
        }
        if input.tab.is_down && !self.was_down {
            self.fullscreen = !self.fullscreen;

            if self.fullscreen {
                event_queue.enqueue_command(EngineCommands::Resize(Vector3::new(64.0, 64.0, 0.0)));
            } else {
                event_queue.enqueue_command(EngineCommands::Resize(Vector3::new(1920.0, 1080.0, 0.0)));
            }
        }

        self.was_down = input.tab.is_down;
    }
}
