//! `ToolButton` — square icon-only toggle for the emulator sub-toolbar.

use egui::{Response, Ui, Vec2};
use crate::theme::Tokens;

pub fn show(ui: &mut Ui, t: &Tokens, glyph: &str, toggled: bool) -> Response {
    let size: f32 = 28.0;
    let sense = egui::Sense::click();
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), sense);

    let (bg, fg) = if toggled {
        (t.accent, t.on_accent)
    } else if response.hovered() {
        (t.surface_raised, t.text)
    } else {
        (t.surface, t.text_muted)
    };

    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(rect, 4.0, bg);
        if response.has_focus() {
            ui.painter().rect_stroke(rect.shrink(1.0), 4.0,
                egui::Stroke::new(2.0_f32, t.accent));
        }
    }

    let galley = ui.painter().layout_no_wrap(
        glyph.to_owned(),
        egui::FontId::proportional(14.0),
        fg,
    );
    let pos = egui::pos2(rect.center().x - galley.size().x / 2.0,
                         rect.center().y - galley.size().y / 2.0);
    ui.painter().galley(pos, galley, fg);
    response
}
