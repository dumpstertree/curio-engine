use crate::collections::event_queue::EventQueue;
use crate::collections::game_state::GameState;
use crate::collections::key_state::KeyState;
use crate::dumpster_engine::GameMode;
use crate::events::engine_commands::EngineCommands;
use crate::input::key_code::KeyCode;
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
            winit::keyboard::KeyCode::Backslash => return Some(KeyCode::Backslash),
            winit::keyboard::KeyCode::BracketLeft => return Some(KeyCode::BracketLeft),
            winit::keyboard::KeyCode::BracketRight => return Some(KeyCode::BracketRight),
            winit::keyboard::KeyCode::Comma => return Some(KeyCode::Comma),
            winit::keyboard::KeyCode::Digit0 => return Some(KeyCode::Digit0),
            winit::keyboard::KeyCode::Digit1 => return Some(KeyCode::Digit1),
            winit::keyboard::KeyCode::Digit2 => return Some(KeyCode::Digit2),
            winit::keyboard::KeyCode::Digit3 => return Some(KeyCode::Digit3),
            winit::keyboard::KeyCode::Digit4 => return Some(KeyCode::Digit4),
            winit::keyboard::KeyCode::Digit5 => return Some(KeyCode::Digit5),
            winit::keyboard::KeyCode::Digit6 => return Some(KeyCode::Digit6),
            winit::keyboard::KeyCode::Digit7 => return Some(KeyCode::Digit7),
            winit::keyboard::KeyCode::Digit8 => return Some(KeyCode::Digit8),
            winit::keyboard::KeyCode::Digit9 => return Some(KeyCode::Digit9),
            winit::keyboard::KeyCode::Equal => return Some(KeyCode::Equal),
            winit::keyboard::KeyCode::IntlBackslash => return Some(KeyCode::IntlBackslash),
            winit::keyboard::KeyCode::IntlRo => return Some(KeyCode::IntlRo),
            winit::keyboard::KeyCode::IntlYen => return Some(KeyCode::IntlYen),
            winit::keyboard::KeyCode::KeyB => return Some(KeyCode::KeyB),
            winit::keyboard::KeyCode::KeyC => return Some(KeyCode::KeyC),
            winit::keyboard::KeyCode::KeyE => return Some(KeyCode::KeyE),
            winit::keyboard::KeyCode::KeyF => return Some(KeyCode::KeyF),
            winit::keyboard::KeyCode::KeyG => return Some(KeyCode::KeyG),
            winit::keyboard::KeyCode::KeyH => return Some(KeyCode::KeyH),
            winit::keyboard::KeyCode::KeyM => return Some(KeyCode::KeyM),
            winit::keyboard::KeyCode::KeyN => return Some(KeyCode::KeyN),
            winit::keyboard::KeyCode::KeyO => return Some(KeyCode::KeyO),
            winit::keyboard::KeyCode::KeyP => return Some(KeyCode::KeyP),
            winit::keyboard::KeyCode::KeyQ => return Some(KeyCode::KeyQ),
            winit::keyboard::KeyCode::KeyR => return Some(KeyCode::KeyR),
            winit::keyboard::KeyCode::KeyT => return Some(KeyCode::KeyT),
            winit::keyboard::KeyCode::KeyU => return Some(KeyCode::KeyU),
            winit::keyboard::KeyCode::KeyV => return Some(KeyCode::KeyV),
            winit::keyboard::KeyCode::KeyX => return Some(KeyCode::KeyX),
            winit::keyboard::KeyCode::KeyY => return Some(KeyCode::KeyY),
            winit::keyboard::KeyCode::KeyZ => return Some(KeyCode::KeyZ),
            winit::keyboard::KeyCode::Minus => return Some(KeyCode::Minus),
            winit::keyboard::KeyCode::Period => return Some(KeyCode::Period),
            winit::keyboard::KeyCode::Quote => return Some(KeyCode::Quote),
            winit::keyboard::KeyCode::Semicolon => return Some(KeyCode::Semicolon),
            winit::keyboard::KeyCode::Slash => return Some(KeyCode::Slash),
            winit::keyboard::KeyCode::AltLeft => return Some(KeyCode::AltLeft),
            winit::keyboard::KeyCode::AltRight => return Some(KeyCode::AltRight),
            winit::keyboard::KeyCode::Backspace => return Some(KeyCode::Backspace),
            winit::keyboard::KeyCode::CapsLock => return Some(KeyCode::CapsLock),
            winit::keyboard::KeyCode::ContextMenu => return Some(KeyCode::ContextMenu),
            winit::keyboard::KeyCode::ControlLeft => return Some(KeyCode::ControlLeft),
            winit::keyboard::KeyCode::ControlRight => return Some(KeyCode::ControlRight),
            winit::keyboard::KeyCode::Enter => return Some(KeyCode::Enter),
            winit::keyboard::KeyCode::SuperLeft => return Some(KeyCode::SuperLeft),
            winit::keyboard::KeyCode::SuperRight => return Some(KeyCode::SuperRight),
            winit::keyboard::KeyCode::ShiftLeft => return Some(KeyCode::ShiftLeft),
            winit::keyboard::KeyCode::ShiftRight => return Some(KeyCode::ShiftRight),
            winit::keyboard::KeyCode::Space => return Some(KeyCode::Space),
            winit::keyboard::KeyCode::Tab => return Some(KeyCode::Tab),
            winit::keyboard::KeyCode::Convert => return Some(KeyCode::Convert),
            winit::keyboard::KeyCode::KanaMode => return Some(KeyCode::KanaMode),
            winit::keyboard::KeyCode::Lang1 => return Some(KeyCode::Lang1),
            winit::keyboard::KeyCode::Lang2 => return Some(KeyCode::Lang2),
            winit::keyboard::KeyCode::Lang3 => return Some(KeyCode::Lang3),
            winit::keyboard::KeyCode::Lang4 => return Some(KeyCode::Lang4),
            winit::keyboard::KeyCode::Lang5 => return Some(KeyCode::Lang5),
            winit::keyboard::KeyCode::NonConvert => return Some(KeyCode::NonConvert),
            winit::keyboard::KeyCode::Delete => return Some(KeyCode::Delete),
            winit::keyboard::KeyCode::End => return Some(KeyCode::End),
            winit::keyboard::KeyCode::Help => return Some(KeyCode::Help),
            winit::keyboard::KeyCode::Home => return Some(KeyCode::Home),
            winit::keyboard::KeyCode::Insert => return Some(KeyCode::Insert),
            winit::keyboard::KeyCode::PageDown => return Some(KeyCode::PageDown),
            winit::keyboard::KeyCode::PageUp => return Some(KeyCode::PageUp),
            winit::keyboard::KeyCode::ArrowDown => return Some(KeyCode::ArrowDown),
            winit::keyboard::KeyCode::ArrowLeft => return Some(KeyCode::ArrowLeft),
            winit::keyboard::KeyCode::ArrowRight => return Some(KeyCode::ArrowRight),
            winit::keyboard::KeyCode::ArrowUp => return Some(KeyCode::ArrowUp),
            winit::keyboard::KeyCode::NumLock => return Some(KeyCode::NumLock),
            winit::keyboard::KeyCode::Numpad0 => return Some(KeyCode::Numpad0),
            winit::keyboard::KeyCode::Numpad1 => return Some(KeyCode::Numpad1),
            winit::keyboard::KeyCode::Numpad2 => return Some(KeyCode::Numpad2),
            winit::keyboard::KeyCode::Numpad3 => return Some(KeyCode::Numpad3),
            winit::keyboard::KeyCode::Numpad4 => return Some(KeyCode::Numpad4),
            winit::keyboard::KeyCode::Numpad5 => return Some(KeyCode::Numpad5),
            winit::keyboard::KeyCode::Numpad6 => return Some(KeyCode::Numpad6),
            winit::keyboard::KeyCode::Numpad7 => return Some(KeyCode::Numpad7),
            winit::keyboard::KeyCode::Numpad8 => return Some(KeyCode::Numpad8),
            winit::keyboard::KeyCode::Numpad9 => return Some(KeyCode::Numpad9),
            winit::keyboard::KeyCode::NumpadAdd => return Some(KeyCode::NumpadAdd),
            winit::keyboard::KeyCode::NumpadBackspace => return Some(KeyCode::NumpadBackspace),
            winit::keyboard::KeyCode::NumpadClear => return Some(KeyCode::NumpadClear),
            winit::keyboard::KeyCode::NumpadClearEntry => return Some(KeyCode::NumpadClearEntry),
            winit::keyboard::KeyCode::NumpadComma => return Some(KeyCode::NumpadComma),
            winit::keyboard::KeyCode::NumpadDecimal => return Some(KeyCode::NumpadDecimal),
            winit::keyboard::KeyCode::NumpadDivide => return Some(KeyCode::NumpadDivide),
            winit::keyboard::KeyCode::NumpadEnter => return Some(KeyCode::NumpadEnter),
            winit::keyboard::KeyCode::NumpadEqual => return Some(KeyCode::NumpadEqual),
            winit::keyboard::KeyCode::NumpadHash => return Some(KeyCode::NumpadHash),
            winit::keyboard::KeyCode::NumpadMemoryAdd => return Some(KeyCode::NumpadMemoryAdd),
            winit::keyboard::KeyCode::NumpadMemoryClear => return Some(KeyCode::NumpadMemoryClear),
            winit::keyboard::KeyCode::NumpadMemoryRecall => return Some(KeyCode::NumpadMemoryRecall),
            winit::keyboard::KeyCode::NumpadMemoryStore => return Some(KeyCode::NumpadMemoryStore),
            winit::keyboard::KeyCode::NumpadMemorySubtract => return Some(KeyCode::NumpadMemorySubtract),
            winit::keyboard::KeyCode::NumpadMultiply => return Some(KeyCode::NumpadMultiply),
            winit::keyboard::KeyCode::NumpadParenLeft => return Some(KeyCode::NumpadParenLeft),
            winit::keyboard::KeyCode::NumpadParenRight => return Some(KeyCode::NumpadParenRight),
            winit::keyboard::KeyCode::NumpadStar => return Some(KeyCode::NumpadStar),
            winit::keyboard::KeyCode::NumpadSubtract => return Some(KeyCode::NumpadSubtract),
            winit::keyboard::KeyCode::Escape => return Some(KeyCode::Escape),
            winit::keyboard::KeyCode::Fn => return Some(KeyCode::Fn),
            winit::keyboard::KeyCode::FnLock => return Some(KeyCode::FnLock),
            winit::keyboard::KeyCode::PrintScreen => return Some(KeyCode::PrintScreen),
            winit::keyboard::KeyCode::ScrollLock => return Some(KeyCode::ScrollLock),
            winit::keyboard::KeyCode::Pause => return Some(KeyCode::Pause),
            winit::keyboard::KeyCode::BrowserBack => return Some(KeyCode::BrowserBack),
            winit::keyboard::KeyCode::BrowserFavorites => return Some(KeyCode::BrowserFavorites),
            winit::keyboard::KeyCode::BrowserForward => return Some(KeyCode::BrowserForward),
            winit::keyboard::KeyCode::BrowserHome => return Some(KeyCode::BrowserHome),
            winit::keyboard::KeyCode::BrowserRefresh => return Some(KeyCode::BrowserRefresh),
            winit::keyboard::KeyCode::BrowserSearch => return Some(KeyCode::BrowserSearch),
            winit::keyboard::KeyCode::BrowserStop => return Some(KeyCode::BrowserStop),
            winit::keyboard::KeyCode::Eject => return Some(KeyCode::Eject),
            winit::keyboard::KeyCode::LaunchApp1 => return Some(KeyCode::LaunchApp1),
            winit::keyboard::KeyCode::LaunchApp2 => return Some(KeyCode::LaunchApp2),
            winit::keyboard::KeyCode::LaunchMail => return Some(KeyCode::LaunchMail),
            winit::keyboard::KeyCode::MediaPlayPause => return Some(KeyCode::MediaPlayPause),
            winit::keyboard::KeyCode::MediaSelect => return Some(KeyCode::MediaSelect),
            winit::keyboard::KeyCode::MediaStop => return Some(KeyCode::MediaStop),
            winit::keyboard::KeyCode::MediaTrackNext => return Some(KeyCode::MediaTrackNext),
            winit::keyboard::KeyCode::MediaTrackPrevious => return Some(KeyCode::MediaTrackPrevious),
            winit::keyboard::KeyCode::Power => return Some(KeyCode::Power),
            winit::keyboard::KeyCode::Sleep => return Some(KeyCode::Sleep),
            winit::keyboard::KeyCode::AudioVolumeDown => return Some(KeyCode::AudioVolumeDown),
            winit::keyboard::KeyCode::AudioVolumeMute => return Some(KeyCode::AudioVolumeMute),
            winit::keyboard::KeyCode::AudioVolumeUp => return Some(KeyCode::AudioVolumeUp),
            winit::keyboard::KeyCode::WakeUp => return Some(KeyCode::WakeUp),
            winit::keyboard::KeyCode::Meta => return Some(KeyCode::Meta),
            winit::keyboard::KeyCode::Hyper => return Some(KeyCode::Hyper),
            winit::keyboard::KeyCode::Turbo => return Some(KeyCode::Turbo),
            winit::keyboard::KeyCode::Abort => return Some(KeyCode::Abort),
            winit::keyboard::KeyCode::Resume => return Some(KeyCode::Resume),
            winit::keyboard::KeyCode::Suspend => return Some(KeyCode::Suspend),
            winit::keyboard::KeyCode::Again => return Some(KeyCode::Again),
            winit::keyboard::KeyCode::Copy => return Some(KeyCode::Copy),
            winit::keyboard::KeyCode::Cut => return Some(KeyCode::Cut),
            winit::keyboard::KeyCode::Find => return Some(KeyCode::Find),
            winit::keyboard::KeyCode::Open => return Some(KeyCode::Open),
            winit::keyboard::KeyCode::Paste => return Some(KeyCode::Paste),
            winit::keyboard::KeyCode::Props => return Some(KeyCode::Props),
            winit::keyboard::KeyCode::Select => return Some(KeyCode::Select),
            winit::keyboard::KeyCode::Undo => return Some(KeyCode::Undo),
            winit::keyboard::KeyCode::Hiragana => return Some(KeyCode::Hiragana),
            winit::keyboard::KeyCode::Katakana => return Some(KeyCode::Katakana),
            winit::keyboard::KeyCode::F1 => return Some(KeyCode::F1),
            winit::keyboard::KeyCode::F2 => return Some(KeyCode::F2),
            winit::keyboard::KeyCode::F3 => return Some(KeyCode::F3),
            winit::keyboard::KeyCode::F4 => return Some(KeyCode::F4),
            winit::keyboard::KeyCode::F5 => return Some(KeyCode::F5),
            winit::keyboard::KeyCode::F6 => return Some(KeyCode::F6),
            winit::keyboard::KeyCode::F7 => return Some(KeyCode::F7),
            winit::keyboard::KeyCode::F8 => return Some(KeyCode::F8),
            winit::keyboard::KeyCode::F9 => return Some(KeyCode::F9),
            winit::keyboard::KeyCode::F10 => return Some(KeyCode::F10),
            winit::keyboard::KeyCode::F11 => return Some(KeyCode::F11),
            winit::keyboard::KeyCode::F12 => return Some(KeyCode::F12),
            _ => None,
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
