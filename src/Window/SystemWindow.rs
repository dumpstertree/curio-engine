use crate::system::system_component::ISystemComponent;
use crate::system::system_components::gameplay_components::gameplay_component_default::EngineCommands;
use crate::system_adapters::adapter_system_gpu::{CustomEvents, SystemGPU};
use crate::Collections::game_state::GameState;
use crate::Collections::key_state::KeyState;
use winit::event_loop::EventLoop;
use winit::{application::ApplicationHandler, event::WindowEvent};

pub struct SystemWindow {
    gamestate: GameState,
    components: Vec<Box<dyn ISystemComponent>>,
}
impl SystemWindow {
    // constructor
    pub fn new(components: Vec<Box<dyn ISystemComponent>>) -> SystemWindow {
        SystemWindow {
            gamestate: GameState::new(),
            components: components,
        }
    }

    pub fn run(&mut self, event_loop: EventLoop<CustomEvents>) {
        self.components.sort_by(|a, b| a.order().cmp(&b.order()));

        // iterate over each in new order
        for c in self.components.iter_mut() {
            c.init(&mut self.gamestate);
        }

        // run
        let _ = event_loop.run_app(self);
    }
}
impl ApplicationHandler<CustomEvents> for SystemWindow {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        // reorder the compnents incase something changed
        self.components.sort_by(|a, b| a.order().cmp(&b.order()));

        // match the event
        match event {
            WindowEvent::CloseRequested => {
                for c in self.components.iter_mut() {
                    c.quit();
                }
            }
            WindowEvent::Resized(size) => {
                for c in self.components.iter_mut() {
                    c.resize(size.width as f32, size.height as f32);
                }
            }
            WindowEvent::RedrawRequested => {
                for c in self.components.iter_mut() {
                    for event in c.render(&mut self.gamestate) {
                        match event {
                            EngineCommands::Resize(vector3) => SystemGPU::set_resolution(vector3.x as i32, vector3.y as i32),
                            EngineCommands::Fullscreen(is_fullscreen) => SystemGPU::set_fullscreen(*is_fullscreen),
                            EngineCommands::Resizable(resizable) => SystemGPU::set_resizable(*resizable),
                            EngineCommands::Cursor(visible) => SystemGPU::set_cursor_visible(*visible),
                            EngineCommands::Exit => event_loop.exit(),
                        }
                    }
                }
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                for c in self.components.iter_mut() {
                    // convert to key code
                    let code;
                    match event.physical_key {
                        winit::keyboard::PhysicalKey::Code(key_code) => {
                            code = key_code;
                        }
                        _ => return,
                    }

                    // next
                    let state = if event.state.is_pressed() { KeyState::Down } else { KeyState::Up };
                    c.input_keyboard(&mut self.gamestate, code, state);
                }
            }
            WindowEvent::MouseInput { device_id, state, button } => {
                for c in self.components.iter_mut() {
                    c.input_mouse();
                }
            }
            _ => {}
        }
    }
}
