//! `StatusBar` — full-width strip with left/center/right sections.

use egui::{Align, Ui};
use crate::theme::Tokens;

#[derive(Default, Debug)]
pub struct Status {
    pub left: String,
    pub center: String,
    pub right: String,
    pub error: Option<String>,
}

pub fn show(ui: &mut Ui, t: &Tokens, status: &Status) {
    let frame = egui::Frame::group(&egui::Style::default())
        .fill(t.surface)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .rounding(egui::Rounding::same(0.0))
        .inner_margin(egui::Margin::symmetric(8.0, 2.0));
    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width() * 0.34, 20.0),
                egui::Layout::left_to_right(Align::Center),
                |ui| ui.label(egui::RichText::new(&status.left).color(t.text_muted).size(11.0)),
            );
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width() * 0.5, 20.0),
                egui::Layout::left_to_right(Align::Center),
                |ui| {
                    let msg = status.error.clone().unwrap_or_else(|| status.center.clone());
                    let color = if status.error.is_some() { t.danger } else { t.text_muted };
                    ui.label(egui::RichText::new(msg).color(color).size(11.0));
                },
            );
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                ui.label(egui::RichText::new(&status.right).color(t.text_muted).size(11.0));
            });
        });
    });
}
