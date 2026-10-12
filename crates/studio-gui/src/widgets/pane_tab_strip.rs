//! `PaneTabStrip` — bottom-of-emulator pane switcher.
//! Variants: screen | source | reset | aux | vars | debug | stack | flags.

use egui::{Sense, Ui, Vec2};
use crate::theme::Tokens;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Pane {
    Screen, Source, Reset, Aux, Vars, Debug, Stack, Flags,
}

impl Pane {
    pub fn label(self) -> &'static str {
        match self {
            Pane::Screen => "Screen",
            Pane::Source => "Source",
            Pane::Reset  => "Reset",
            Pane::Aux    => "Aux",
            Pane::Vars   => "Vars",
            Pane::Debug  => "Debug",
            Pane::Stack  => "Stack",
            Pane::Flags  => "Flags",
        }
    }

    fn all() -> [Pane; 8] {
        [Pane::Screen, Pane::Source, Pane::Reset, Pane::Aux,
         Pane::Vars,   Pane::Debug,  Pane::Stack, Pane::Flags]
    }
}

pub fn show(ui: &mut Ui, t: &Tokens, active: Pane) -> Option<Pane> {
    let mut chosen: Option<Pane> = None;
    let w: f32 = 80.0;
    let h: f32 = 24.0;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for p in Pane::all() {
            let (rect, response) = ui.allocate_exact_size(
                Vec2::new(w, h),
                Sense::click(),
            );
            if ui.is_rect_visible(rect) {
                let is_active = p == active;
                let bg = if is_active { t.surface_raised } else { t.surface };
                ui.painter().rect_filled(rect, 0.0, bg);
                let label_color = if is_active { t.text } else { t.text_muted };
                let galley = ui.painter().layout_no_wrap(
                    p.label().to_owned(),
                    egui::FontId::proportional(12.0),
                    label_color,
                );
                let text_pos = egui::pos2(
                    rect.center().x - galley.size().x * 0.5,
                    rect.center().y - galley.size().y * 0.5,
                );
                ui.painter().galley(text_pos, galley, label_color);
                if is_active {
                    let accent = egui::Rect::from_min_size(
                        egui::pos2(rect.min.x, rect.max.y - 2.0),
                        Vec2::new(rect.width(), 2.0),
                    );
                    ui.painter().rect_filled(accent, 0.0, t.accent);
                }
                if response.clicked() {
                    chosen = Some(p);
                }
            }
        }
    });
    chosen
}
