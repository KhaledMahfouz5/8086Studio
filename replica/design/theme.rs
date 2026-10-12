// replica/design/theme.rs
// Bridge from `replica/design/tokens.json` to an `egui::Visuals` at runtime.
// egui does not consume CSS variables; the JSON values are copied into the
// runtime structs here. Any change to tokens.json that touches colors should
// be reflected here before the next build.

use egui::{Color32, Stroke, Style, Visuals};

fn hex(s: &str) -> Color32 {
    let s = s.trim_start_matches('#');
    let r = u8::from_str_radix(&s[0..2], 16).unwrap();
    let g = u8::from_str_radix(&s[2..4], 16).unwrap();
    let b = u8::from_str_radix(&s[4..6], 16).unwrap();
    Color32::from_rgb(r, g, b)
}

pub struct Tokens {
    pub bg: Color32,
    pub surface: Color32,
    pub surface_raised: Color32,
    pub border: Color32,
    pub border_input: Color32,
    pub text: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub on_accent: Color32,
    pub danger: Color32,
    pub success: Color32,
    pub warning: Color32,

    pub syn_keyword: Color32,
    pub syn_directive: Color32,
    pub syn_comment: Color32,
    pub syn_string: Color32,
    pub syn_label: Color32,
    pub syn_number: Color32,

    pub mem_bg: Color32,
    pub mem_fg: Color32,
    pub mem_cursor: Color32,
    pub code_current_line: Color32,
    pub breakpoint_dot: Color32,
    pub breakpoint_hit: Color32,
    pub flag_set: Color32,
    pub flag_clear: Color32,
}

impl Tokens {
    pub fn from_json(raw: &serde_json::Value) -> Self {
        let c = |k: &str| hex(raw["color"][k].as_str().unwrap());
        Self {
            bg: c("bg"), surface: c("surface"), surface_raised: c("surface-raised"),
            border: c("border"), border_input: c("border-input"),
            text: c("text"), text_muted: c("text-muted"),
            accent: c("accent"), on_accent: c("on-accent"),
            danger: c("danger"), success: c("success"), warning: c("warning"),
            syn_keyword: c("syntax-keyword"), syn_directive: c("syntax-directive"),
            syn_comment: c("syntax-comment"), syn_string: c("syntax-string"),
            syn_label: c("syntax-label"), syn_number: c("syntax-number"),
            mem_bg: c("memory-bg"), mem_fg: c("memory-fg"), mem_cursor: c("memory-cursor"),
            code_current_line: c("code-current-line"),
            breakpoint_dot: c("breakpoint-dot"), breakpoint_hit: c("breakpoint-hit"),
            flag_set: c("flag-set"), flag_clear: c("flag-clear"),
        }
    }
}

pub fn apply(style: &mut Style, t: &Tokens) {
    style.visuals = Visuals::dark();
    let v = &mut style.visuals;
    v.override_text_color = Some(t.text);
    v.widgets.noninteractive.bg_fill = t.surface;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, t.border);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, t.text_muted);
    v.widgets.inactive.bg_fill = t.surface_raised;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, t.border_input);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, t.text);
    v.widgets.hovered.bg_fill = t.border_input;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, t.accent);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, t.text);
    v.widgets.active.bg_fill = t.accent;
    v.widgets.active.bg_stroke = Stroke::new(1.0, t.accent);
    v.widgets.active.fg_stroke = Stroke::new(2.0, t.on_accent);
    v.widgets.open.bg_fill = t.surface_raised;
    v.widgets.open.bg_stroke = Stroke::new(1.0, t.accent);
    v.widgets.open.fg_stroke = Stroke::new(1.0, t.text);

    v.window_fill = t.surface;
    v.panel_fill = t.bg;
    v.extreme_bg_color = t.mem_bg;
    v.faint_bg_color = t.surface;
    v.code_bg_color = t.mem_bg;
    v.hyperlink_color = t.accent;
    v.warn_fg_color = t.warning;
    v.error_fg_color = t.danger;

    v.selection.bg_fill = t.accent;
    v.selection.stroke = Stroke::new(1.0, t.accent);
}
