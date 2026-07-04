use crate::state::TopTab;
use crate::theme;
use eframe::egui::{self, RichText, Ui};

pub fn show(ui: &mut Ui, tab: TopTab) {
    let label = match tab {
        TopTab::Asset => "Asset Browser",
        TopTab::Input => "Input Mapping",
        TopTab::Prefab => "Prefab Editor",
        TopTab::Play => "Play",
    };

    ui.centered_and_justified(|ui| {
        ui.vertical_centered(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(48.0, 48.0), egui::Sense::hover());
            let painter = ui.painter_at(rect);
            painter.rect_stroke(rect.shrink(6.0), 4.0, egui::Stroke::new(1.0, theme::TEXT_MUTED), egui::StrokeKind::Middle);
            let c = rect.center();
            painter.line_segment([c - egui::vec2(0.0, 8.0), c + egui::vec2(0.0, 8.0)], egui::Stroke::new(1.0, theme::TEXT_MUTED));
            painter.line_segment([c - egui::vec2(8.0, 0.0), c + egui::vec2(8.0, 0.0)], egui::Stroke::new(1.0, theme::TEXT_MUTED));

            ui.add_space(8.0);
            ui.label(RichText::new(label).color(theme::TEXT_SECONDARY));
            ui.label(RichText::new("Not yet implemented").small().color(theme::TEXT_MUTED));
        });
    });
}
