//! `BreakpointDot` — 8px circle in a code-panel gutter or editor gutter.
//! `hit=true` paints in warning color and pulses (the caller controls timing).

use egui::{Response, Sense, Ui, Vec2};
use crate::theme::Tokens;

pub fn show(ui: &mut Ui, t: &Tokens, hit: bool) -> Response {
    let size: f32 = 10.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    if ui.is_rect_visible(rect) {
        let color = if hit { t.breakpoint_hit } else { t.breakpoint_dot };
        let r = rect.shrink(1.0);
        ui.painter().circle_filled(r.center(), r.width() / 2.0, color);
    }
    response
}
