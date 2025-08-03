use std::any::Any;

use crate::gameplay::ecs::system;
use crate::system::system_component::ISystemComponent;
use crate::system::system_components::gameplay_components::gameplay_component_default::{EngineCommands, EventQueue};
use crate::system::system_game_state::IState;
use crate::system::system_game_states::state_debug::StateDebug;
use crate::system::system_game_states::state_screeen::StateScreen;
use crate::system_adapters;
use crate::system_adapters::adapter_system_gpu::SystemGPU;
use crate::Collections::game_state::GameState;
use crate::Collections::key_state::KeyState;
use winit::event::{self, KeyEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::CursorGrabMode;
use winit::{application::ApplicationHandler, event::WindowEvent};

pub struct SystemWindow {
    system_event_queue: EventQueue<EngineCommands>,
    gamestate: GameState,
    components: Vec<Box<dyn ISystemComponent>>,
}
impl SystemWindow {
    // constructor
    pub fn new(components: Vec<Box<dyn ISystemComponent>>) -> SystemWindow {
        SystemWindow {
            system_event_queue: EventQueue::new(),
            gamestate: GameState::new(),
            components: components,
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
                // update forced packets
                self.gamestate
                    .set_value2::<StateScreen>(StateScreen::default());

                //
                for c in self.components.iter_mut() {
                    // run tick on all
                    c.tick(&mut self.gamestate, &mut self.system_event_queue);

                    // if debuging run debug
                    if self.gamestate.get_value2::<StateDebug>().is_inspecting {
                        c.debug(&mut self.gamestate, &mut self.system_event_queue);
                    }

                    // invoke all events
                    for event in &self.system_event_queue.evnt_queue {
                        match event {
                            EngineCommands::Redraw => {}
                            EngineCommands::Resize(vector3) => SystemGPU::set_resolution(vector3.x as i32, vector3.y as i32),
                            EngineCommands::Fullscreen(is_fullscreen) => SystemGPU::set_fullscreen(*is_fullscreen),
                            EngineCommands::Resizable(resizable) => SystemGPU::set_resizable(*resizable),
                            EngineCommands::Cursor(visible) => SystemGPU::set_cursor_visible(*visible),
                            EngineCommands::Exit => event_loop.exit(),
                            EngineCommands::SetDebugMode(active) => self
                                .gamestate
                                .edit::<StateDebug>(|x| x.is_inspecting = *active),
                            EngineCommands::SetPauseMode(active) => self.gamestate.edit::<StateDebug>(|x| x.is_paused = *active),
                            EngineCommands::Tick => println!("Cannot call tick from inside tick!"),
                        }
                    }
                    let _ = &self.system_event_queue.evnt_queue.clear();
                }
            }
            EngineCommands::Redraw => {}
            EngineCommands::Exit => todo!(),
            EngineCommands::Resize(vector3) => todo!(),
            EngineCommands::Fullscreen(_) => todo!(),
            EngineCommands::Resizable(_) => todo!(),
            EngineCommands::Cursor(_) => todo!(),
            EngineCommands::SetDebugMode(active) => self
                .gamestate
                .edit::<StateDebug>(|x| x.is_inspecting = active),
            EngineCommands::SetPauseMode(active) => self.gamestate.edit::<StateDebug>(|x| x.is_paused = active),
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

                let window = SystemGPU::get_window();
                window.request_redraw();
            }

            WindowEvent::KeyboardInput {
                device_id,
                is_synthetic,
                event,
            } => {
                if event.repeat {
                    return;
                }
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
                    self.user_event(
                        event_loop,
                        EngineCommands::SetDebugMode(!self.gamestate.get_value2::<StateDebug>().is_inspecting),
                    );
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
                let state = if state.is_pressed() { KeyState::Down } else { KeyState::Up };

                for c in self.components.iter_mut() {
                    c.input_mouse(button, state);
                }
            }
            _ => {}
        }
    }
}
