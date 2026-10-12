//! Theme: bridges `replica/design/tokens.json` into an `egui::Style`.
//!
//! Per the unix covenant: every colour comes from a token, never hardcoded.
//! The token JSON is `include!`d into the binary so the GUI has no runtime
//! dependency on the file system.

use egui::{Color32, Stroke, Style, Visuals};

#[derive(Clone, Copy, Debug)]
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

fn hex(s: &str) -> Color32 {
    let s = s.trim_start_matches('#');
    let r = u8::from_str_radix(&s[0..2], 16).expect("known invariant: tokens.json hex colors are valid 2-digit-per-channel values");
    let g = u8::from_str_radix(&s[2..4], 16).expect("known invariant: tokens.json hex colors are valid 2-digit-per-channel values");
    let b = u8::from_str_radix(&s[4..6], 16).expect("known invariant: tokens.json hex colors are valid 2-digit-per-channel values");
    Color32::from_rgb(r, g, b)
}

/// Tokens as they exist at the time the design system was last updated.
/// Kept in sync with `replica/design/tokens.json` by hand; a test in
/// `tests/tokens_consistent.rs` asserts the values match.
pub const DEFAULT_TOKENS: Tokens = Tokens {
    bg: Color32::from_rgb(0x11, 0x18, 0x13),
    surface: Color32::from_rgb(0x18, 0x20, 0x19),
    surface_raised: Color32::from_rgb(0x21, 0x2a, 0x22),
    border: Color32::from_rgb(0x5b, 0x6b, 0x5e),
    border_input: Color32::from_rgb(0x71, 0x85, 0x71),
    text: Color32::from_rgb(0xe9, 0xf0, 0xea),
    text_muted: Color32::from_rgb(0x96, 0xa8, 0x9b),
    accent: Color32::from_rgb(0x34, 0xd3, 0x99),
    on_accent: Color32::from_rgb(0x0c, 0x14, 0x10),
    danger: Color32::from_rgb(0xf8, 0x71, 0x71),
    success: Color32::from_rgb(0x4a, 0xde, 0x80),
    warning: Color32::from_rgb(0xfb, 0xbf, 0x24),

    syn_keyword: Color32::from_rgb(0x34, 0xd3, 0x99),
    syn_directive: Color32::from_rgb(0xa7, 0x8b, 0xfa),
    syn_comment: Color32::from_rgb(0x96, 0xa8, 0x9b),
    syn_string: Color32::from_rgb(0xfb, 0xbf, 0x24),
    syn_label: Color32::from_rgb(0xfb, 0x92, 0x3c),
    syn_number: Color32::from_rgb(0xf4, 0x72, 0xb6),

    mem_bg: Color32::from_rgb(0x0c, 0x12, 0x0e),
    mem_fg: Color32::from_rgb(0x4a, 0xde, 0x80),
    mem_cursor: Color32::from_rgb(0x34, 0xd3, 0x99),
    code_current_line: Color32::from_rgb(0x21, 0x2a, 0x22),
    breakpoint_dot: Color32::from_rgb(0xf8, 0x71, 0x71),
    breakpoint_hit: Color32::from_rgb(0xfb, 0xbf, 0x24),
    flag_set: Color32::from_rgb(0x4a, 0xde, 0x80),
    flag_clear: Color32::from_rgb(0x71, 0x85, 0x71),
};

impl Tokens {
    /// Parse from the on-disk `tokens.json`. Used by the preview binary
    /// and by future "live reload" support; the GUI itself uses
    /// `DEFAULT_TOKENS` at runtime for zero-friction startup.
    pub fn from_json(value: &serde_json::Value) -> Self {
        let c = value.get("color").expect("known invariant: tokens.json has a \"color\" object at its root");
        let pick = |k: &str| hex(c.get(k).and_then(|v| v.as_str()).unwrap_or("#000000"));
        Self {
            bg: pick("bg"),
            surface: pick("surface"),
            surface_raised: pick("surface-raised"),
            border: pick("border"),
            border_input: pick("border-input"),
            text: pick("text"),
            text_muted: pick("text-muted"),
            accent: pick("accent"),
            on_accent: pick("on-accent"),
            danger: pick("danger"),
            success: pick("success"),
            warning: pick("warning"),
            syn_keyword: pick("syntax-keyword"),
            syn_directive: pick("syntax-directive"),
            syn_comment: pick("syntax-comment"),
            syn_string: pick("syntax-string"),
            syn_label: pick("syntax-label"),
            syn_number: pick("syntax-number"),
            mem_bg: pick("memory-bg"),
            mem_fg: pick("memory-fg"),
            mem_cursor: pick("memory-cursor"),
            code_current_line: pick("code-current-line"),
            breakpoint_dot: pick("breakpoint-dot"),
            breakpoint_hit: pick("breakpoint-hit"),
            flag_set: pick("flag-set"),
            flag_clear: pick("flag-clear"),
        }
    }
}

pub fn apply(style: &mut Style, t: &Tokens) {
    style.visuals = Visuals::dark();
    let v = &mut style.visuals;
    v.override_text_color = Some(t.text);

    v.widgets.noninteractive.bg_fill = t.surface;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, t.border);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, t.text_muted);

    v.widgets.inactive.bg_fill = t.surface_raised;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, t.border_input);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, t.text);

    v.widgets.hovered.bg_fill = t.border_input;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, t.accent);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, t.text);

    v.widgets.active.bg_fill = t.accent;
    v.widgets.active.bg_stroke = Stroke::new(1.0_f32, t.accent);
    v.widgets.active.fg_stroke = Stroke::new(2.0_f32, t.on_accent);

    v.widgets.open.bg_fill = t.surface_raised;
    v.widgets.open.bg_stroke = Stroke::new(1.0_f32, t.accent);
    v.widgets.open.fg_stroke = Stroke::new(1.0_f32, t.text);

    v.window_fill = t.surface;
    v.window_stroke = Stroke::new(1.0_f32, t.border);
    v.panel_fill = t.bg;
    v.extreme_bg_color = t.mem_bg;
    v.faint_bg_color = t.surface;
    v.code_bg_color = t.mem_bg;
    v.hyperlink_color = t.accent;
    v.warn_fg_color = t.warning;
    v.error_fg_color = t.danger;

    v.selection.bg_fill = t.accent;
    v.selection.stroke = Stroke::new(1.0_f32, t.accent);
}
