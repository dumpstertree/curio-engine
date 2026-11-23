use crate::panels::{center_panel, inspector, left_panel, placeholder, status_bar, tab_bar, toolbar};
use crate::project::Project;
use crate::render_shared::RenderShared;
use crate::state::{CompileStatus, EditorMode, EditorState, TopTab};
use crate::theme;
use eframe::egui;

pub struct CurioEditorApp {
    state: EditorState,
}

impl CurioEditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>, project: Project) -> Self {
        theme::install_fonts(&cc.egui_ctx);
        theme::apply(&cc.egui_ctx);
        let mut state = EditorState::new(project);

        // Grabbed once, up front: the same Device/Queue/Renderer eframe's
        // own paint pass uses. This is ONLY for the GLB/PNG/Spine/prefab
        // asset previews (`render_shared.rs`) — the game runner has its own
        // fully private headless device (see `runner/capture.rs`'s doc
        // comment for why that's deliberate, not leftover). `RenderState.
        // device`/`.queue` are plain `wgpu::Device`/`Queue` here (already
        // cheap-to-clone handles internally, not wrapped in an extra `Arc`
        // by egui-wgpu) — wrapped in our own `Arc` for API consistency with
        // how the previews pass them around. `render_state` itself is kept
        // whole (see `RenderShared`'s doc comment) so nothing here has to
        // name the exact type of its `renderer` field. Without the "wgpu"
        // eframe feature (see Cargo.toml) `wgpu_render_state` is `None` —
        // previews would need a fallback headless device to work without
        // it, but that path isn't implemented here since it isn't the
        // configuration this was built for.
        let render_state = cc
            .wgpu_render_state
            .as_ref()
            .expect("eframe must be built with the \"wgpu\" renderer (Cargo.toml: eframe features = [..., \"wgpu\"])");
        state.set_render_shared(RenderShared {
            device: std::sync::Arc::new(render_state.device.clone()),
            queue: std::sync::Arc::new(render_state.queue.clone()),
            render_state: render_state.clone(),
        });
        state.ensure_runner_started();

        Self { state }
    }
}

impl eframe::App for CurioEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.state.tick();
        toolbar::handle_shortcuts(ctx, &mut self.state);

        // Keep repainting while there's something moving (playing, compiling,
        // or logs streaming in) — otherwise egui only repaints on input.
        let live = self.state.mode != EditorMode::Stopped || self.state.compile_status == CompileStatus::Compiling;
        if live {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }

        egui::TopBottomPanel::top("toolbar")
            .frame(egui::Frame::NONE.fill(theme::BG_SECONDARY).inner_margin(4))
            .show(ctx, |ui| {
                toolbar::show(ui, &mut self.state);
            });

        egui::TopBottomPanel::top("tab_bar")
            .frame(
                egui::Frame::NONE
                    .fill(theme::BG_SECONDARY)
                    .inner_margin(egui::Margin::symmetric(8, 4)),
            )
            .show(ctx, |ui| {
                tab_bar::show(ui, &mut self.state);
            });

        egui::TopBottomPanel::bottom("status_bar")
            .frame(
                egui::Frame::NONE
                    .fill(theme::BG_SECONDARY)
                    .inner_margin(egui::Margin::symmetric(10, 4)),
            )
            .show(ctx, |ui| {
                status_bar::show(ui, &self.state);
            });

        match self.state.active_tab {
            TopTab::Play => {
                egui::SidePanel::left("left_panel")
                    .frame(egui::Frame::NONE.fill(theme::BG_SECONDARY).inner_margin(6))
                    .default_width(240.0)
                    .resizable(true)
                    .show(ctx, |ui| {
                        left_panel::show(ui, &mut self.state);
                    });
                egui::SidePanel::right("inspector_panel")
                    .frame(egui::Frame::NONE.fill(theme::BG_SECONDARY).inner_margin(6))
                    .default_width(280.0)
                    // Only a minimum here — no max. `inspector.rs` now
                    // bounds every value widget to the space actually
                    // available to it (wrapping text, DragValues that
                    // wrap onto a new line instead of overflowing, etc.),
                    // so content can no longer force the panel wider on
                    // its own; the user is free to drag it as large as
                    // they want.
                    .min_width(220.0)
                    .resizable(true)
                    .show(ctx, |ui| {
                        inspector::show(ui, &self.state);
                    });
            }
            TopTab::Asset => {
                egui::SidePanel::left("asset_tree_panel")
                    .frame(egui::Frame::NONE.fill(theme::BG_SECONDARY).inner_margin(6))
                    .default_width(260.0)
                    .resizable(true)
                    .show(ctx, |ui| {
                        crate::panels::asset_tab::show_tree(ui, &mut self.state);
                    });
                egui::SidePanel::right("asset_inspector_panel")
                    .frame(egui::Frame::NONE.fill(theme::BG_SECONDARY).inner_margin(6))
                    .default_width(280.0)
                    .resizable(true)
                    .show(ctx, |ui| {
                        crate::panels::asset_tab::show_inspector_for_selected(ui, &mut self.state);
                    });
            }
            _ => {}
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(theme::BG_PRIMARY).inner_margin(6))
            .show(ctx, |ui| match self.state.active_tab {
                TopTab::Play => center_panel::show(ui, &mut self.state),
                TopTab::Asset => crate::panels::asset_tab::show_viewport(ui, &mut self.state),
                other => placeholder::show(ui, other),
            });
    }
}
