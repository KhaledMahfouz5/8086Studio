//! `MemoryCell` — one byte of segmented memory, green-on-black, hex + ASCII.
//!
//! Per the unix covenant: this is a paint function. The data lives in the
//! caller; this widget draws a row of bytes for a given (segment, offset)
//! range.

use egui::{Response, Sense, Ui, Vec2};
use crate::theme::Tokens;

pub struct MemoryView {
    pub response: Response,
}

pub fn row(
    ui: &mut Ui,
    t: &Tokens,
    seg: u16,
    base_off: u16,
    bytes: &[u8; 16],
    cursor_byte: Option<usize>,
) -> MemoryView {
    let row_height: f32 = 16.0;
    let char_w: f32 = 7.2;
    let prefix_w: f32 = 11.0 * char_w;
    let hex_w: f32 = (16.0 * 3.0 + 1.0) * char_w;
    let ascii_w: f32 = 17.0 * char_w;
    let total_w: f32 = prefix_w + 8.0 + hex_w + 8.0 + ascii_w;
    let desired = Vec2::new(total_w, row_height);
    let (rect, response) = ui.allocate_exact_size(desired, Sense::hover());

    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(rect, 0.0, t.mem_bg);

        let mono = egui::FontId::monospace(11.0);
        let draw_text = |x: f32, y: f32, s: &str, color: egui::Color32| {
            let g = ui.painter().layout_no_wrap(s.to_owned(), mono.clone(), color);
            ui.painter().galley(egui::pos2(x, y), g, color);
        };

        let y = rect.min.y + 2.0;
        let x0 = rect.min.x;
        draw_text(x0, y, &format!("{:04X}:{:04X}", seg, base_off), t.text_muted);

        let x1 = x0 + prefix_w + 8.0;
        let mut line = String::with_capacity(16 * 3);
        for (i, b) in bytes.iter().enumerate() {
            if i > 0 && i % 8 == 0 { line.push(' '); }
            line.push_str(&format!("{:02X} ", b));
        }
        draw_text(x1, y, &line, t.mem_fg);

        let x2 = x1 + hex_w + 8.0;
        let mut ascii = String::with_capacity(16);
        for &b in bytes {
            let c = if (0x20..0x7F).contains(&b) { b as char } else { '.' };
            ascii.push(c);
        }
        draw_text(x2, y, &format!("|{}|", ascii), t.mem_fg);

        if let Some(idx) = cursor_byte {
            let groups_before: usize = if idx >= 8 { 1 } else { 0 };
            let x_cell = x1 + ((idx as f32) * 3.0 + (groups_before as f32)) * char_w - 1.0;
            let cell_rect = egui::Rect::from_min_size(
                egui::pos2(x_cell, y - 1.0),
                Vec2::new(char_w + 2.0, row_height),
            );
            let cursor_dim = t.mem_cursor.linear_multiply(0.25);
            ui.painter().rect_filled(cell_rect, 0.0, cursor_dim);
            ui.painter().rect_stroke(cell_rect, 0.0,
                egui::Stroke::new(1.0_f32, t.mem_cursor));
        }
    }

    MemoryView { response }
}
