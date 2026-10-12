//! `CodePanel` — read-only disassembly view: address | hex bytes | mnemonic.
//!
//! One row per decoded instruction. Caller hands us a list of
//! `CodeRow { address, bytes, mnemonic }`. We render the row and the
//! breakpoint dot in the left margin.

use egui::{Sense, Ui, Vec2};
use crate::theme::Tokens;

#[derive(Debug, Clone)]
pub struct CodeRow {
    pub address: u32,
    pub bytes: Vec<u8>,
    pub mnemonic: String,
}

pub fn show(ui: &mut Ui, t: &Tokens, rows: &[CodeRow], current: Option<u32>, bp: &[u32]) -> Option<u32> {
    let h: f32 = 16.0;
    let margin: f32 = 8.0;
    let mut clicked: Option<u32> = None;
    for r in rows {
        let bp_set = bp.contains(&r.address);
        let is_current = current == Some(r.address);
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), h),
            Sense::click(),
        );
        if response.clicked() {
            clicked = Some(r.address);
        }
        if ui.is_rect_visible(rect) {
            if is_current {
                ui.painter().rect_filled(rect, 0.0, t.code_current_line);
            }
            if bp_set {
                ui.painter().circle_filled(
                    egui::pos2(rect.min.x + 5.0, rect.center().y),
                    3.0,
                    t.breakpoint_dot,
                );
            }
            let addr = format!("{:04X}:{:04X}", (r.address >> 4) & 0xFFFF, r.address & 0xF);
            let bytes_str: String = r.bytes.iter()
                .take(6)
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");
            let line = format!(
                "{:>11}  {:<17}  {}",
                addr,
                bytes_str,
                r.mnemonic,
            );
            let galley = ui.painter().layout_no_wrap(
                line,
                egui::FontId::monospace(11.0),
                t.text,
            );
            let text_pos = egui::pos2(
                rect.min.x + margin,
                rect.center().y - galley.size().y * 0.5,
            );
            ui.painter().galley(text_pos, galley, t.text);
        }
    }
    clicked
}
