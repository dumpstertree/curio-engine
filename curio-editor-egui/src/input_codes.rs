//! Translates egui's key enum into curio's own [`ButtonCode`] space.
//!
//! The game runner receives fully-typed [`ButtonCode`]s in its input events,
//! so this is the single place that knows about egui's key naming. Anything
//! egui reports that has no curio equivalent (shifted symbols like `Plus`,
//! function keys past `F12`, modifiers, which egui delivers separately via
//! `Modifiers`) maps to `None` and is simply not forwarded.

use curio_core::ButtonCode;

pub fn egui_key_to_button_code(key: egui::Key) -> Option<ButtonCode> {
    let code = match key {
        // letters
        egui::Key::A => ButtonCode::KeyA,
        egui::Key::B => ButtonCode::KeyB,
        egui::Key::C => ButtonCode::KeyC,
        egui::Key::D => ButtonCode::KeyD,
        egui::Key::E => ButtonCode::KeyE,
        egui::Key::F => ButtonCode::KeyF,
        egui::Key::G => ButtonCode::KeyG,
        egui::Key::H => ButtonCode::KeyH,
        egui::Key::I => ButtonCode::KeyI,
        egui::Key::J => ButtonCode::KeyJ,
        egui::Key::K => ButtonCode::KeyK,
        egui::Key::L => ButtonCode::KeyL,
        egui::Key::M => ButtonCode::KeyM,
        egui::Key::N => ButtonCode::KeyN,
        egui::Key::O => ButtonCode::KeyO,
        egui::Key::P => ButtonCode::KeyP,
        egui::Key::Q => ButtonCode::KeyQ,
        egui::Key::R => ButtonCode::KeyR,
        egui::Key::S => ButtonCode::KeyS,
        egui::Key::T => ButtonCode::KeyT,
        egui::Key::U => ButtonCode::KeyU,
        egui::Key::V => ButtonCode::KeyV,
        egui::Key::W => ButtonCode::KeyW,
        egui::Key::X => ButtonCode::KeyX,
        egui::Key::Y => ButtonCode::KeyY,
        egui::Key::Z => ButtonCode::KeyZ,

        // numbers (egui folds the numpad into these; they reach us as digits)
        egui::Key::Num0 => ButtonCode::Digit0,
        egui::Key::Num1 => ButtonCode::Digit1,
        egui::Key::Num2 => ButtonCode::Digit2,
        egui::Key::Num3 => ButtonCode::Digit3,
        egui::Key::Num4 => ButtonCode::Digit4,
        egui::Key::Num5 => ButtonCode::Digit5,
        egui::Key::Num6 => ButtonCode::Digit6,
        egui::Key::Num7 => ButtonCode::Digit7,
        egui::Key::Num8 => ButtonCode::Digit8,
        egui::Key::Num9 => ButtonCode::Digit9,

        // punctuation
        egui::Key::Backtick => ButtonCode::Backquote,
        egui::Key::Minus => ButtonCode::Minus,
        egui::Key::Equals => ButtonCode::Equal,
        egui::Key::OpenBracket => ButtonCode::BracketLeft,
        egui::Key::CloseBracket => ButtonCode::BracketRight,
        egui::Key::Backslash => ButtonCode::Backslash,
        egui::Key::Semicolon => ButtonCode::Semicolon,
        egui::Key::Quote => ButtonCode::Quote,
        egui::Key::Comma => ButtonCode::Comma,
        egui::Key::Period => ButtonCode::Period,
        egui::Key::Slash => ButtonCode::Slash,

        // whitespace / editing
        egui::Key::Space => ButtonCode::Space,
        egui::Key::Enter => ButtonCode::Enter,
        egui::Key::Tab => ButtonCode::Tab,
        egui::Key::Backspace => ButtonCode::Backspace,
        egui::Key::Delete => ButtonCode::Delete,
        egui::Key::Insert => ButtonCode::Insert,
        egui::Key::Escape => ButtonCode::Escape,

        // navigation
        egui::Key::ArrowDown => ButtonCode::ArrowDown,
        egui::Key::ArrowLeft => ButtonCode::ArrowLeft,
        egui::Key::ArrowRight => ButtonCode::ArrowRight,
        egui::Key::ArrowUp => ButtonCode::ArrowUp,
        egui::Key::Home => ButtonCode::Home,
        egui::Key::End => ButtonCode::End,
        egui::Key::PageUp => ButtonCode::PageUp,
        egui::Key::PageDown => ButtonCode::PageDown,

        // consumer keys
        egui::Key::Copy => ButtonCode::Copy,
        egui::Key::Cut => ButtonCode::Cut,
        egui::Key::Paste => ButtonCode::Paste,

        // function keys
        egui::Key::F1 => ButtonCode::F1,
        egui::Key::F2 => ButtonCode::F2,
        egui::Key::F3 => ButtonCode::F3,
        egui::Key::F4 => ButtonCode::F4,
        egui::Key::F5 => ButtonCode::F5,
        egui::Key::F6 => ButtonCode::F6,
        egui::Key::F7 => ButtonCode::F7,
        egui::Key::F8 => ButtonCode::F8,
        egui::Key::F9 => ButtonCode::F9,
        egui::Key::F10 => ButtonCode::F10,
        egui::Key::F11 => ButtonCode::F11,
        egui::Key::F12 => ButtonCode::F12,

        // everything else (shifted symbols, F13+, modifiers) has no ButtonCode
        _ => return None,
    };
    Some(code)
}
