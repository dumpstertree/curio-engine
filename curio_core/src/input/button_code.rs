use serde::{Deserialize, Serialize};
use winit::{
    event::MouseButton,
    keyboard::{KeyCode, PhysicalKey},
};

///Definitions for all types of binary based inputs
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ButtonCode {
    MousePrimary,
    MouseSecondary,
    /// <kbd>`</kbd> on a US keyboard. This is also called a backtick or grave.
    /// This is the <kbd>半角</kbd>/<kbd>全角</kbd>/<kbd>漢字</kbd>
    /// (hankaku/zenkaku/kanji) key on Japanese keyboards
    Backquote,
    /// Used for both the US <kbd>\\</kbd> (on the 101-key layout) and also for the key
    /// located between the <kbd>"</kbd> and <kbd>Enter</kbd> keys on row C of the 102-,
    /// 104- and 106-key layouts.
    /// Labeled <kbd>#</kbd> on a UK (102) keyboard.
    Backslash,
    /// <kbd>[</kbd> on a US keyboard.
    BracketLeft,
    /// <kbd>]</kbd> on a US keyboard.
    BracketRight,
    /// <kbd>,</kbd> on a US keyboard.
    Comma,
    /// <kbd>0</kbd> on a US keyboard.
    Digit0,
    /// <kbd>1</kbd> on a US keyboard.
    Digit1,
    /// <kbd>2</kbd> on a US keyboard.
    Digit2,
    /// <kbd>3</kbd> on a US keyboard.
    Digit3,
    /// <kbd>4</kbd> on a US keyboard.
    Digit4,
    /// <kbd>5</kbd> on a US keyboard.
    Digit5,
    /// <kbd>6</kbd> on a US keyboard.
    Digit6,
    /// <kbd>7</kbd> on a US keyboard.
    Digit7,
    /// <kbd>8</kbd> on a US keyboard.
    Digit8,
    /// <kbd>9</kbd> on a US keyboard.
    Digit9,
    /// <kbd>=</kbd> on a US keyboard.
    Equal,
    /// Located between the left <kbd>Shift</kbd> and <kbd>Z</kbd> keys.
    /// Labeled <kbd>\\</kbd> on a UK keyboard.
    IntlBackslash,
    /// Located between the <kbd>/</kbd> and right <kbd>Shift</kbd> keys.
    /// Labeled <kbd>\\</kbd> (ro) on a Japanese keyboard.
    IntlRo,
    /// Located between the <kbd>=</kbd> and <kbd>Backspace</kbd> keys.
    /// Labeled <kbd>¥</kbd> (yen) on a Japanese keyboard. <kbd>\\</kbd> on a
    /// Russian keyboard.
    IntlYen,
    /// <kbd>a</kbd> on a US keyboard.
    /// Labeled <kbd>q</kbd> on an AZERTY (e.g., French) keyboard.
    KeyA,
    /// <kbd>b</kbd> on a US keyboard.
    KeyB,
    /// <kbd>c</kbd> on a US keyboard.
    KeyC,
    /// <kbd>d</kbd> on a US keyboard.
    KeyD,
    /// <kbd>e</kbd> on a US keyboard.
    KeyE,
    /// <kbd>f</kbd> on a US keyboard.
    KeyF,
    /// <kbd>g</kbd> on a US keyboard.
    KeyG,
    /// <kbd>h</kbd> on a US keyboard.
    KeyH,
    /// <kbd>i</kbd> on a US keyboard.
    KeyI,
    /// <kbd>j</kbd> on a US keyboard.
    KeyJ,
    /// <kbd>k</kbd> on a US keyboard.
    KeyK,
    /// <kbd>l</kbd> on a US keyboard.
    KeyL,
    /// <kbd>m</kbd> on a US keyboard.
    KeyM,
    /// <kbd>n</kbd> on a US keyboard.
    KeyN,
    /// <kbd>o</kbd> on a US keyboard.
    KeyO,
    /// <kbd>p</kbd> on a US keyboard.
    KeyP,
    /// <kbd>q</kbd> on a US keyboard.
    /// Labeled <kbd>a</kbd> on an AZERTY (e.g., French) keyboard.
    KeyQ,
    /// <kbd>r</kbd> on a US keyboard.
    KeyR,
    /// <kbd>s</kbd> on a US keyboard.
    KeyS,
    /// <kbd>t</kbd> on a US keyboard.
    KeyT,
    /// <kbd>u</kbd> on a US keyboard.
    KeyU,
    /// <kbd>v</kbd> on a US keyboard.
    KeyV,
    /// <kbd>w</kbd> on a US keyboard.
    /// Labeled <kbd>z</kbd> on an AZERTY (e.g., French) keyboard.
    KeyW,
    /// <kbd>x</kbd> on a US keyboard.
    KeyX,
    /// <kbd>y</kbd> on a US keyboard.
    /// Labeled <kbd>z</kbd> on a QWERTZ (e.g., German) keyboard.
    KeyY,
    /// <kbd>z</kbd> on a US keyboard.
    /// Labeled <kbd>w</kbd> on an AZERTY (e.g., French) keyboard, and <kbd>y</kbd> on a
    /// QWERTZ (e.g., German) keyboard.
    KeyZ,
    /// <kbd>-</kbd> on a US keyboard.
    Minus,
    /// <kbd>.</kbd> on a US keyboard.
    Period,
    /// <kbd>'</kbd> on a US keyboard.
    Quote,
    /// <kbd>;</kbd> on a US keyboard.
    Semicolon,
    /// <kbd>/</kbd> on a US keyboard.
    Slash,
    /// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
    AltLeft,
    /// <kbd>Alt</kbd>, <kbd>Option</kbd>, or <kbd>⌥</kbd>.
    /// This is labeled <kbd>AltGr</kbd> on many keyboard layouts.
    AltRight,
    /// <kbd>Backspace</kbd> or <kbd>⌫</kbd>.
    /// Labeled <kbd>Delete</kbd> on Apple keyboards.
    Backspace,
    /// <kbd>CapsLock</kbd> or <kbd>⇪</kbd>
    CapsLock,
    /// The application context menu key, which is typically found between the right
    /// <kbd>Super</kbd> key and the right <kbd>Control</kbd> key.
    ContextMenu,
    /// <kbd>Control</kbd> or <kbd>⌃</kbd>
    ControlLeft,
    /// <kbd>Control</kbd> or <kbd>⌃</kbd>
    ControlRight,
    /// <kbd>Enter</kbd> or <kbd>↵</kbd>. Labeled <kbd>Return</kbd> on Apple keyboards.
    Enter,
    /// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
    SuperLeft,
    /// The Windows, <kbd>⌘</kbd>, <kbd>Command</kbd>, or other OS symbol key.
    SuperRight,
    /// <kbd>Shift</kbd> or <kbd>⇧</kbd>
    ShiftLeft,
    /// <kbd>Shift</kbd> or <kbd>⇧</kbd>
    ShiftRight,
    /// <kbd> </kbd> (space)
    Space,
    /// <kbd>Tab</kbd> or <kbd>⇥</kbd>
    Tab,
    /// Japanese: <kbd>変</kbd> (henkan)
    Convert,
    /// Japanese: <kbd>カタカナ</kbd>/<kbd>ひらがな</kbd>/<kbd>ローマ字</kbd>
    /// (katakana/hiragana/romaji)
    KanaMode,
    /// Korean: HangulMode <kbd>한/영</kbd> (han/yeong)
    ///
    /// Japanese (Mac keyboard): <kbd>か</kbd> (kana)
    Lang1,
    /// Korean: Hanja <kbd>한</kbd> (hanja)
    ///
    /// Japanese (Mac keyboard): <kbd>英</kbd> (eisu)
    Lang2,
    /// Japanese (word-processing keyboard): Katakana
    Lang3,
    /// Japanese (word-processing keyboard): Hiragana
    Lang4,
    /// Japanese (word-processing keyboard): Zenkaku/Hankaku
    Lang5,
    /// Japanese: <kbd>無変換</kbd> (muhenkan)
    NonConvert,
    /// <kbd>⌦</kbd>. The forward delete key.
    /// Note that on Apple keyboards, the key labelled <kbd>Delete</kbd> on the main part of
    /// the keyboard is encoded as [`Backspace`].
    ///
    /// [`Backspace`]: Self::Backspace
    Delete,
    /// <kbd>Page Down</kbd>, <kbd>End</kbd>, or <kbd>↘</kbd>
    End,
    /// <kbd>Help</kbd>. Not present on standard PC keyboards.
    Help,
    /// <kbd>Home</kbd> or <kbd>↖</kbd>
    Home,
    /// <kbd>Insert</kbd> or <kbd>Ins</kbd>. Not present on Apple keyboards.
    Insert,
    /// <kbd>Page Down</kbd>, <kbd>PgDn</kbd>, or <kbd>⇟</kbd>
    PageDown,
    /// <kbd>Page Up</kbd>, <kbd>PgUp</kbd>, or <kbd>⇞</kbd>
    PageUp,
    /// <kbd>↓</kbd>
    ArrowDown,
    /// <kbd>←</kbd>
    ArrowLeft,
    /// <kbd>→</kbd>
    ArrowRight,
    /// <kbd>↑</kbd>
    ArrowUp,
    /// On the Mac, this is used for the numpad <kbd>Clear</kbd> key.
    NumLock,
    /// <kbd>0 Ins</kbd> on a keyboard. <kbd>0</kbd> on a phone or remote control
    Numpad0,
    /// <kbd>1 End</kbd> on a keyboard. <kbd>1</kbd> or <kbd>1 QZ</kbd> on a phone or remote
    /// control
    Numpad1,
    /// <kbd>2 ↓</kbd> on a keyboard. <kbd>2 ABC</kbd> on a phone or remote control
    Numpad2,
    /// <kbd>3 PgDn</kbd> on a keyboard. <kbd>3 DEF</kbd> on a phone or remote control
    Numpad3,
    /// <kbd>4 ←</kbd> on a keyboard. <kbd>4 GHI</kbd> on a phone or remote control
    Numpad4,
    /// <kbd>5</kbd> on a keyboard. <kbd>5 JKL</kbd> on a phone or remote control
    Numpad5,
    /// <kbd>6 →</kbd> on a keyboard. <kbd>6 MNO</kbd> on a phone or remote control
    Numpad6,
    /// <kbd>7 Home</kbd> on a keyboard. <kbd>7 PQRS</kbd> or <kbd>7 PRS</kbd> on a phone
    /// or remote control
    Numpad7,
    /// <kbd>8 ↑</kbd> on a keyboard. <kbd>8 TUV</kbd> on a phone or remote control
    Numpad8,
    /// <kbd>9 PgUp</kbd> on a keyboard. <kbd>9 WXYZ</kbd> or <kbd>9 WXY</kbd> on a phone
    /// or remote control
    Numpad9,
    /// <kbd>+</kbd>
    NumpadAdd,
    /// Found on the Microsoft Natural Keyboard.
    NumpadBackspace,
    /// <kbd>C</kbd> or <kbd>A</kbd> (All Clear). Also for use with numpads that have a
    /// <kbd>Clear</kbd> key that is separate from the <kbd>NumLock</kbd> key. On the Mac, the
    /// numpad <kbd>Clear</kbd> key is encoded as [`NumLock`].
    ///
    /// [`NumLock`]: Self::NumLock
    NumpadClear,
    /// <kbd>C</kbd> (Clear Entry)
    NumpadClearEntry,
    /// <kbd>,</kbd> (thousands separator). For locales where the thousands separator
    /// is a "." (e.g., Brazil), this key may generate a <kbd>.</kbd>.
    NumpadComma,
    /// <kbd>. Del</kbd>. For locales where the decimal separator is "," (e.g.,
    /// Brazil), this key may generate a <kbd>,</kbd>.
    NumpadDecimal,
    /// <kbd>/</kbd>
    NumpadDivide,
    NumpadEnter,
    /// <kbd>=</kbd>
    NumpadEqual,
    /// <kbd>#</kbd> on a phone or remote control device. This key is typically found
    /// below the <kbd>9</kbd> key and to the right of the <kbd>0</kbd> key.
    NumpadHash,
    /// <kbd>M</kbd> Add current entry to the value stored in memory.
    NumpadMemoryAdd,
    /// <kbd>M</kbd> Clear the value stored in memory.
    NumpadMemoryClear,
    /// <kbd>M</kbd> Replace the current entry with the value stored in memory.
    NumpadMemoryRecall,
    /// <kbd>M</kbd> Replace the value stored in memory with the current entry.
    NumpadMemoryStore,
    /// <kbd>M</kbd> Subtract current entry from the value stored in memory.
    NumpadMemorySubtract,
    /// <kbd>*</kbd> on a keyboard. For use with numpads that provide mathematical
    /// operations (<kbd>+</kbd>, <kbd>-</kbd> <kbd>*</kbd> and <kbd>/</kbd>).
    ///
    /// Use `NumpadStar` for the <kbd>*</kbd> key on phones and remote controls.
    NumpadMultiply,
    /// <kbd>(</kbd> Found on the Microsoft Natural Keyboard.
    NumpadParenLeft,
    /// <kbd>)</kbd> Found on the Microsoft Natural Keyboard.
    NumpadParenRight,
    /// <kbd>*</kbd> on a phone or remote control device.
    ///
    /// This key is typically found below the <kbd>7</kbd> key and to the left of
    /// the <kbd>0</kbd> key.
    ///
    /// Use <kbd>"NumpadMultiply"</kbd> for the <kbd>*</kbd> key on
    /// numeric keypads.
    NumpadStar,
    /// <kbd>-</kbd>
    NumpadSubtract,
    /// <kbd>Esc</kbd> or <kbd>⎋</kbd>
    Escape,
    /// <kbd>Fn</kbd> This is typically a hardware key that does not generate a separate code.
    Fn,
    /// <kbd>FLock</kbd> or <kbd>FnLock</kbd>. Function Lock key. Found on the Microsoft
    /// Natural Keyboard.
    FnLock,
    /// <kbd>PrtScr SysRq</kbd> or <kbd>Print Screen</kbd>
    PrintScreen,
    /// <kbd>Scroll Lock</kbd>
    ScrollLock,
    /// <kbd>Pause Break</kbd>
    Pause,
    /// Some laptops place this key to the left of the <kbd>↑</kbd> key.
    ///
    /// This also the "back" button (triangle) on Android.
    BrowserBack,
    BrowserFavorites,
    /// Some laptops place this key to the right of the <kbd>↑</kbd> key.
    BrowserForward,
    /// The "home" button on Android.
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    BrowserStop,
    /// <kbd>Eject</kbd> or <kbd>⏏</kbd>. This key is placed in the function section on some Apple
    /// keyboards.
    Eject,
    /// Sometimes labelled <kbd>My Computer</kbd> on the keyboard
    LaunchApp1,
    /// Sometimes labelled <kbd>Calculator</kbd> on the keyboard
    LaunchApp2,
    LaunchMail,
    MediaPlayPause,
    MediaSelect,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,
    /// This key is placed in the function section on some Apple keyboards, replacing the
    /// <kbd>Eject</kbd> key.
    Power,
    Sleep,
    AudioVolumeDown,
    AudioVolumeMute,
    AudioVolumeUp,
    WakeUp,
    // Legacy modifier key. Also called "Super" in certain places.
    Meta,
    // Legacy modifier key.
    Hyper,
    Turbo,
    Abort,
    Resume,
    Suspend,
    /// Found on Sun’s USB keyboard.
    Again,
    /// Found on Sun’s USB keyboard.
    Copy,
    /// Found on Sun’s USB keyboard.
    Cut,
    /// Found on Sun’s USB keyboard.
    Find,
    /// Found on Sun’s USB keyboard.
    Open,
    /// Found on Sun’s USB keyboard.
    Paste,
    /// Found on Sun’s USB keyboard.
    Props,
    /// Found on Sun’s USB keyboard.
    Select,
    /// Found on Sun’s USB keyboard.
    Undo,
    /// Use for dedicated <kbd>ひらがな</kbd> key found on some Japanese word processing keyboards.
    Hiragana,
    /// Use for dedicated <kbd>カタカナ</kbd> key found on some Japanese word processing keyboards.
    Katakana,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F1,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F2,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F3,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F4,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F5,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F6,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F7,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F8,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F9,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F10,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F11,
    /// General-purpose function key.
    /// Usually found at the top of the keyboard.
    F12,
}
impl ButtonCode {
    /// Converts a Winit::MouseButton to a ButtonCode
    pub fn from_winit_mousebutton(winit_mouse: MouseButton) -> Option<ButtonCode> {
        match winit_mouse {
            MouseButton::Left => return Some(ButtonCode::MousePrimary),
            MouseButton::Right => return Some(ButtonCode::MouseSecondary),
            _ => return None,
        }
    }

    pub fn from_winit_physical_key(winit_key: PhysicalKey) -> Option<ButtonCode> {
        match winit_key {
            PhysicalKey::Code(key_code) => Self::from_winit_keycode(key_code),
            _ => return None,
        }
    }

    /// Converts a Winit::MouseButton to a ButtonCode
    pub fn from_winit_keycode(winit_key: KeyCode) -> Option<ButtonCode> {
        match winit_key {
            KeyCode::Backquote => return Some(ButtonCode::Backquote),
            KeyCode::KeyW => return Some(ButtonCode::KeyW),
            KeyCode::KeyA => return Some(ButtonCode::KeyA),
            KeyCode::KeyS => return Some(ButtonCode::KeyS),
            KeyCode::KeyD => return Some(ButtonCode::KeyD),
            KeyCode::KeyI => return Some(ButtonCode::KeyI),
            KeyCode::KeyJ => return Some(ButtonCode::KeyJ),
            KeyCode::KeyK => return Some(ButtonCode::KeyK),
            KeyCode::KeyL => return Some(ButtonCode::KeyL),
            KeyCode::Backslash => return Some(ButtonCode::Backslash),
            KeyCode::BracketLeft => return Some(ButtonCode::BracketLeft),
            KeyCode::BracketRight => return Some(ButtonCode::BracketRight),
            KeyCode::Comma => return Some(ButtonCode::Comma),
            KeyCode::Digit0 => return Some(ButtonCode::Digit0),
            KeyCode::Digit1 => return Some(ButtonCode::Digit1),
            KeyCode::Digit2 => return Some(ButtonCode::Digit2),
            KeyCode::Digit3 => return Some(ButtonCode::Digit3),
            KeyCode::Digit4 => return Some(ButtonCode::Digit4),
            KeyCode::Digit5 => return Some(ButtonCode::Digit5),
            KeyCode::Digit6 => return Some(ButtonCode::Digit6),
            KeyCode::Digit7 => return Some(ButtonCode::Digit7),
            KeyCode::Digit8 => return Some(ButtonCode::Digit8),
            KeyCode::Digit9 => return Some(ButtonCode::Digit9),
            KeyCode::Equal => return Some(ButtonCode::Equal),
            KeyCode::IntlBackslash => return Some(ButtonCode::IntlBackslash),
            KeyCode::IntlRo => return Some(ButtonCode::IntlRo),
            KeyCode::IntlYen => return Some(ButtonCode::IntlYen),
            KeyCode::KeyB => return Some(ButtonCode::KeyB),
            KeyCode::KeyC => return Some(ButtonCode::KeyC),
            KeyCode::KeyE => return Some(ButtonCode::KeyE),
            KeyCode::KeyF => return Some(ButtonCode::KeyF),
            KeyCode::KeyG => return Some(ButtonCode::KeyG),
            KeyCode::KeyH => return Some(ButtonCode::KeyH),
            KeyCode::KeyM => return Some(ButtonCode::KeyM),
            KeyCode::KeyN => return Some(ButtonCode::KeyN),
            KeyCode::KeyO => return Some(ButtonCode::KeyO),
            KeyCode::KeyP => return Some(ButtonCode::KeyP),
            KeyCode::KeyQ => return Some(ButtonCode::KeyQ),
            KeyCode::KeyR => return Some(ButtonCode::KeyR),
            KeyCode::KeyT => return Some(ButtonCode::KeyT),
            KeyCode::KeyU => return Some(ButtonCode::KeyU),
            KeyCode::KeyV => return Some(ButtonCode::KeyV),
            KeyCode::KeyX => return Some(ButtonCode::KeyX),
            KeyCode::KeyY => return Some(ButtonCode::KeyY),
            KeyCode::KeyZ => return Some(ButtonCode::KeyZ),
            KeyCode::Minus => return Some(ButtonCode::Minus),
            KeyCode::Period => return Some(ButtonCode::Period),
            KeyCode::Quote => return Some(ButtonCode::Quote),
            KeyCode::Semicolon => return Some(ButtonCode::Semicolon),
            KeyCode::Slash => return Some(ButtonCode::Slash),
            KeyCode::AltLeft => return Some(ButtonCode::AltLeft),
            KeyCode::AltRight => return Some(ButtonCode::AltRight),
            KeyCode::Backspace => return Some(ButtonCode::Backspace),
            KeyCode::CapsLock => return Some(ButtonCode::CapsLock),
            KeyCode::ContextMenu => return Some(ButtonCode::ContextMenu),
            KeyCode::ControlLeft => return Some(ButtonCode::ControlLeft),
            KeyCode::ControlRight => return Some(ButtonCode::ControlRight),
            KeyCode::Enter => return Some(ButtonCode::Enter),
            KeyCode::SuperLeft => return Some(ButtonCode::SuperLeft),
            KeyCode::SuperRight => return Some(ButtonCode::SuperRight),
            KeyCode::ShiftLeft => return Some(ButtonCode::ShiftLeft),
            KeyCode::ShiftRight => return Some(ButtonCode::ShiftRight),
            KeyCode::Space => return Some(ButtonCode::Space),
            KeyCode::Tab => return Some(ButtonCode::Tab),
            KeyCode::Convert => return Some(ButtonCode::Convert),
            KeyCode::KanaMode => return Some(ButtonCode::KanaMode),
            KeyCode::Lang1 => return Some(ButtonCode::Lang1),
            KeyCode::Lang2 => return Some(ButtonCode::Lang2),
            KeyCode::Lang3 => return Some(ButtonCode::Lang3),
            KeyCode::Lang4 => return Some(ButtonCode::Lang4),
            KeyCode::Lang5 => return Some(ButtonCode::Lang5),
            KeyCode::NonConvert => return Some(ButtonCode::NonConvert),
            KeyCode::Delete => return Some(ButtonCode::Delete),
            KeyCode::End => return Some(ButtonCode::End),
            KeyCode::Help => return Some(ButtonCode::Help),
            KeyCode::Home => return Some(ButtonCode::Home),
            KeyCode::Insert => return Some(ButtonCode::Insert),
            KeyCode::PageDown => return Some(ButtonCode::PageDown),
            KeyCode::PageUp => return Some(ButtonCode::PageUp),
            KeyCode::ArrowDown => return Some(ButtonCode::ArrowDown),
            KeyCode::ArrowLeft => return Some(ButtonCode::ArrowLeft),
            KeyCode::ArrowRight => return Some(ButtonCode::ArrowRight),
            KeyCode::ArrowUp => return Some(ButtonCode::ArrowUp),
            KeyCode::NumLock => return Some(ButtonCode::NumLock),
            KeyCode::Numpad0 => return Some(ButtonCode::Numpad0),
            KeyCode::Numpad1 => return Some(ButtonCode::Numpad1),
            KeyCode::Numpad2 => return Some(ButtonCode::Numpad2),
            KeyCode::Numpad3 => return Some(ButtonCode::Numpad3),
            KeyCode::Numpad4 => return Some(ButtonCode::Numpad4),
            KeyCode::Numpad5 => return Some(ButtonCode::Numpad5),
            KeyCode::Numpad6 => return Some(ButtonCode::Numpad6),
            KeyCode::Numpad7 => return Some(ButtonCode::Numpad7),
            KeyCode::Numpad8 => return Some(ButtonCode::Numpad8),
            KeyCode::Numpad9 => return Some(ButtonCode::Numpad9),
            KeyCode::NumpadAdd => return Some(ButtonCode::NumpadAdd),
            KeyCode::NumpadBackspace => return Some(ButtonCode::NumpadBackspace),
            KeyCode::NumpadClear => return Some(ButtonCode::NumpadClear),
            KeyCode::NumpadClearEntry => return Some(ButtonCode::NumpadClearEntry),
            KeyCode::NumpadComma => return Some(ButtonCode::NumpadComma),
            KeyCode::NumpadDecimal => return Some(ButtonCode::NumpadDecimal),
            KeyCode::NumpadDivide => return Some(ButtonCode::NumpadDivide),
            KeyCode::NumpadEnter => return Some(ButtonCode::NumpadEnter),
            KeyCode::NumpadEqual => return Some(ButtonCode::NumpadEqual),
            KeyCode::NumpadHash => return Some(ButtonCode::NumpadHash),
            KeyCode::NumpadMemoryAdd => return Some(ButtonCode::NumpadMemoryAdd),
            KeyCode::NumpadMemoryClear => return Some(ButtonCode::NumpadMemoryClear),
            KeyCode::NumpadMemoryRecall => return Some(ButtonCode::NumpadMemoryRecall),
            KeyCode::NumpadMemoryStore => return Some(ButtonCode::NumpadMemoryStore),
            KeyCode::NumpadMemorySubtract => return Some(ButtonCode::NumpadMemorySubtract),
            KeyCode::NumpadMultiply => return Some(ButtonCode::NumpadMultiply),
            KeyCode::NumpadParenLeft => return Some(ButtonCode::NumpadParenLeft),
            KeyCode::NumpadParenRight => return Some(ButtonCode::NumpadParenRight),
            KeyCode::NumpadStar => return Some(ButtonCode::NumpadStar),
            KeyCode::NumpadSubtract => return Some(ButtonCode::NumpadSubtract),
            KeyCode::Escape => return Some(ButtonCode::Escape),
            KeyCode::Fn => return Some(ButtonCode::Fn),
            KeyCode::FnLock => return Some(ButtonCode::FnLock),
            KeyCode::PrintScreen => return Some(ButtonCode::PrintScreen),
            KeyCode::ScrollLock => return Some(ButtonCode::ScrollLock),
            KeyCode::Pause => return Some(ButtonCode::Pause),
            KeyCode::BrowserBack => return Some(ButtonCode::BrowserBack),
            KeyCode::BrowserFavorites => return Some(ButtonCode::BrowserFavorites),
            KeyCode::BrowserForward => return Some(ButtonCode::BrowserForward),
            KeyCode::BrowserHome => return Some(ButtonCode::BrowserHome),
            KeyCode::BrowserRefresh => return Some(ButtonCode::BrowserRefresh),
            KeyCode::BrowserSearch => return Some(ButtonCode::BrowserSearch),
            KeyCode::BrowserStop => return Some(ButtonCode::BrowserStop),
            KeyCode::Eject => return Some(ButtonCode::Eject),
            KeyCode::LaunchApp1 => return Some(ButtonCode::LaunchApp1),
            KeyCode::LaunchApp2 => return Some(ButtonCode::LaunchApp2),
            KeyCode::LaunchMail => return Some(ButtonCode::LaunchMail),
            KeyCode::MediaPlayPause => return Some(ButtonCode::MediaPlayPause),
            KeyCode::MediaSelect => return Some(ButtonCode::MediaSelect),
            KeyCode::MediaStop => return Some(ButtonCode::MediaStop),
            KeyCode::MediaTrackNext => return Some(ButtonCode::MediaTrackNext),
            KeyCode::MediaTrackPrevious => return Some(ButtonCode::MediaTrackPrevious),
            KeyCode::Power => return Some(ButtonCode::Power),
            KeyCode::Sleep => return Some(ButtonCode::Sleep),
            KeyCode::AudioVolumeDown => return Some(ButtonCode::AudioVolumeDown),
            KeyCode::AudioVolumeMute => return Some(ButtonCode::AudioVolumeMute),
            KeyCode::AudioVolumeUp => return Some(ButtonCode::AudioVolumeUp),
            KeyCode::WakeUp => return Some(ButtonCode::WakeUp),
            KeyCode::Meta => return Some(ButtonCode::Meta),
            KeyCode::Hyper => return Some(ButtonCode::Hyper),
            KeyCode::Turbo => return Some(ButtonCode::Turbo),
            KeyCode::Abort => return Some(ButtonCode::Abort),
            KeyCode::Resume => return Some(ButtonCode::Resume),
            KeyCode::Suspend => return Some(ButtonCode::Suspend),
            KeyCode::Again => return Some(ButtonCode::Again),
            KeyCode::Copy => return Some(ButtonCode::Copy),
            KeyCode::Cut => return Some(ButtonCode::Cut),
            KeyCode::Find => return Some(ButtonCode::Find),
            KeyCode::Open => return Some(ButtonCode::Open),
            KeyCode::Paste => return Some(ButtonCode::Paste),
            KeyCode::Props => return Some(ButtonCode::Props),
            KeyCode::Select => return Some(ButtonCode::Select),
            KeyCode::Undo => return Some(ButtonCode::Undo),
            KeyCode::Hiragana => return Some(ButtonCode::Hiragana),
            KeyCode::Katakana => return Some(ButtonCode::Katakana),
            KeyCode::F1 => return Some(ButtonCode::F1),
            KeyCode::F2 => return Some(ButtonCode::F2),
            KeyCode::F3 => return Some(ButtonCode::F3),
            KeyCode::F4 => return Some(ButtonCode::F4),
            KeyCode::F5 => return Some(ButtonCode::F5),
            KeyCode::F6 => return Some(ButtonCode::F6),
            KeyCode::F7 => return Some(ButtonCode::F7),
            KeyCode::F8 => return Some(ButtonCode::F8),
            KeyCode::F9 => return Some(ButtonCode::F9),
            KeyCode::F10 => return Some(ButtonCode::F10),
            KeyCode::F11 => return Some(ButtonCode::F11),
            KeyCode::F12 => return Some(ButtonCode::F12),
            _ => None,
        }
    }
}
