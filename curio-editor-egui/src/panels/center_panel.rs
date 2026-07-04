use crate::state::{CompileStatus, EditorMode, EditorState, LogLevel};
use crate::theme;
use eframe::egui::{self, RichText, Ui};

const RESOLUTIONS: &[&str] = &["1280x720", "1920x1080", "2560x1440"];

/// Strips ANSI escape sequences (`ESC[...m`) emitted by the `colored` crate.
/// The original TS console overlay parsed these into colored spans; that's
/// left as a follow-up (see doc comment on `console_overlay`) — for now the
/// text is just de-escaped so it's at least readable.
fn strip_ansi(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for c2 in chars.by_ref() {
                if c2 == 'm' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn show(ui: &mut Ui, state: &mut EditorState) {
    ui.vertical(|ui| {
        ui.set_width(ui.available_width());

        viewport_toolbar(ui, state);
        ui.add_space(4.0);

        let viewport_height = ui.available_height() - 44.0; // leave room for play bar
        egui::Frame::NONE.fill(theme::BG_TERTIARY).show(ui, |ui| {
            ui.set_min_size(egui::vec2(ui.available_width(), viewport_height.max(120.0)));
            viewport(ui, state);
        });

        compile_error_modal(ui, state);

        ui.add_space(4.0);
        play_bar(ui, state);
    });
}

// ── Viewport toolbar ────────────────────────────────────────────────────────

fn viewport_toolbar(ui: &mut Ui, state: &mut EditorState) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Resolution").small().color(theme::TEXT_SECONDARY));
        egui::ComboBox::from_id_salt("resolution_dropdown").selected_text(RESOLUTIONS[0]).show_ui(ui, |ui| {
            for r in RESOLUTIONS {
                ui.selectable_label(false, *r);
            }
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let label = if state.console_open { "Console \u{25BC}".to_string() } else if state.unread_logs > 0 { format!("Console ({})", state.unread_logs.min(99)) } else { "Console".to_string() };
            let btn = egui::Button::new(label).fill(if state.console_open { theme::BG_ACTIVE } else { theme::BG_TERTIARY });
            if ui.add(btn).clicked() {
                state.toggle_console();
            }
        });
    });
}

// ── Viewport body ───────────────────────────────────────────────────────────

fn viewport(ui: &mut Ui, state: &mut EditorState) {
    if state.mode == EditorMode::Stopped {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(40.0, 40.0), egui::Sense::hover());
                let painter = ui.painter_at(rect);
                painter.circle_stroke(rect.center(), 17.0, egui::Stroke::new(1.2, theme::TEXT_MUTED));
                let c = rect.center();
                painter.add(egui::Shape::convex_polygon(
                    vec![c + egui::vec2(-4.0, -7.0), c + egui::vec2(10.0, 0.0), c + egui::vec2(-4.0, 7.0)],
                    theme::TEXT_MUTED,
                    egui::Stroke::NONE,
                ));
                ui.add_space(6.0);
                ui.label(RichText::new("Press Play to launch").color(theme::TEXT_MUTED));
            });
        });
    } else {
        game_texture(ui, state);
    }

    if state.compile_status == CompileStatus::Compiling {
        egui::Area::new("compile_indicator".into()).anchor(egui::Align2::LEFT_TOP, egui::vec2(10.0, 10.0)).show(ui.ctx(), |ui| {
            egui::Frame::NONE.fill(theme::BG_SECONDARY).inner_margin(6).corner_radius(3).show(ui, |ui| {
                ui.label(RichText::new("Compiling\u{2026}").color(theme::ACCENT));
            });
        });
    }

    console_overlay(ui, state);
}

/// Displays the live game frame. **Uses CPU-readback frames, not the
/// zero-copy shared-texture trick the GLB/PNG/Spine/prefab previews use** —
/// deliberate, see `runner/capture.rs`'s doc comment: the game runner now
/// has its own fully private headless device precisely so it can't race
/// with eframe's own paint loop, which means it can't register directly
/// into eframe's `egui_wgpu::Renderer` either (that would reintroduce the
/// exact cross-thread contention this design avoids). Instead,
/// `runner::capture::take_latest_frame()` hands over plain RGBA bytes each
/// repaint, uploaded into a persistent `egui::TextureHandle`
/// (`state.game_texture_handle`) via `.set(...)` — a real CPU→GPU reupload
/// each frame, same cost the original Tauri build's canvas-per-frame
/// approach already paid, so not a regression versus what was previously
/// working.
fn game_texture(ui: &mut Ui, state: &mut EditorState) {
    if let Some(frame) = crate::runner::capture::take_latest_frame() {
        let color_image = egui::ColorImage::from_rgba_unmultiplied([frame.width as usize, frame.height as usize], &frame.rgba);
        match &mut state.game_texture_handle {
            Some(handle) => handle.set(color_image, egui::TextureOptions::LINEAR),
            None => state.game_texture_handle = Some(ui.ctx().load_texture("game_viewport", color_image, egui::TextureOptions::LINEAR)),
        }
    }

    let Some(handle) = state.game_texture_handle.clone() else {
        // Runner thread hasn't rendered a frame yet (e.g. the instant after
        // pressing Play, before the plugin finishes loading).
        ui.centered_and_justified(|ui| {
            ui.label(RichText::new("Waiting for first frame\u{2026}").color(theme::TEXT_MUTED));
        });
        return;
    };

    // Fit the texture into the available space, preserving aspect ratio,
    // centered — same behavior as the old `<canvas>`'s CSS `object-fit`.
    let tex_size = handle.size_vec2();
    let avail = ui.available_size();
    let tex_aspect = tex_size.x / tex_size.y.max(1.0);
    let avail_aspect = avail.x / avail.y.max(1.0);
    let size = if avail_aspect > tex_aspect { egui::vec2(avail.y * tex_aspect, avail.y) } else { egui::vec2(avail.x, avail.x / tex_aspect) };

    ui.centered_and_justified(|ui| {
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
        ui.painter().image(handle.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
        forward_input(ui, state, &response, rect);
    });

    if state.mode == EditorMode::Paused {
        egui::Area::new("paused_indicator".into()).anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -12.0)).show(ui.ctx(), |ui| {
            egui::Frame::NONE.fill(theme::BG_SECONDARY.gamma_multiply(0.9)).inner_margin(egui::Margin::symmetric(10, 4)).corner_radius(3).show(ui, |ui| {
                ui.label(RichText::new("\u{23F8} Paused").color(theme::PAUSE));
            });
        });
    }
}

/// Mirrors `ViewportCanvas.tsx`'s onPointerMove/onPointerDown/onPointerUp/
/// onKeyDown/onKeyUp handlers — forwards raw input to the running game via
/// `EditorState::send_input` (a no-op unless `mode == Playing`, same guard
/// the old `api.sendInput` callers had).
fn forward_input(ui: &mut Ui, state: &mut EditorState, response: &egui::Response, rect: egui::Rect) {
    use crate::runner::InputEvent;

    if let Some(pos) = response.hover_pos() {
        let local = pos - rect.min;
        state.send_input(InputEvent::Axis { code: 0, x: local.x, y: local.y });
    }

    for &button in &[egui::PointerButton::Primary, egui::PointerButton::Secondary, egui::PointerButton::Middle] {
        let code = match button {
            egui::PointerButton::Primary => 0,
            egui::PointerButton::Secondary => 2,
            egui::PointerButton::Middle => 1,
            _ => continue,
        };
        if response.drag_started_by(button) || (response.clicked_by(button)) {
            state.send_input(InputEvent::Button { code, pressed: true });
        }
        if response.drag_stopped_by(button) {
            state.send_input(InputEvent::Button { code, pressed: false });
        }
    }

    if response.has_focus() || response.clicked() {
        response.request_focus();
    }
    if response.has_focus() {
        ui.input(|i| {
            for event in &i.events {
                if let egui::Event::Key { physical_key, pressed, .. } = event {
                    if let Some(key) = physical_key {
                        state.send_input(InputEvent::Button { code: *key as u32, pressed: *pressed });
                    }
                }
            }
        });
    }
}

// ── Console overlay ─────────────────────────────────────────────────────────

fn console_overlay(ui: &mut Ui, state: &EditorState) {
    if !state.console_open {
        return;
    }
    egui::Area::new("console_overlay".into()).anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(0.0, 0.0)).show(ui.ctx(), |ui| {
        egui::Frame::NONE.fill(theme::BG_PRIMARY.gamma_multiply(0.96)).inner_margin(8).show(ui, |ui| {
            ui.set_width(ui.ctx().screen_rect().width());
            ui.set_max_height(220.0);
            egui::ScrollArea::vertical().stick_to_bottom(true).auto_shrink([false, false]).show(ui, |ui| {
                if state.logs.is_empty() {
                    ui.weak("No output yet");
                }
                for line in &state.logs {
                    let color = match line.level {
                        LogLevel::Error => theme::RED,
                        LogLevel::Warn => theme::YELLOW,
                        LogLevel::Info => theme::BLUE,
                    };
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&line.time).monospace().small().color(theme::TEXT_MUTED));
                        ui.label(RichText::new(strip_ansi(&line.message)).monospace().small().color(color));
                    });
                }
            });
        });
    });
}

// ── Compile error modal ─────────────────────────────────────────────────────

fn compile_error_modal(ui: &mut Ui, state: &mut EditorState) {
    if state.compile_status != CompileStatus::Error || state.compile_error.is_empty() {
        return;
    }
    let mut open = true;
    egui::Window::new("Compile Failed").collapsible(false).resizable(true).open(&mut open).show(ui.ctx(), |ui| {
        egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
            ui.label(RichText::new(&state.compile_error).monospace().color(theme::RED));
        });
    });
    if !open {
        state.compile_error.clear();
    }
}

// ── Play bar ─────────────────────────────────────────────────────────────────

fn play_bar(ui: &mut Ui, state: &mut EditorState) {
    ui.horizontal(|ui| {
        let playing = state.mode == EditorMode::Playing;
        let stopped = state.mode == EditorMode::Stopped;

        if ui.add_enabled(!playing, egui::Button::new(RichText::new("\u{25B6} Play").color(if playing { theme::PLAY } else { theme::TEXT_PRIMARY }))).clicked() {
            state.play();
        }
        if ui.add_enabled(!stopped, egui::Button::new("\u{23F8} Pause")).clicked() {
            state.pause();
        }
        if ui.add_enabled(!stopped, egui::Button::new("\u{25A0} Stop")).clicked() {
            state.stop();
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let (text, color) = match state.mode {
                EditorMode::Playing => ("\u{25CF} Playing", theme::PLAY),
                EditorMode::Paused => ("\u{23F8} Paused", theme::PAUSE),
                EditorMode::Stopped => ("\u{25A0} Stopped", theme::TEXT_SECONDARY),
            };
            ui.label(RichText::new(text).color(color));
        });
    });
}
