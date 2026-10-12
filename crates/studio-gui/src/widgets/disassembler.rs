//! Disassembler widget.
//!
//! Renders a list of decoded instructions (from `studio_debug::disassemble`)
//! as a column of `addr | bytes | mnemonic args` rows, with the current IP
//! highlighted and click‑to‑jump‑to‑IP semantics. Read‑only display; the
//! widget is the *view*, the disassembler library is the *model*.

use egui::{self, Color32, Sense, Ui, Vec2};
use studio_debug::disassembler::{DisasmList, DisasmLine};
use studio_isa::Flags;

use crate::theme::Tokens;

/// State the caller owns and passes in. Keeps the widget stateless.
pub struct DisassemblerState<'a> {
    pub list: &'a DisasmList,
    /// Current instruction pointer. The line whose `addr == current_ip` is
    /// highlighted; if `None`, no row is highlighted.
    pub current_ip: Option<u16>,
    /// Whether the panel is in an "active" state (e.g. user just stepped).
    pub focused: bool,
}

const ROW_HEIGHT: f32 = 18.0;

pub fn show(ui: &mut Ui, t: &Tokens, s: &DisassemblerState) {
    // Compute the longest line so the row has a fixed indent for the
    // address column.
    let avail_w = ui.available_width();
    let mut y = ui.cursor().top();
    let indent = 0.0_f32;
    let _ = indent;
    for line in &s.list.lines {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(avail_w, ROW_HEIGHT),
            Sense::click(),
        );
        let is_current = s.current_ip == Some(line.addr);
        if is_current {
            ui.painter().rect_filled(rect, 0.0, t.code_current_line);
        } else if response.hovered() {
            ui.painter().rect_filled(rect, 0.0, t.surface_raised);
        }
        let colour = colour_for(line, t);
        let galley = ui.painter().layout_no_wrap(
            line.format(),
            egui::FontId::monospace(13.0),
            colour,
        );
        let text_pos = egui::pos2(rect.min.x + 6.0, rect.center().y - galley.size().y * 0.5);
        ui.painter().galley(text_pos, galley, colour);
        let _ = response; // future: click-to-jump
        y += ROW_HEIGHT;
    }
    if s.list.truncated {
        // One extra row to signal "end of buffer".
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(avail_w, ROW_HEIGHT),
            Sense::hover(),
        );
        let text = egui::RichText::new("(end of buffer / unknown opcode)")
            .color(t.text_muted)
            .italics();
        ui.painter().text(
            rect.left_center() + egui::vec2(6.0, -6.0),
            egui::Align2::LEFT_TOP,
            text.text(),
            egui::FontId::monospace(12.0),
            t.text_muted,
        );
    }
    let _ = (y, s.focused, Flags::CF);
}

fn colour_for(line: &DisasmLine, t: &Tokens) -> Color32 {
    // Keep this small: a separate colour per common token class is nice but
    // not required for v1. The line is rendered as a single galley, so we
    // can't recolour pieces without splitting the text — defer to a later
    // version that uses `LayoutJob`.
    let _ = t;
    match line.mnemonic.as_str() {
        "RET" | "HLT" | "NOP" => t.syn_directive,
        "JMP" | "JZ" | "JNZ" | "LOOP" | "CALL" => t.syn_directive,
        "INT" => t.warning,
        "PUSH" | "POP" => t.syn_directive,
        _ => t.text,
    }
}
