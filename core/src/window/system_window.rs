use crate::collections::event_queue::EventQueue;
use crate::collections::game_state::GameState;
use crate::collections::key_state::KeyState;
use crate::dumpster_engine::GameMode;
use crate::events::engine_commands::EngineCommands;
use crate::input::key_code::KeyCode;
use crate::system::system_component::SystemComponent;
use crate::system::system_game_state::IState;
use crate::system::system_game_states::state_debug::StateDebug;
use crate::system::system_game_states::state_screeen::StateScreen;
use crate::system_adapters::adapter_system_gpu::SystemGPU;
use winit::event_loop::EventLoop;
use winit::{application::ApplicationHandler, event::WindowEvent};

pub struct SystemWindow {
    system_event_queue: EventQueue,
    gamestate: GameState,
    components: Vec<Box<dyn SystemComponent>>,
    game_mode: GameMode,
}
impl SystemWindow {
    // constructor
    pub fn new(components: Vec<Box<dyn SystemComponent>>, game_mode: GameMode) -> SystemWindow {
        SystemWindow {
            system_event_queue: EventQueue::new(),
            gamestate: GameState::new(),
            components: components,
            game_mode: game_mode,
        }
    }

    // run
    pub fn run(&mut self, event_loop: EventLoop<EngineCommands>) {
        self.components.sort_by(|a, b| a.order().cmp(&b.order()));

        // iterate over each in new order
        for c in self.components.iter_mut() {
            c.init(&mut self.gamestate);
        }

        for c in self.components.iter_mut() {
            c.set_game_mode(&self.game_mode);
        }

        // run
        let _ = event_loop.run_app(self);
    }

    // convert input events
    fn convert_winit_keycode(winit_key: winit::keyboard::KeyCode) -> Option<KeyCode> {
        match winit_key {
            winit::keyboard::KeyCode::Backquote => return Some(KeyCode::Backquote),
            winit::keyboard::KeyCode::KeyW => return Some(KeyCode::KeyW),
            winit::keyboard::KeyCode::KeyA => return Some(KeyCode::KeyA),
            winit::keyboard::KeyCode::KeyS => return Some(KeyCode::KeyS),
            winit::keyboard::KeyCode::KeyD => return Some(KeyCode::KeyD),
            winit::keyboard::KeyCode::KeyI => return Some(KeyCode::KeyI),
            winit::keyboard::KeyCode::KeyJ => return Some(KeyCode::KeyJ),
            winit::keyboard::KeyCode::KeyK => return Some(KeyCode::KeyK),
            winit::keyboard::KeyCode::KeyL => return Some(KeyCode::KeyL),
            _ => return None,
        }
    }
    fn convert_winit_mousecode(winit_key: winit::event::MouseButton) -> Option<KeyCode> {
        match winit_key {
            winit::event::MouseButton::Left => return Some(KeyCode::MousePrimary),
            winit::event::MouseButton::Right => return Some(KeyCode::MouseSecondary),
            _ => return None,
        }
    }
}

impl ApplicationHandler<EngineCommands> for SystemWindow {
    fn resumed(&mut self, _: &winit::event_loop::ActiveEventLoop) {}

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
                    let queue = self
                        .system_event_queue
                        .get_queued_events::<EngineCommands>();
                    for event in queue {
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
                            EngineCommands::SetNumInputs(_) => todo!(),
                            EngineCommands::SetNumScreens(_) => todo!(),
                            EngineCommands::SetHost() => todo!(),
                            EngineCommands::SetPeer() => todo!(),
                        }
                    }
                    let _ = &self
                        .system_event_queue
                        .clear_queued_events::<EngineCommands>();
                }
            }
            EngineCommands::Redraw => {}
            EngineCommands::Exit => todo!(),
            EngineCommands::Resize(_) => todo!(),
            EngineCommands::Fullscreen(_) => todo!(),
            EngineCommands::Resizable(_) => todo!(),
            EngineCommands::Cursor(_) => todo!(),
            EngineCommands::SetDebugMode(active) => self
                .gamestate
                .edit::<StateDebug>(|x| x.is_inspecting = active),
            EngineCommands::SetPauseMode(active) => self.gamestate.edit::<StateDebug>(|x| x.is_paused = active),
            EngineCommands::SetNumInputs(_num) => todo!(),
            EngineCommands::SetNumScreens(_num) => todo!(),
            EngineCommands::SetHost() => todo!(),
            EngineCommands::SetPeer() => todo!(),
        }
    }
    fn window_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, _: winit::window::WindowId, event: winit::event::WindowEvent) {
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
                    c.application_quit();
                }
            }
            WindowEvent::Resized(size) => {
                for c in self.components.iter_mut() {
                    c.application_resize(size.width as f32, size.height as f32);
                }
            }
            WindowEvent::RedrawRequested => {
                let mut events: Vec<EngineCommands> = Vec::new();
                for c in self.components.iter_mut() {
                    let s = c.refresh(&mut self.gamestate);
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
                device_id: _,
                is_synthetic: _,
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

                    if code == winit::keyboard::KeyCode::Backquote && state == KeyState::Down {
                        toggle_debug = true;
                    }

                    let Some(code) = SystemWindow::convert_winit_keycode(code) else {
                        return;
                    };

                    c.input_button(&mut self.gamestate, code, state);
                }
                if toggle_debug {
                    self.user_event(
                        event_loop,
                        EngineCommands::SetDebugMode(!self.gamestate.get_value2::<StateDebug>().is_inspecting),
                    );
                }
            }
            WindowEvent::CursorMoved { device_id: _, position } => {
                for c in self.components.iter_mut() {
                    c.input_axis(
                        &mut self.gamestate,
                        crate::input::axis_code::AxisCode::Cursor,
                        crate::collections::vector3::Vector3::new(position.x as f32, position.y as f32, 0.0),
                    );
                }
            }
            WindowEvent::MouseInput { device_id: _, state, button } => {
                let state = if state.is_pressed() { KeyState::Down } else { KeyState::Up };

                let Some(code) = SystemWindow::convert_winit_mousecode(button) else {
                    return;
                };

                for c in self.components.iter_mut() {
                    c.input_button(&mut self.gamestate, code, state);
                }
            }
            _ => {}
        }
    }
}
