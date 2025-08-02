use std::any::Any;

use crate::system::system_component::ISystemComponent;
use crate::system::system_components::gameplay_components::gameplay_component_default::EngineCommands;
use crate::system_adapters::adapter_system_gpu::SystemGPU;
use crate::Collections::game_state::GameState;
use crate::Collections::key_state::KeyState;
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::{application::ApplicationHandler, event::WindowEvent};

pub struct SystemWindow {
    pause_mode: bool,
    debug_mode: bool,
    gamestate: GameState,
    components: Vec<Box<dyn ISystemComponent>>,
}
impl SystemWindow {
    // constructor
    pub fn new(components: Vec<Box<dyn ISystemComponent>>) -> SystemWindow {
        SystemWindow {
            gamestate: GameState::new(),
            components: components,
            debug_mode: false,
            pause_mode: false,
        }
    }

    pub fn run(&mut self, event_loop: EventLoop<EngineCommands>) {
        self.components.sort_by(|a, b| a.order().cmp(&b.order()));

        // iterate over each in new order
        for c in self.components.iter_mut() {
            c.init(&mut self.gamestate);
        }

        // run
        let _ = event_loop.run_app(self);
    }
}
impl ApplicationHandler<EngineCommands> for SystemWindow {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: EngineCommands) {
        match event {
            EngineCommands::Tick => {
                for c in self.components.iter_mut() {
                    for event in c.tick(&mut self.gamestate) {
                        match event {
                            EngineCommands::Redraw => {
                                // let window = SystemGPU::get_window();
                                // window.request_redraw();
                            }
                            EngineCommands::Resize(vector3) => SystemGPU::set_resolution(vector3.x as i32, vector3.y as i32),
                            EngineCommands::Fullscreen(is_fullscreen) => SystemGPU::set_fullscreen(*is_fullscreen),
                            EngineCommands::Resizable(resizable) => SystemGPU::set_resizable(*resizable),
                            EngineCommands::Cursor(visible) => SystemGPU::set_cursor_visible(*visible),
                            EngineCommands::Exit => event_loop.exit(),
                            EngineCommands::SetDebugMode(active) => self.debug_mode = *active,
                            EngineCommands::SetPauseMode(active) => self.debug_mode = *active,
                            EngineCommands::Tick => println!("Cannot call tick from inside tick!"),
                        }
                    }
                    if self.debug_mode {
                        c.debug(&mut self.gamestate);
                    }
                }
            }
            EngineCommands::Redraw => {
                let window = SystemGPU::get_window();
                window.request_redraw();
            }
            EngineCommands::Exit => todo!(),
            EngineCommands::Resize(vector3) => todo!(),
            EngineCommands::Fullscreen(_) => todo!(),
            EngineCommands::Resizable(_) => todo!(),
            EngineCommands::Cursor(_) => todo!(),
            EngineCommands::SetDebugMode(active) => self.debug_mode = active,
            EngineCommands::SetPauseMode(active) => self.pause_mode = active,
        }
    }
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        // reorder the compnents incase something changed
        self.components.sort_by(|a, b| a.order().cmp(&b.order()));

        // send the raw event
        for c in self.components.iter_mut() {
            c.raw_event(event.clone());
        }
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
                let window = SystemGPU::get_window();
                window.request_redraw();

                let mut events: Vec<EngineCommands> = Vec::new();
                for c in self.components.iter_mut() {
                    let s = c.render(&mut self.gamestate);
                    for x in s {
                        events.push(x.clone());
                    }
                }
                for event in events {
                    self.user_event(event_loop, event);
                }
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                let mut toggle_debug = false;
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

                    if code == KeyCode::Backquote && state == KeyState::Down {
                        toggle_debug = true;
                    }

                    c.input_keyboard(&mut self.gamestate, code, state);
                }
                if toggle_debug {
                    self.user_event(event_loop, EngineCommands::SetDebugMode(!self.debug_mode));
                }
            }
            WindowEvent::CursorMoved { device_id, position } => {
                for c in self.components.iter_mut() {
                    c.input_mouse_position(
                        &mut self.gamestate,
                        crate::Collections::vector3::Vector3::new(position.x as f32, position.y as f32, 0.0),
                    );
                }
            }
            WindowEvent::MouseInput { device_id, state, button } => {
                // match button {
                //     winit::event::MouseButton::Left => todo!(),
                //     winit::event::MouseButton::Right => todo!(),
                //     winit::event::MouseButton::Middle => todo!(),
                //     _ => {}
                // }

                for c in self.components.iter_mut() {
                    c.input_mouse();
                }
            }
            _ => {}
        }
    }
}
