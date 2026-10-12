//! `RegisterCell` — 16-bit or 8-bit register, editable on click.

use egui::{Response, TextEdit, Ui, Vec2};
use crate::theme::Tokens;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RegKind { Bit16, Bit8, Segment }

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RegCellState { Idle, Focused, ChangedSinceStep }

pub struct RegCellResponse {
    pub response: Response,
    pub new_value: Option<u16>,
}

pub fn show(
    ui: &mut Ui,
    t: &Tokens,
    name: &str,
    value: u16,
    kind: RegKind,
    state: RegCellState,
) -> RegCellResponse {
    let width: f32 = match kind {
        RegKind::Bit16 => 72.0,
        RegKind::Bit8 => 60.0,
        RegKind::Segment => 64.0,
    };
    let height: f32 = 20.0;
    let desired = Vec2::new(width, height);
    let sense = egui::Sense::click();
    let (rect, response) = ui.allocate_exact_size(desired, sense);

    let bg = match state {
        RegCellState::Idle => t.surface,
        RegCellState::Focused | RegCellState::ChangedSinceStep => t.surface_raised,
    };
    let border = if matches!(state, RegCellState::Focused) { t.accent } else { t.border };
    let fg = if matches!(state, RegCellState::ChangedSinceStep) { t.warning } else { t.text };

    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(rect, 3.0, bg);
        ui.painter().rect_stroke(rect, 3.0, egui::Stroke::new(1.0_f32, border));
    }

    // Left: register name
    let name_galley = ui.painter().layout_no_wrap(
        name.to_owned(),
        egui::FontId::monospace(11.0),
        t.text_muted,
    );
    ui.painter().galley(
        egui::pos2(rect.min.x + 4.0, rect.center().y - name_galley.size().y / 2.0),
        name_galley,
        t.text_muted,
    );

    // Right: hex value, editable on click
    let value_text = format!("{:04X}", value & 0xFFFF);
    let mut edit_value = value_text.clone();
    let edit_rect = egui::Rect::from_min_size(
        egui::pos2(rect.min.x + width - 38.0, rect.min.y + 2.0),
        Vec2::new(34.0, height - 4.0),
    );
    let edit_response = ui.put(
        edit_rect,
        TextEdit::singleline(&mut edit_value)
            .font(egui::FontId::monospace(11.0))
            .text_color(fg)
            .frame(false)
            .hint_text("0000"),
    );
    let new_value = if edit_response.lost_focus() && edit_value != value_text {
        u16::from_str_radix(&edit_value, 16).ok()
    } else {
        None
    };

    RegCellResponse { response, new_value }
}
