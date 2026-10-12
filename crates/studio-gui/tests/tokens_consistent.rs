//! Asserts that `replica/design/tokens.json` and the in-code `DEFAULT_TOKENS`
//! agree. Catches drift when a designer changes a value in the JSON but the
//! Rust `Tokens` is hand-mirrored.
//!
//! Per the unix covenant: data, not logic. The JSON is the source of truth
//! at design time; this test makes the build fail if they fall out of sync.

use std::path::PathBuf;
use studio_gui::theme::{Tokens, DEFAULT_TOKENS};

fn tokens_path() -> PathBuf {
    // src/lib.rs → ../replica/design/tokens.json
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // crates/
    p.pop(); // workspace root
    p.push("replica");
    p.push("design");
    p.push("tokens.json");
    p
}

fn parse_json() -> serde_json::Value {
    let raw = std::fs::read_to_string(tokens_path())
        .expect("replica/design/tokens.json should exist; it is the design source of truth");
    serde_json::from_str(&raw).expect("tokens.json should be valid JSON")
}

fn same_hex(a: egui::Color32, b_json: &str) -> bool {
    let s = b_json.trim_start_matches('#');
    let r = u8::from_str_radix(&s[0..2], 16).unwrap();
    let g = u8::from_str_radix(&s[2..4], 16).unwrap();
    let b = u8::from_str_radix(&s[4..6], 16).unwrap();
    a == egui::Color32::from_rgb(r, g, b)
}

#[test]
fn default_tokens_match_tokens_json() {
    let v = parse_json();
    let json = Tokens::from_json(&v);
    let pairs = [
        ("bg",                 DEFAULT_TOKENS.bg,                 json.bg),
        ("surface",            DEFAULT_TOKENS.surface,            json.surface),
        ("surface-raised",     DEFAULT_TOKENS.surface_raised,     json.surface_raised),
        ("border",             DEFAULT_TOKENS.border,             json.border),
        ("border-input",       DEFAULT_TOKENS.border_input,       json.border_input),
        ("text",               DEFAULT_TOKENS.text,               json.text),
        ("text-muted",         DEFAULT_TOKENS.text_muted,         json.text_muted),
        ("accent",             DEFAULT_TOKENS.accent,             json.accent),
        ("on-accent",          DEFAULT_TOKENS.on_accent,          json.on_accent),
        ("danger",             DEFAULT_TOKENS.danger,             json.danger),
        ("success",            DEFAULT_TOKENS.success,            json.success),
        ("warning",            DEFAULT_TOKENS.warning,            json.warning),
        ("syntax-keyword",     DEFAULT_TOKENS.syn_keyword,        json.syn_keyword),
        ("syntax-directive",   DEFAULT_TOKENS.syn_directive,      json.syn_directive),
        ("syntax-comment",     DEFAULT_TOKENS.syn_comment,        json.syn_comment),
        ("syntax-string",      DEFAULT_TOKENS.syn_string,         json.syn_string),
        ("syntax-label",       DEFAULT_TOKENS.syn_label,          json.syn_label),
        ("syntax-number",      DEFAULT_TOKENS.syn_number,         json.syn_number),
        ("memory-bg",          DEFAULT_TOKENS.mem_bg,             json.mem_bg),
        ("memory-fg",          DEFAULT_TOKENS.mem_fg,             json.mem_fg),
        ("memory-cursor",      DEFAULT_TOKENS.mem_cursor,         json.mem_cursor),
        ("code-current-line",  DEFAULT_TOKENS.code_current_line,  json.code_current_line),
        ("breakpoint-dot",     DEFAULT_TOKENS.breakpoint_dot,     json.breakpoint_dot),
        ("breakpoint-hit",     DEFAULT_TOKENS.breakpoint_hit,     json.breakpoint_hit),
        ("flag-set",           DEFAULT_TOKENS.flag_set,           json.flag_set),
        ("flag-clear",         DEFAULT_TOKENS.flag_clear,         json.flag_clear),
    ];
    let mut mismatches: Vec<&str> = Vec::new();
    for (name, default, from_json) in pairs {
        if default != from_json {
            mismatches.push(name);
            eprintln!(
                "drift in {name}: DEFAULT_TOKENS={default:?}, from_json={from_json:?}",
                default = default,
                from_json = from_json,
            );
        }
    }
    assert!(mismatches.is_empty(),
        "{} token(s) drifted between DEFAULT_TOKENS and tokens.json: {:?}",
        mismatches.len(), mismatches);
}

/// Lighter test that only checks one pair, useful when the full check is slow.
#[test]
fn tokens_json_contains_no_unparseable_colors() {
    let v = parse_json();
    let c = v.get("color").expect("color block");
    let mut bad: Vec<&str> = Vec::new();
    for (k, val) in c.as_object().unwrap() {
        if let Some(s) = val.as_str() {
            if !s.starts_with('#') || (s.len() != 7 && s.len() != 4) {
                bad.push(k);
            }
        }
    }
    assert!(bad.is_empty(), "unparseable colors: {:?}", bad);
}

#[test]
fn _suppress_unused_warning() {
    let _ = same_hex(egui::Color32::WHITE, "#ffffff");
}
