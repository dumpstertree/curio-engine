//! `extern "C"` callback stubs originally wired into `EngineServices`
//! (pre-rename `Services`). The current `Services` struct constructed in
//! `game_runner.rs` (`assets`/`logger`/`gpu` only) doesn't reference these
//! anymore, so they're currently unused/orphaned — left in place rather
//! than deleted in case `curio_core` still wants them wired up through a
//! different path. Direct port from the Tauri build either way — these
//! were already no-ops/TODOs there too.

#[unsafe(no_mangle)]
pub extern "C" fn set_resolution(_x: i32, _y: i32) {}

#[unsafe(no_mangle)]
pub extern "C" fn set_fullscreen(_x: bool) {}

#[unsafe(no_mangle)]
pub extern "C" fn set_cursor_visible(_x: bool) {}
