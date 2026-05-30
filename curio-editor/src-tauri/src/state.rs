use crate::game::runner2::GameMessage2;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Stopped,
    Playing,
    Paused,
}

pub struct EditorState {
    pub mode: EditorMode,
    pub game_tx: Option<Sender<GameMessage2>>,
    pub game_thread: Option<JoinHandle<()>>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            mode: EditorMode::Stopped,
            game_tx: None,
            game_thread: None,
        }
    }
}
