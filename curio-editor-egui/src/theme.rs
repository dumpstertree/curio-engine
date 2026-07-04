//! Color/style constants ported 1:1 from `src/App.css`'s `:root` block,
//! plus an `apply()` helper that configures egui's `Visuals`/`Style` to
//! match the original VS-Code-ish dark theme.

use eframe::egui::{self, Color32, CornerRadius, Stroke};

pub const BG_PRIMARY: Color32 = Color32::from_rgb(0x1e, 0x1e, 0x1e);
pub const BG_SECONDARY: Color32 = Color32::from_rgb(0x25, 0x25, 0x26);
pub const BG_TERTIARY: Color32 = Color32::from_rgb(0x2d, 0x2d, 0x2d);
pub const BG_HOVER: Color32 = Color32::from_rgb(0x2a, 0x2d, 0x2e);
pub const BG_ACTIVE: Color32 = Color32::from_rgb(0x37, 0x37, 0x3d);
pub const BG_SELECTED: Color32 = Color32::from_rgb(0x09, 0x47, 0x71);
pub const BG_SELECTED_HOVER: Color32 = Color32::from_rgb(0x0d, 0x53, 0x92);
pub const BORDER: Color32 = Color32::from_rgb(0x33, 0x33, 0x33);
pub const BORDER_LIGHT: Color32 = Color32::from_rgb(0x45, 0x45, 0x45);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(0xcc, 0xcc, 0xcc);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(0x96, 0x96, 0x96);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x6b, 0x6b, 0x6b);
pub const ACCENT: Color32 = Color32::from_rgb(0x00, 0x7a, 0xcc);
pub const GREEN: Color32 = Color32::from_rgb(0x4e, 0xc9, 0xb0);
pub const BLUE: Color32 = Color32::from_rgb(0x9c, 0xdc, 0xfe);
pub const ORANGE: Color32 = Color32::from_rgb(0xce, 0x91, 0x78);
pub const PLAY: Color32 = Color32::from_rgb(0x4c, 0xaf, 0x50);
pub const PAUSE: Color32 = Color32::from_rgb(0xf5, 0x9e, 0x0b);
pub const RED: Color32 = Color32::from_rgb(0xf8, 0x71, 0x71);
pub const YELLOW: Color32 = Color32::from_rgb(0xfb, 0xbf, 0x24);

pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    visuals.panel_fill = BG_PRIMARY;
    visuals.window_fill = BG_SECONDARY;
    visuals.extreme_bg_color = BG_TERTIARY;
    visuals.faint_bg_color = BG_HOVER;
    visuals.code_bg_color = BG_TERTIARY;

    visuals.widgets.noninteractive.bg_fill = BG_PRIMARY;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);

    visuals.widgets.inactive.bg_fill = BG_TERTIARY;
    visuals.widgets.inactive.weak_bg_fill = BG_TERTIARY;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);

    visuals.widgets.hovered.bg_fill = BG_HOVER;
    visuals.widgets.hovered.weak_bg_fill = BG_HOVER;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, BORDER_LIGHT);

    visuals.widgets.active.bg_fill = BG_ACTIVE;
    visuals.widgets.active.weak_bg_fill = BG_ACTIVE;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);

    visuals.selection.bg_fill = BG_SELECTED;
    visuals.selection.stroke = Stroke::new(1.0, BLUE);

    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(2);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(2);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(2);
    visuals.widgets.active.corner_radius = CornerRadius::same(2);

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(8.0, 3.0);
    ctx.set_style(style);
}
