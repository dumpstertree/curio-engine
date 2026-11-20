use curio_core::Ledger;
use egui_wgpu::wgpu::{CommandEncoder, TextureView};

// dabs of oil paint: world-pinned brushstrokes drawn as geometry, not a
// screen-space filter. this crate slots in next to the 3d/post chain the way
// ui_overlay slots its own pass — the rendering system owns the invocation,
// we own the pipeline and buffers.
//
// NOTE: RenderFeature3D is deliberately NOT implemented here. that trait
// lives in the rendering crate, and implementing it from here would cycle the
// dependency (rendering -> us). ui_overlay set the precedent of a crate-local
// trait the system calls directly.
//
// submission contract, phase 1 onward — per dab, ~48 bytes, layout fixed now
// so wind/fire phases never re-pack:
//   pin  vec3   object-LOCAL space, rigid motion is a vertex-shader matrix
//               mult (phase 4 freebie)
//   dir  vec3   the way the stroke runs (phase 1: angle proof)
//   size vec2   across, along, metres (phase 1: anisotropy proof)
//   pal  u32   palette/style id (phase 2)
//   seed vec4  hash salt (phase 2)

// phase 1: dab submission record + static/dynamic instance buffers
// phase 2: wgsl dab shader + pipeline + shared uniforms (time, pixel size)
pub trait OilPaintFeature {
    fn render(&mut self, encoder: &mut CommandEncoder, target_view: &TextureView, ledger: &Ledger);
}

pub struct RenderFeatureDrawDabs {
    // phase 1: pipeline: None, instance buffer + capacity, submitted dabs
    pub tick_count: usize,
}

impl RenderFeatureDrawDabs {
    pub fn new() -> RenderFeatureDrawDabs {
        RenderFeatureDrawDabs { tick_count: 0 }
    }
}

impl OilPaintFeature for RenderFeatureDrawDabs {
    fn render(&mut self, _encoder: &mut CommandEncoder, _target_view: &TextureView, _ledger: &Ledger) {
        // phase 0 checkpoint: pass is invoked every frame, draws nothing
        self.tick_count += 1;
        if self.tick_count % 300 == 1 {
            println!("[oil-paint] dab pass tick {} (stub, draws nothing)", self.tick_count);
        }
    }
}
