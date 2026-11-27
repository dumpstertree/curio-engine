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
//! individual files. This module is for the shapes worth actually drawing:
//! tree chevrons, status dots, play/pause/stop, and file-type icons (the
//! asset tree used to show `"[Dir]"`/`"[Png]"`/etc. bracketed text for
//! these instead).

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
    ui.painter()
        .add(egui::Shape::convex_polygon(points, color, Stroke::NONE));
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
        ui.painter()
            .circle_stroke(rect.center(), radius, Stroke::new(1.2, color));
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

// ─────────────────────────────────────────────────────────────────────────────
// File-type icons — used by the asset tree in place of the `"[Dir]"`/
// `"[Png]"`/`"[Glb]"`/`"[Anim]"`/`"[Comp]"`/`"[File]"` bracketed-text tags it
// used to show. Same rationale as the rest of this module: hand-drawn shapes
// can't ever come up missing the way a font glyph or an image asset could.
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FileIconKind {
    Dir,
    Png,
    Glb,
    Anim,
    Comp,
    Other,
}

/// Draws a small file/folder-type icon and returns an interactive response
/// for it (`Sense::click_and_drag()`, matching what the asset tree's rows
/// need it for — selection, drag source, and right-click menu all hang off
/// this response the same way they would off a `Label`).
pub fn file_icon(ui: &mut Ui, kind: FileIconKind, color: Color32) -> Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    let r = rect.shrink(1.5);
    let stroke = Stroke::new(1.1, color);
    match kind {
        FileIconKind::Dir => draw_folder(&painter, r, stroke),
        FileIconKind::Png => draw_image(&painter, r, stroke),
        FileIconKind::Glb => draw_cube(&painter, r, stroke),
        FileIconKind::Anim => draw_anim(&painter, r, color, stroke),
        FileIconKind::Comp => draw_comp(&painter, r, stroke),
        FileIconKind::Other => draw_generic_file(&painter, r, stroke),
    }
    response
}

fn draw_folder(painter: &Painter, r: egui::Rect, stroke: Stroke) {
    let tab_w = r.width() * 0.55;
    let tab_h = r.height() * 0.2;
    let body_top = r.top() + tab_h;
    painter.rect_stroke(egui::Rect::from_min_max(r.min, egui::pos2(r.left() + tab_w, body_top)), 1.0, stroke, egui::StrokeKind::Inside);
    painter.rect_stroke(egui::Rect::from_min_max(egui::pos2(r.left(), body_top), r.max), 1.0, stroke, egui::StrokeKind::Inside);
}

fn draw_image(painter: &Painter, r: egui::Rect, stroke: Stroke) {
    painter.rect_stroke(r, 1.0, stroke, egui::StrokeKind::Inside);
    let sun_c = egui::pos2(r.left() + r.width() * 0.3, r.top() + r.height() * 0.32);
    painter.circle_stroke(sun_c, r.width() * 0.12, stroke);
    let base_y = r.bottom() - r.height() * 0.24;
    let p1 = egui::pos2(r.left() + r.width() * 0.1, base_y);
    let p2 = egui::pos2(r.left() + r.width() * 0.42, r.top() + r.height() * 0.48);
    let p3 = egui::pos2(r.left() + r.width() * 0.66, base_y - r.height() * 0.08);
    let p4 = egui::pos2(r.right() - r.width() * 0.12, r.top() + r.height() * 0.58);
    let p5 = egui::pos2(r.right() - r.width() * 0.06, base_y);
    painter.line_segment([p1, p2], stroke);
    painter.line_segment([p2, p3], stroke);
    painter.line_segment([p3, p4], stroke);
    painter.line_segment([p4, p5], stroke);
}

/// Isometric wireframe cube (hexagon outline + 3 spokes to alternating
/// corners) — the standard way to draw a "3D model/box" glyph with nothing
/// but straight lines.
fn draw_cube(painter: &Painter, r: egui::Rect, stroke: Stroke) {
    let c = r.center();
    let radius = r.width().min(r.height()) * 0.5;
    let pts: Vec<Pos2> = [-90.0_f32, -30.0, 30.0, 90.0, 150.0, 210.0]
        .iter()
        .map(|deg| {
            let rad = deg.to_radians();
            c + egui::vec2(radius * rad.cos(), radius * rad.sin())
        })
        .collect();
    for i in 0..6 {
        painter.line_segment([pts[i], pts[(i + 1) % 6]], stroke);
    }
    painter.line_segment([c, pts[0]], stroke);
    painter.line_segment([c, pts[2]], stroke);
    painter.line_segment([c, pts[4]], stroke);
}

/// Simplified running figure — distinct silhouette (filled head) so it
/// reads clearly at a glance next to the other, purely-outlined icons.
fn draw_anim(painter: &Painter, r: egui::Rect, color: Color32, stroke: Stroke) {
    let head_c = egui::pos2(r.center().x - r.width() * 0.06, r.top() + r.height() * 0.22);
    painter.circle_filled(head_c, r.width() * 0.13, color);
    let shoulder = egui::pos2(head_c.x, head_c.y + r.height() * 0.16);
    let hip = egui::pos2(shoulder.x + r.width() * 0.08, shoulder.y + r.height() * 0.3);
    painter.line_segment([shoulder, hip], stroke);
    painter.line_segment([hip, egui::pos2(hip.x - r.width() * 0.32, r.bottom())], stroke);
    painter.line_segment([hip, egui::pos2(hip.x + r.width() * 0.28, r.bottom() - r.height() * 0.18)], stroke);
    painter.line_segment([shoulder, egui::pos2(shoulder.x - r.width() * 0.28, shoulder.y + r.height() * 0.18)], stroke);
    painter.line_segment([shoulder, egui::pos2(shoulder.x + r.width() * 0.26, shoulder.y + r.height() * 0.1)], stroke);
}

/// 2x2 grid of small squares — reads as "grouped/composed parts", for
/// prefab (`.comp`) files.
fn draw_comp(painter: &Painter, r: egui::Rect, stroke: Stroke) {
    let gap = r.width() * 0.16;
    let sq = (r.width() - gap) / 2.0;
    for row in 0..2 {
        for col in 0..2 {
            let min = r.min + egui::vec2(col as f32 * (sq + gap), row as f32 * (sq + gap));
            painter.rect_stroke(egui::Rect::from_min_size(min, egui::vec2(sq, sq)), 1.0, stroke, egui::StrokeKind::Inside);
        }
    }
}

/// Page with a folded corner — generic/unrecognized file type.
fn draw_generic_file(painter: &Painter, r: egui::Rect, stroke: Stroke) {
    let fold = r.width() * 0.3;
    let tl = r.left_top();
    let tr_fold = egui::pos2(r.right() - fold, r.top());
    let corner = egui::pos2(r.right(), r.top() + fold);
    let br = r.right_bottom();
    let bl = r.left_bottom();
    painter.line_segment([tl, tr_fold], stroke);
    painter.line_segment([tr_fold, corner], stroke);
    painter.line_segment([corner, br], stroke);
    painter.line_segment([br, bl], stroke);
    painter.line_segment([bl, tl], stroke);
    painter.line_segment([tr_fold, egui::pos2(r.right() - fold, r.top() + fold)], stroke);
    painter.line_segment([egui::pos2(r.right() - fold, r.top() + fold), corner], stroke);
}
