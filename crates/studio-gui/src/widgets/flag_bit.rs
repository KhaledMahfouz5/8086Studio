//! `FlagBit` — 9 condition-code / control flags. Click to toggle.

use egui::{Response, Ui, Vec2};
use studio_isa::Flags;
use crate::theme::Tokens;

pub fn show(ui: &mut Ui, t: &Tokens, flag: Flags, name: &str, set: bool) -> Response {
    let size: f32 = 22.0;
    let sense = egui::Sense::click();
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), sense);

    let bg = if set { t.surface_raised } else { t.surface };
    let dot_color = if set { t.flag_set } else { t.flag_clear };

    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(rect, 3.0, bg);
        ui.painter().rect_stroke(rect, 3.0,
            egui::Stroke::new(1.0_f32, t.border));
        let dot_rect = rect.shrink(6.0);
        ui.painter().rect_filled(dot_rect, 2.0, dot_color);
        if response.has_focus() {
            ui.painter().rect_stroke(rect.shrink(1.0), 3.0,
                egui::Stroke::new(2.0_f32, t.accent));
        }
    }

    if response.hovered() {
        let full = match flag {
            Flags::OF => "Overflow", Flags::DF => "Direction",
            Flags::IF => "Interrupt enable", Flags::TF => "Trap",
            Flags::SF => "Sign", Flags::ZF => "Zero",
            Flags::AF => "Auxiliary carry", Flags::PF => "Parity",
            Flags::CF => "Carry",
            _ => name,
        };
        let msg = format!("{} ({})", full, if set { "set" } else { "clear" });
        response.clone().on_hover_text(msg);
    }
    response
}
