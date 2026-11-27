//! The pieces of eframe's `egui_wgpu::RenderState` the asset previews need
//! (GLB/PNG/Spine/prefab-scene — see `glb_viewer.rs`, `anim_viewer.rs`,
//! `prefab_viewer.rs`). Grabbed once from `CreationContext::wgpu_render_state`
//! in `app.rs` and stored on `EditorState`.
//!
//! This used to also be handed to the game runner thread so it could share
//! the same `Device`/`Queue`/`Renderer` for a zero-copy live-game texture.
//! That's been reverted (see `runner/capture.rs`'s doc comment for why) —
//! the game runner now has its own fully private headless device again, and
//! this type exists purely for the previews, which render synchronously on
//! the UI thread and have no cross-thread contention to worry about.

use std::sync::Arc;

#[derive(Clone)]
pub struct RenderShared {
    pub device: Arc<egui_wgpu::wgpu::Device>,
    pub queue: Arc<egui_wgpu::wgpu::Queue>,
    pub render_state: egui_wgpu::RenderState,
}
