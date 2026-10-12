//! `Button` — primary | secondary | ghost | danger.

use egui::{Response, Ui, Vec2};
use crate::theme::Tokens;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ButtonVariant { Primary, Secondary, Ghost, Danger }

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ButtonSize { Sm, Md, Lg }

impl ButtonSize {
    fn height(self) -> f32 {
        match self { ButtonSize::Sm => 24.0, ButtonSize::Md => 30.0, ButtonSize::Lg => 40.0 }
    }
    fn pad_x(self) -> f32 {
        match self { ButtonSize::Sm => 8.0, ButtonSize::Md => 12.0, ButtonSize::Lg => 16.0 }
    }
    fn font(self) -> f32 {
        match self { ButtonSize::Sm => 12.0, ButtonSize::Md => 13.0, ButtonSize::Lg => 14.0 }
    }
}

pub fn show(ui: &mut Ui, t: &Tokens, label: &str, variant: ButtonVariant, size: ButtonSize) -> Response {
    let h: f32 = size.height();
    let pad_x: f32 = size.pad_x();
    let font: f32 = size.font();
    let sense = egui::Sense::click();
    let galley = ui.painter().layout_no_wrap(
        label.to_owned(),
        egui::FontId::proportional(font),
        egui::Color32::WHITE,
    );
    let desired = Vec2::new(galley.size().x + pad_x * 2.0, h);
    let (rect, response) = ui.allocate_exact_size(desired, sense);

    let (bg, fg) = match variant {
        ButtonVariant::Primary   => (t.accent, t.on_accent),
        ButtonVariant::Secondary => (t.surface_raised, t.text),
        ButtonVariant::Ghost     => (egui::Color32::TRANSPARENT, t.text_muted),
        ButtonVariant::Danger    => (t.danger, t.on_accent),
    };

    let hovered = response.hovered();
    let draw_bg = if hovered && !matches!(variant, ButtonVariant::Ghost) {
        t.border_input
    } else {
        bg
    };

    if ui.is_rect_visible(rect) {
        if !matches!(variant, ButtonVariant::Ghost) {
            ui.painter().rect_filled(rect, 4.0, draw_bg);
        }
        if hovered {
            ui.painter().rect_stroke(rect, 4.0, egui::Stroke::new(1.0_f32, t.accent));
        }
    }

    if response.has_focus() {
        ui.painter().rect_stroke(rect.shrink(1.0), 4.0,
            egui::Stroke::new(2.0_f32, t.accent));
    }

    let text_pos = egui::pos2(rect.center().x - galley.size().x / 2.0,
                              rect.center().y - galley.size().y / 2.0);
    ui.painter().galley(text_pos, galley, fg);

    response
}
