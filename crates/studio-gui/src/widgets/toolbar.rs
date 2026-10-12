//! `Toolbar` — composes buttons in a dense horizontal strip.
//!
//! One job: lay out its children. Caller picks which buttons (via callback)
//! and what they do. No state.

use egui::Ui;
use crate::theme::Tokens;

pub fn show(ui: &mut Ui, t: &Tokens, add_contents: impl FnOnce(&mut Ui, &Tokens)) {
    let frame = egui::Frame::group(&egui::Style::default())
        .fill(t.surface)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::same(4.0));
    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            add_contents(ui, t);
        });
    });
}
