//! `Editor` — a monospace text view with line numbers, optional breakpoint
//! dots, and a single highlighted current line.
//!
//! The editor is read-only by design in the design step. v1's edit loop
//! will use egui's `TextEdit` underneath; for the preview we render the
//! source as monospace text with one row per source line. This keeps the
//! widget composable and testable in isolation.

use egui::{Sense, Ui, Vec2};
use crate::theme::Tokens;
use crate::widgets::line_gutter::{self, LineState};

#[derive(Debug, Clone)]
pub struct EditorRow {
    pub line: usize,
    pub text: String,
    pub bp: bool,
    pub current: bool,
}

pub fn show(ui: &mut Ui, t: &Tokens, rows: &[EditorRow]) {
    let row_h: f32 = 18.0;
    for r in rows {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            let state = if r.current { LineState::CurrentLine } else { LineState::Normal };
            line_gutter::show(ui, t, r.line, r.bp, state);
            let (rect, _response) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), row_h),
                Sense::hover(),
            );
            if ui.is_rect_visible(rect) {
                if r.current {
                    ui.painter().rect_filled(rect, 0.0, t.code_current_line);
                }
                let galley = ui.painter().layout_no_wrap(
                    r.text.clone(),
                    egui::FontId::monospace(13.0),
                    t.text,
                );
                let text_pos = egui::pos2(
                    rect.min.x + 6.0,
                    rect.center().y - galley.size().y * 0.5,
                );
                ui.painter().galley(text_pos, galley, t.text);
            }
        });
    }
}
