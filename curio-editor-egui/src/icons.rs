//! Small hand-drawn icons via `egui::Painter`, used in place of Unicode
//! symbol/emoji glyphs (▸▾⏸■▶●○✕✎⟳⤓🎬📁📦🖼🧩📄 etc.) that this project
//! originally used as `RichText` characters. egui's bundled default fonts
//! (`default_fonts` feature) cover Latin text plus a small, specific set of
//! symbols it needs for its own chrome — NOT general emoji or most
//! Miscellaneous Symbols/Dingbats/Supplemental Arrows ranges. Those
//! rendered as empty "tofu" boxes rather than the intended glyph. Drawing
//! shapes directly with the painter sidesteps font coverage entirely —
//! guaranteed to render regardless of what fonts are installed/bundled.
//!
//! Text-label icons (delete/rename/import/refresh/etc.) are handled at the
//! call site instead, by just using plain ASCII words/letters — see the
//! individual files. This module is only for the shapes worth actually
//! drawing (tree chevrons, status dots, play/pause/stop).

use eframe::egui::{self, Color32, Painter, Pos2, Response, Sense, Stroke, Ui};

/// Small clickable expand/collapse chevron (▸ when collapsed, ▾ when
/// expanded), drawn as a filled triangle. Returns the response so the
/// caller can check `.clicked()`.
pub fn chevron(ui: &mut Ui, expanded: bool, color: Color32) -> Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), Sense::click());
    let c = rect.center();
    let s = 3.2;
    let points = if expanded {
        vec![egui::pos2(c.x - s, c.y - s * 0.4), egui::pos2(c.x + s, c.y - s * 0.4), egui::pos2(c.x, c.y + s * 0.7)]
    } else {
        vec![egui::pos2(c.x - s * 0.4, c.y - s), egui::pos2(c.x - s * 0.4, c.y + s), egui::pos2(c.x + s * 0.7, c.y)]
    };
    ui.painter().add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
    response
}

/// A small circle indicator — filled (leaf / "active") or outlined (has
/// children / "inactive"). Allocates its own tiny bit of space so it
/// composes naturally inside `ui.horizontal(...)`.
pub fn dot(ui: &mut Ui, filled: bool, radius: f32, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(radius * 2.6, radius * 2.6), Sense::hover());
    if filled {
        ui.painter().circle_filled(rect.center(), radius, color);
    } else {
        ui.painter().circle_stroke(rect.center(), radius, Stroke::new(1.2, color));
    }
}

pub fn play_triangle(painter: &Painter, center: Pos2, size: f32, color: Color32) {
    let points = vec![center + egui::vec2(-size * 0.35, -size * 0.55), center + egui::vec2(size * 0.55, 0.0), center + egui::vec2(-size * 0.35, size * 0.55)];
    painter.add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
}

pub fn pause_bars(painter: &Painter, center: Pos2, size: f32, color: Color32) {
    let w = (size * 0.18).max(2.0);
    let h = size * 0.55;
    let gap = size * 0.14;
    painter.rect_filled(egui::Rect::from_center_size(center - egui::vec2(gap, 0.0), egui::vec2(w, h)), 0.0, color);
    painter.rect_filled(egui::Rect::from_center_size(center + egui::vec2(gap, 0.0), egui::vec2(w, h)), 0.0, color);
}

pub fn stop_square(painter: &Painter, center: Pos2, size: f32, color: Color32) {
    painter.rect_filled(egui::Rect::from_center_size(center, egui::vec2(size * 0.55, size * 0.55)), 1.0, color);
}
