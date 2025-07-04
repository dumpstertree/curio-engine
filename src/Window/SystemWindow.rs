use pollster::FutureExt;
use winit::{application::ApplicationHandler, event::WindowEvent};

use crate::game_state::GameState;
use crate::system::system_component::ISystemComponent;
use crate::system_adapters::adapter_system_gpu::SYS_GPU;
use crate::Collections::key_state::KeyState;
use crate::Window::state::{self, State};

pub struct SystemWindow {
    gamestate: GameState,
    components: Vec<Box<dyn ISystemComponent>>,
    state: Option<State>,
}
impl SystemWindow {
    // constructor
    pub fn new(components: Vec<Box<dyn ISystemComponent>>) -> SystemWindow {
        SystemWindow {
            gamestate: GameState::new(),
            components: components,
            state: None,
        }
    }

    pub fn run(&mut self) {
        let mut guard_sys_gpu = SYS_GPU.lock().unwrap();
        let event_loop = guard_sys_gpu.from_window().block_on();
        drop(guard_sys_gpu);

        // self.gamestate.add(KEY_DEVICE_STATE, state);
        self.state = Some(state::State::new().block_on());
        let Some(state) = self.state.as_mut() else {
            return;
        };

        self.components.sort_by(|a, b| a.order().cmp(&b.order()));

        // iterate over each in new order
        for c in self.components.iter_mut() {
            c.init(state, &mut self.gamestate);
        }

        // run
        let _ = event_loop.run_app(self);
    }
}
impl ApplicationHandler<State> for SystemWindow {
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
                // let x = SYS_GPU.lock().unwrap()
                for c in self.components.iter_mut() {
                    c.resize(size.width as f32, size.height as f32);
                }
            }
            WindowEvent::RedrawRequested => {
                for c in self.components.iter_mut() {
                    match &mut self.state {
                        Some(state) => {
                            c.render(state, &mut self.gamestate);
                        }
                        _ => {}
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
