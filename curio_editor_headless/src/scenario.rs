//! YAML scenario DSL for the headless runner.
//!
//! One file per experiment; LLMs write it, the runner reads it. Example:
//!
//! ```yaml
//! steps: 180                 # frames to simulate
//! dump_every: 60             # ledger JSON dump every N frames (0 = never)
//! dump_dir: out/ledger
//! capture_dir: out/shots
//! capture_width: 640         # screenshot texture size
//! capture_height: 360
//! capture_frames: [30, 60, 120]
//! time_target_frame_rate: 30.0   # SysRecordTime injection (see todo.md for
//!                                # its dlopen-mode limitations)
//! events:
//!   - frame: 10
//!     button: MousePrimary   # any ButtonCode variant name
//!     pressed: Down          # Down or Up
//!   - frame: 40
//!     button: MousePrimary
//!     pressed: Up
//!   - frame: 90
//!     axis: Cursor           # any AxisCode variant name
//!     x: 0.5
//!     y: -0.25
//! ```

use curio_core::{AxisCode, ButtonCode, ButtonPressed};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A full run description.
#[derive(Default, Clone, Deserialize, Serialize)]
pub struct Scenario {
    pub steps: u64,
    // ledger dumps
    #[serde(default)]
    pub dump_every: u64,
    #[serde(default)]
    pub dump_dir: String,
    // screenshots
    #[serde(default)]
    pub capture_dir: String,
    #[serde(default)]
    pub capture_width: u32,
    #[serde(default)]
    pub capture_height: u32,
    #[serde(default)]
    pub capture_frames: Vec<u64>,
    // injection
    #[serde(default)]
    pub time_target_frame_rate: Option<f32>,
    #[serde(default)]
    pub events: Vec<InputEvent>,
}

/// One input event fired at a frame boundary — forwarded through
/// `Curio::input_button`/`Curio::input_axis`, exactly as the window
/// event handlers in the engine and editor do.
#[derive(Default, Clone, Deserialize, Serialize)]
pub struct InputEvent {
    pub frame: u64,
    // button half
    #[serde(default)]
    pub button: Option<ButtonCode>,
    // serde enum name: `Down` / `Up`
    #[serde(default)]
    pub pressed: Option<ButtonPressed>,
    // axis half
    #[serde(default)]
    pub axis: Option<AxisCode>,
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
}

impl Scenario {
    /// Events bucketed by frame, for the frame loop's lookup.
    pub fn events_by_frame(&self) -> HashMap<u64, Vec<InputEvent>> {
        let mut by_frame: HashMap<u64, Vec<InputEvent>> = HashMap::new();
        for i in 0..self.events.len() {
            let e = &self.events[i];
            if let Some(list) = by_frame.get_mut(&e.frame) {
                list.push(e.clone());
            } else {
                by_frame.insert(e.frame, vec![e.clone()]);
            }
        }
        return by_frame;
    }
}
