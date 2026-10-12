//! `LineGutter` — left margin of the editor: line numbers + breakpoint dot.

use egui::{Response, Sense, Ui, Vec2};
use crate::theme::Tokens;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LineState { Normal, CurrentLine }

pub fn show(ui: &mut Ui, t: &Tokens, line: usize, bp_set: bool, state: LineState) -> Response {
    let w = if bp_set { 36.0 } else { 28.0 };
    let h = 18.0_f32;
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(w, h),
        Sense::click(),
    );

    if ui.is_rect_visible(rect) {
        if state == LineState::CurrentLine {
            ui.painter().rect_filled(rect, 0.0, t.code_current_line);
        }
        if bp_set {
            let dot_rect = egui::Rect::from_min_size(
                egui::pos2(rect.min.x + 4.0, rect.min.y + (h - 8.0) * 0.5),
                Vec2::splat(8.0),
            );
            ui.painter().circle_filled(dot_rect.center(), 4.0, t.breakpoint_dot);
        }
        let num = format!("{:>3}", line);
        let galley = ui.painter().layout_no_wrap(
            num,
            egui::FontId::monospace(11.0),
            t.text_muted,
        );
        let text_pos = egui::pos2(
            rect.max.x - galley.size().x - 4.0,
            rect.center().y - galley.size().y * 0.5,
        );
        ui.painter().galley(text_pos, galley, t.text_muted);
    }

    response
}
