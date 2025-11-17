use crate::collections::event_queue::EventQueue;
use crate::collections::game_state::GameState;
use crate::collections::key_state::KeyState;
use crate::dumpster_engine::GameMode;
use crate::events::engine_commands::EngineCommands;
use crate::input::key_code::ButtonCode;
use crate::random::Random;
use crate::system::system_component::SystemComponent;
// use crate::system::system_game_states::state_debug::StateDebug;
// use crate::system::system_game_states::state_screeen::StateScreen;
use crate::system_adapters::adapter_system_gpu::SystemGPU;
use winit::event_loop::EventLoop;
use winit::{application::ApplicationHandler, event::WindowEvent};

pub struct SystemWindow {
    system_event_queue: Vec<EventQueue>,
    gamestate: Vec<GameState>,
    components: Vec<Box<dyn SystemComponent>>,
    game_mode: GameMode,
}
impl SystemWindow {
    // constructor
    pub fn new(components: Vec<Box<dyn SystemComponent>>, game_mode: GameMode) -> SystemWindow {
        let mut all_instance_id = vec![];
        for _ in &game_mode.game_instances {
            all_instance_id.push(Random::range_int(-999, 999));
        }
        let mut states = vec![];
        for i in 0..game_mode.game_instances.len() {
            let x = &game_mode.game_instances[i];
            let id = *(&all_instance_id[i]);
            states.push(GameState::new(x.network_mode.clone(), id, all_instance_id.clone()));
        }
        let mut events = vec![];
        for i in 0..game_mode.game_instances.len() {
            events.push(EventQueue::new());
        }

        SystemWindow {
            system_event_queue: events,
            gamestate: states,
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
            c.set_game_mode(&mut self.gamestate, &self.game_mode);
        }

        // run
        let _ = event_loop.run_app(self);
    }

    // convert input events
    fn convert_winit_keycode(winit_key: winit::keyboard::KeyCode) -> Option<ButtonCode> {
        match winit_key {
            winit::keyboard::KeyCode::Backquote => return Some(ButtonCode::Backquote),
            winit::keyboard::KeyCode::KeyW => return Some(ButtonCode::KeyW),
            winit::keyboard::KeyCode::KeyA => return Some(ButtonCode::KeyA),
            winit::keyboard::KeyCode::KeyS => return Some(ButtonCode::KeyS),
            winit::keyboard::KeyCode::KeyD => return Some(ButtonCode::KeyD),
            winit::keyboard::KeyCode::KeyI => return Some(ButtonCode::KeyI),
            winit::keyboard::KeyCode::KeyJ => return Some(ButtonCode::KeyJ),
            winit::keyboard::KeyCode::KeyK => return Some(ButtonCode::KeyK),
            winit::keyboard::KeyCode::KeyL => return Some(ButtonCode::KeyL),
            winit::keyboard::KeyCode::Backslash => return Some(ButtonCode::Backslash),
            winit::keyboard::KeyCode::BracketLeft => return Some(ButtonCode::BracketLeft),
            winit::keyboard::KeyCode::BracketRight => return Some(ButtonCode::BracketRight),
            winit::keyboard::KeyCode::Comma => return Some(ButtonCode::Comma),
            winit::keyboard::KeyCode::Digit0 => return Some(ButtonCode::Digit0),
            winit::keyboard::KeyCode::Digit1 => return Some(ButtonCode::Digit1),
            winit::keyboard::KeyCode::Digit2 => return Some(ButtonCode::Digit2),
            winit::keyboard::KeyCode::Digit3 => return Some(ButtonCode::Digit3),
            winit::keyboard::KeyCode::Digit4 => return Some(ButtonCode::Digit4),
            winit::keyboard::KeyCode::Digit5 => return Some(ButtonCode::Digit5),
            winit::keyboard::KeyCode::Digit6 => return Some(ButtonCode::Digit6),
            winit::keyboard::KeyCode::Digit7 => return Some(ButtonCode::Digit7),
            winit::keyboard::KeyCode::Digit8 => return Some(ButtonCode::Digit8),
            winit::keyboard::KeyCode::Digit9 => return Some(ButtonCode::Digit9),
            winit::keyboard::KeyCode::Equal => return Some(ButtonCode::Equal),
            winit::keyboard::KeyCode::IntlBackslash => return Some(ButtonCode::IntlBackslash),
            winit::keyboard::KeyCode::IntlRo => return Some(ButtonCode::IntlRo),
            winit::keyboard::KeyCode::IntlYen => return Some(ButtonCode::IntlYen),
            winit::keyboard::KeyCode::KeyB => return Some(ButtonCode::KeyB),
            winit::keyboard::KeyCode::KeyC => return Some(ButtonCode::KeyC),
            winit::keyboard::KeyCode::KeyE => return Some(ButtonCode::KeyE),
            winit::keyboard::KeyCode::KeyF => return Some(ButtonCode::KeyF),
            winit::keyboard::KeyCode::KeyG => return Some(ButtonCode::KeyG),
            winit::keyboard::KeyCode::KeyH => return Some(ButtonCode::KeyH),
            winit::keyboard::KeyCode::KeyM => return Some(ButtonCode::KeyM),
            winit::keyboard::KeyCode::KeyN => return Some(ButtonCode::KeyN),
            winit::keyboard::KeyCode::KeyO => return Some(ButtonCode::KeyO),
            winit::keyboard::KeyCode::KeyP => return Some(ButtonCode::KeyP),
            winit::keyboard::KeyCode::KeyQ => return Some(ButtonCode::KeyQ),
            winit::keyboard::KeyCode::KeyR => return Some(ButtonCode::KeyR),
            winit::keyboard::KeyCode::KeyT => return Some(ButtonCode::KeyT),
            winit::keyboard::KeyCode::KeyU => return Some(ButtonCode::KeyU),
            winit::keyboard::KeyCode::KeyV => return Some(ButtonCode::KeyV),
            winit::keyboard::KeyCode::KeyX => return Some(ButtonCode::KeyX),
            winit::keyboard::KeyCode::KeyY => return Some(ButtonCode::KeyY),
            winit::keyboard::KeyCode::KeyZ => return Some(ButtonCode::KeyZ),
            winit::keyboard::KeyCode::Minus => return Some(ButtonCode::Minus),
            winit::keyboard::KeyCode::Period => return Some(ButtonCode::Period),
            winit::keyboard::KeyCode::Quote => return Some(ButtonCode::Quote),
            winit::keyboard::KeyCode::Semicolon => return Some(ButtonCode::Semicolon),
            winit::keyboard::KeyCode::Slash => return Some(ButtonCode::Slash),
            winit::keyboard::KeyCode::AltLeft => return Some(ButtonCode::AltLeft),
            winit::keyboard::KeyCode::AltRight => return Some(ButtonCode::AltRight),
            winit::keyboard::KeyCode::Backspace => return Some(ButtonCode::Backspace),
            winit::keyboard::KeyCode::CapsLock => return Some(ButtonCode::CapsLock),
            winit::keyboard::KeyCode::ContextMenu => return Some(ButtonCode::ContextMenu),
            winit::keyboard::KeyCode::ControlLeft => return Some(ButtonCode::ControlLeft),
            winit::keyboard::KeyCode::ControlRight => return Some(ButtonCode::ControlRight),
            winit::keyboard::KeyCode::Enter => return Some(ButtonCode::Enter),
            winit::keyboard::KeyCode::SuperLeft => return Some(ButtonCode::SuperLeft),
            winit::keyboard::KeyCode::SuperRight => return Some(ButtonCode::SuperRight),
            winit::keyboard::KeyCode::ShiftLeft => return Some(ButtonCode::ShiftLeft),
            winit::keyboard::KeyCode::ShiftRight => return Some(ButtonCode::ShiftRight),
            winit::keyboard::KeyCode::Space => return Some(ButtonCode::Space),
            winit::keyboard::KeyCode::Tab => return Some(ButtonCode::Tab),
            winit::keyboard::KeyCode::Convert => return Some(ButtonCode::Convert),
            winit::keyboard::KeyCode::KanaMode => return Some(ButtonCode::KanaMode),
            winit::keyboard::KeyCode::Lang1 => return Some(ButtonCode::Lang1),
            winit::keyboard::KeyCode::Lang2 => return Some(ButtonCode::Lang2),
            winit::keyboard::KeyCode::Lang3 => return Some(ButtonCode::Lang3),
            winit::keyboard::KeyCode::Lang4 => return Some(ButtonCode::Lang4),
            winit::keyboard::KeyCode::Lang5 => return Some(ButtonCode::Lang5),
            winit::keyboard::KeyCode::NonConvert => return Some(ButtonCode::NonConvert),
            winit::keyboard::KeyCode::Delete => return Some(ButtonCode::Delete),
            winit::keyboard::KeyCode::End => return Some(ButtonCode::End),
            winit::keyboard::KeyCode::Help => return Some(ButtonCode::Help),
            winit::keyboard::KeyCode::Home => return Some(ButtonCode::Home),
            winit::keyboard::KeyCode::Insert => return Some(ButtonCode::Insert),
            winit::keyboard::KeyCode::PageDown => return Some(ButtonCode::PageDown),
            winit::keyboard::KeyCode::PageUp => return Some(ButtonCode::PageUp),
            winit::keyboard::KeyCode::ArrowDown => return Some(ButtonCode::ArrowDown),
            winit::keyboard::KeyCode::ArrowLeft => return Some(ButtonCode::ArrowLeft),
            winit::keyboard::KeyCode::ArrowRight => return Some(ButtonCode::ArrowRight),
            winit::keyboard::KeyCode::ArrowUp => return Some(ButtonCode::ArrowUp),
            winit::keyboard::KeyCode::NumLock => return Some(ButtonCode::NumLock),
            winit::keyboard::KeyCode::Numpad0 => return Some(ButtonCode::Numpad0),
            winit::keyboard::KeyCode::Numpad1 => return Some(ButtonCode::Numpad1),
            winit::keyboard::KeyCode::Numpad2 => return Some(ButtonCode::Numpad2),
            winit::keyboard::KeyCode::Numpad3 => return Some(ButtonCode::Numpad3),
            winit::keyboard::KeyCode::Numpad4 => return Some(ButtonCode::Numpad4),
            winit::keyboard::KeyCode::Numpad5 => return Some(ButtonCode::Numpad5),
            winit::keyboard::KeyCode::Numpad6 => return Some(ButtonCode::Numpad6),
            winit::keyboard::KeyCode::Numpad7 => return Some(ButtonCode::Numpad7),
            winit::keyboard::KeyCode::Numpad8 => return Some(ButtonCode::Numpad8),
            winit::keyboard::KeyCode::Numpad9 => return Some(ButtonCode::Numpad9),
            winit::keyboard::KeyCode::NumpadAdd => return Some(ButtonCode::NumpadAdd),
            winit::keyboard::KeyCode::NumpadBackspace => return Some(ButtonCode::NumpadBackspace),
            winit::keyboard::KeyCode::NumpadClear => return Some(ButtonCode::NumpadClear),
            winit::keyboard::KeyCode::NumpadClearEntry => return Some(ButtonCode::NumpadClearEntry),
            winit::keyboard::KeyCode::NumpadComma => return Some(ButtonCode::NumpadComma),
            winit::keyboard::KeyCode::NumpadDecimal => return Some(ButtonCode::NumpadDecimal),
            winit::keyboard::KeyCode::NumpadDivide => return Some(ButtonCode::NumpadDivide),
            winit::keyboard::KeyCode::NumpadEnter => return Some(ButtonCode::NumpadEnter),
            winit::keyboard::KeyCode::NumpadEqual => return Some(ButtonCode::NumpadEqual),
            winit::keyboard::KeyCode::NumpadHash => return Some(ButtonCode::NumpadHash),
            winit::keyboard::KeyCode::NumpadMemoryAdd => return Some(ButtonCode::NumpadMemoryAdd),
            winit::keyboard::KeyCode::NumpadMemoryClear => return Some(ButtonCode::NumpadMemoryClear),
            winit::keyboard::KeyCode::NumpadMemoryRecall => return Some(ButtonCode::NumpadMemoryRecall),
            winit::keyboard::KeyCode::NumpadMemoryStore => return Some(ButtonCode::NumpadMemoryStore),
            winit::keyboard::KeyCode::NumpadMemorySubtract => return Some(ButtonCode::NumpadMemorySubtract),
            winit::keyboard::KeyCode::NumpadMultiply => return Some(ButtonCode::NumpadMultiply),
            winit::keyboard::KeyCode::NumpadParenLeft => return Some(ButtonCode::NumpadParenLeft),
            winit::keyboard::KeyCode::NumpadParenRight => return Some(ButtonCode::NumpadParenRight),
            winit::keyboard::KeyCode::NumpadStar => return Some(ButtonCode::NumpadStar),
            winit::keyboard::KeyCode::NumpadSubtract => return Some(ButtonCode::NumpadSubtract),
            winit::keyboard::KeyCode::Escape => return Some(ButtonCode::Escape),
            winit::keyboard::KeyCode::Fn => return Some(ButtonCode::Fn),
            winit::keyboard::KeyCode::FnLock => return Some(ButtonCode::FnLock),
            winit::keyboard::KeyCode::PrintScreen => return Some(ButtonCode::PrintScreen),
            winit::keyboard::KeyCode::ScrollLock => return Some(ButtonCode::ScrollLock),
            winit::keyboard::KeyCode::Pause => return Some(ButtonCode::Pause),
            winit::keyboard::KeyCode::BrowserBack => return Some(ButtonCode::BrowserBack),
            winit::keyboard::KeyCode::BrowserFavorites => return Some(ButtonCode::BrowserFavorites),
            winit::keyboard::KeyCode::BrowserForward => return Some(ButtonCode::BrowserForward),
            winit::keyboard::KeyCode::BrowserHome => return Some(ButtonCode::BrowserHome),
            winit::keyboard::KeyCode::BrowserRefresh => return Some(ButtonCode::BrowserRefresh),
            winit::keyboard::KeyCode::BrowserSearch => return Some(ButtonCode::BrowserSearch),
            winit::keyboard::KeyCode::BrowserStop => return Some(ButtonCode::BrowserStop),
            winit::keyboard::KeyCode::Eject => return Some(ButtonCode::Eject),
            winit::keyboard::KeyCode::LaunchApp1 => return Some(ButtonCode::LaunchApp1),
            winit::keyboard::KeyCode::LaunchApp2 => return Some(ButtonCode::LaunchApp2),
            winit::keyboard::KeyCode::LaunchMail => return Some(ButtonCode::LaunchMail),
            winit::keyboard::KeyCode::MediaPlayPause => return Some(ButtonCode::MediaPlayPause),
            winit::keyboard::KeyCode::MediaSelect => return Some(ButtonCode::MediaSelect),
            winit::keyboard::KeyCode::MediaStop => return Some(ButtonCode::MediaStop),
            winit::keyboard::KeyCode::MediaTrackNext => return Some(ButtonCode::MediaTrackNext),
            winit::keyboard::KeyCode::MediaTrackPrevious => return Some(ButtonCode::MediaTrackPrevious),
            winit::keyboard::KeyCode::Power => return Some(ButtonCode::Power),
            winit::keyboard::KeyCode::Sleep => return Some(ButtonCode::Sleep),
            winit::keyboard::KeyCode::AudioVolumeDown => return Some(ButtonCode::AudioVolumeDown),
            winit::keyboard::KeyCode::AudioVolumeMute => return Some(ButtonCode::AudioVolumeMute),
            winit::keyboard::KeyCode::AudioVolumeUp => return Some(ButtonCode::AudioVolumeUp),
            winit::keyboard::KeyCode::WakeUp => return Some(ButtonCode::WakeUp),
            winit::keyboard::KeyCode::Meta => return Some(ButtonCode::Meta),
            winit::keyboard::KeyCode::Hyper => return Some(ButtonCode::Hyper),
            winit::keyboard::KeyCode::Turbo => return Some(ButtonCode::Turbo),
            winit::keyboard::KeyCode::Abort => return Some(ButtonCode::Abort),
            winit::keyboard::KeyCode::Resume => return Some(ButtonCode::Resume),
            winit::keyboard::KeyCode::Suspend => return Some(ButtonCode::Suspend),
            winit::keyboard::KeyCode::Again => return Some(ButtonCode::Again),
            winit::keyboard::KeyCode::Copy => return Some(ButtonCode::Copy),
            winit::keyboard::KeyCode::Cut => return Some(ButtonCode::Cut),
            winit::keyboard::KeyCode::Find => return Some(ButtonCode::Find),
            winit::keyboard::KeyCode::Open => return Some(ButtonCode::Open),
            winit::keyboard::KeyCode::Paste => return Some(ButtonCode::Paste),
            winit::keyboard::KeyCode::Props => return Some(ButtonCode::Props),
            winit::keyboard::KeyCode::Select => return Some(ButtonCode::Select),
            winit::keyboard::KeyCode::Undo => return Some(ButtonCode::Undo),
            winit::keyboard::KeyCode::Hiragana => return Some(ButtonCode::Hiragana),
            winit::keyboard::KeyCode::Katakana => return Some(ButtonCode::Katakana),
            winit::keyboard::KeyCode::F1 => return Some(ButtonCode::F1),
            winit::keyboard::KeyCode::F2 => return Some(ButtonCode::F2),
            winit::keyboard::KeyCode::F3 => return Some(ButtonCode::F3),
            winit::keyboard::KeyCode::F4 => return Some(ButtonCode::F4),
            winit::keyboard::KeyCode::F5 => return Some(ButtonCode::F5),
            winit::keyboard::KeyCode::F6 => return Some(ButtonCode::F6),
            winit::keyboard::KeyCode::F7 => return Some(ButtonCode::F7),
            winit::keyboard::KeyCode::F8 => return Some(ButtonCode::F8),
            winit::keyboard::KeyCode::F9 => return Some(ButtonCode::F9),
            winit::keyboard::KeyCode::F10 => return Some(ButtonCode::F10),
            winit::keyboard::KeyCode::F11 => return Some(ButtonCode::F11),
            winit::keyboard::KeyCode::F12 => return Some(ButtonCode::F12),
            _ => None,
        }
    }
    fn convert_winit_mousecode(winit_key: winit::event::MouseButton) -> Option<ButtonCode> {
        match winit_key {
            winit::event::MouseButton::Left => return Some(ButtonCode::MousePrimary),
            winit::event::MouseButton::Right => return Some(ButtonCode::MouseSecondary),
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
                // self.gamestate
                //     .set_value2::<StateScreen>(StateScreen::default());

                //
                for c in self.components.iter_mut() {
                    // run tick on all
                    c.tick(&mut self.gamestate, &mut self.system_event_queue);

                    // if debuging run debug
                    // if self.gamestate.get_value2::<StateDebug>().is_inspecting {
                    //     c.debug(&mut self.gamestate, &mut self.system_event_queue);
                    // }

                    // invoke all events
                    // let queue = self
                    //     .system_event_queue
                    //     .get_queued_events::<EngineCommands>();
                    // for event in queue {
                    //     match event {
                    //         EngineCommands::Redraw => {}
                    //         EngineCommands::Resize(vector3) => SystemGPU::set_resolution(vector3.x as i32, vector3.y as i32),
                    //         EngineCommands::Fullscreen(is_fullscreen) => SystemGPU::set_fullscreen(*is_fullscreen),
                    //         EngineCommands::Resizable(resizable) => SystemGPU::set_resizable(*resizable),
                    //         EngineCommands::Cursor(visible) => SystemGPU::set_cursor_visible(*visible),
                    //         EngineCommands::Exit => event_loop.exit(),
                    //         EngineCommands::SetDebugMode(active) => (), //self
                    //         // .gamestate
                    //         // .edit::<StateDebug>(|x| x.is_inspecting = *active),
                    //         EngineCommands::SetPauseMode(active) => (), //self.gamestate.edit::<StateDebug>(|x| x.is_paused = *active),
                    //         EngineCommands::Tick => println!("Cannot call tick from inside tick!"),
                    //         EngineCommands::SetNumInputs(_) => todo!(),
                    //         EngineCommands::SetNumScreens(_) => todo!(),
                    //         EngineCommands::SetHost() => todo!(),
                    //         EngineCommands::SetPeer() => todo!(),
                    //     }
                    // }
                    // let _ = &self
                    //     .system_event_queue
                    //     .clear_queued_events::<EngineCommands>();
                }
            }
            EngineCommands::Redraw => {}
            EngineCommands::Exit => todo!(),
            EngineCommands::Resize(_) => todo!(),
            EngineCommands::Fullscreen(_) => todo!(),
            EngineCommands::Resizable(_) => todo!(),
            EngineCommands::Cursor(_) => todo!(),
            // EngineCommands::SetDebugMode(active) => self
            //     .gamestate
            //     .edit::<StateDebug>(|x| x.is_inspecting = active),
            // EngineCommands::SetPauseMode(active) => self.gamestate.edit::<StateDebug>(|x| x.is_paused = active),
            EngineCommands::SetNumInputs(_num) => todo!(),
            EngineCommands::SetNumScreens(_num) => todo!(),
            EngineCommands::SetHost() => todo!(),
            EngineCommands::SetPeer() => todo!(),
            _ => (),
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
                    let s = c.refresh(&mut self.gamestate, &mut self.system_event_queue);
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

            WindowEvent::KeyboardInput { device_id: _, is_synthetic: _, event } => {
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
                // if toggle_debug {
                //     self.user_event(
                //         event_loop,
                //         EngineCommands::SetDebugMode(!self.gamestate.get_value2::<StateDebug>().is_inspecting),
                //     );
                // }
            }
            WindowEvent::CursorMoved { device_id: _, position } => {
                for c in self.components.iter_mut() {
                    c.input_axis(&mut self.gamestate, crate::input::axis_code::AxisCode::Cursor, crate::collections::vector3::Vector3::new(position.x as f32, position.y as f32, 0.0));
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
