//! Design-system preview window.
//!
//! Renders every primitive defined in `components.md` that ships in v1, in
//! every state the spec requires, in a single window. This is the
//! "screenshot the page" step from `replica-design`. Build with
//! `cargo run --bin studio-preview`.

use eframe::egui;
use studio_isa::Flags;
use crate::theme::{apply, Tokens, DEFAULT_TOKENS};
use crate::widgets::{
    breakpoint_dot, button, code_panel, editor, flag_bit, line_gutter, memory_cell,
    pane_tab_strip, register_cell, status_bar, tool_button, toolbar,
};
use button::{ButtonSize, ButtonVariant};
use editor::EditorRow;
use line_gutter::LineState;
use pane_tab_strip::Pane;
use status_bar::Status;

#[derive(Default)]
pub struct PreviewApp;

impl eframe::App for PreviewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let t = DEFAULT_TOKENS;
        egui::TopBottomPanel::top("title").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.heading(
                    egui::RichText::new("8086Studio — design preview")
                        .color(t.text)
                        .size(18.0),
                );
                ui.add_space(12.0);
                ui.label(egui::RichText::new("every v1 widget, every state").color(t.text_muted));
            });
            ui.add_space(8.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(12.0);

                section(ui, &t, "Toolbar (composes buttons)", |ui, t| {
                    toolbar::show(ui, t, |ui, t| {
                        button::show(ui, t, "New",     ButtonVariant::Secondary, ButtonSize::Sm);
                        button::show(ui, t, "Open",    ButtonVariant::Secondary, ButtonSize::Sm);
                        button::show(ui, t, "Save",    ButtonVariant::Secondary, ButtonSize::Sm);
                        button::show(ui, t, "Compile", ButtonVariant::Primary,   ButtonSize::Sm);
                        button::show(ui, t, "Emulate", ButtonVariant::Primary,   ButtonSize::Sm);
                    });
                });

                section(ui, &t, "Button", |ui, t| {
                    for (label, variant) in &[
                        ("Compile",  ButtonVariant::Primary),
                        ("Open",     ButtonVariant::Secondary),
                        ("Reset",    ButtonVariant::Ghost),
                        ("Stop",     ButtonVariant::Danger),
                    ] {
                        button::show(ui, t, label, *variant, ButtonSize::Md);
                        ui.add_space(4.0);
                    }
                    ui.add_space(8.0);
                    for size in [ButtonSize::Sm, ButtonSize::Md, ButtonSize::Lg] {
                        button::show(ui, t, "Run", ButtonVariant::Primary, size);
                        ui.add_space(4.0);
                    }
                });

                section(ui, &t, "ToolButton (icon-only, toggled / not)", |ui, t| {
                    tool_button::show(ui, t, "▶", false);
                    ui.add_space(2.0);
                    tool_button::show(ui, t, "⏸", true);
                    ui.add_space(2.0);
                    tool_button::show(ui, t, "⟳", false);
                });

                section(ui, &t, "Editor (read-only view of hello.asm)", |ui, t| {
                    let rows = vec![
                        EditorRow { line: 1, text: "org 100h".into(),            bp: false, current: false },
                        EditorRow { line: 2, text: "".into(),                    bp: false, current: false },
                        EditorRow { line: 3, text: "mov dx, msg".into(),         bp: true,  current: false },
                        EditorRow { line: 4, text: "mov ah, 9".into(),           bp: false, current: false },
                        EditorRow { line: 5, text: "int 21h".into(),             bp: true,  current: true  },
                        EditorRow { line: 6, text: "ret".into(),                 bp: false, current: false },
                        EditorRow { line: 7, text: "".into(),                    bp: false, current: false },
                        EditorRow { line: 8, text: "msg db \"hello, world$\"".into(), bp: false, current: false },
                    ];
                    editor::show(ui, t, &rows);
                });

                section(ui, &t, "LineGutter (normal / current / with BP)", |ui, t| {
                    ui.horizontal(|ui| {
                        line_gutter::show(ui, t, 12, false, LineState::Normal);
                        line_gutter::show(ui, t, 13, false, LineState::CurrentLine);
                        line_gutter::show(ui, t, 14, true,  LineState::Normal);
                    });
                });

                section(ui, &t, "RegisterCell", |ui, t| {
                    ui.label(egui::RichText::new("Idle").color(t.text_muted));
                    for (i, name) in ["AX", "BX", "CX", "DX"].iter().enumerate() {
                        let v: u16 = [0x1234, 0x0042, 0xFFFF, 0x0000][i];
                        let _ = register_cell::show(ui, t, name, v,
                            register_cell::RegKind::Bit16, register_cell::RegCellState::Idle);
                    }
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("Focused").color(t.text_muted));
                    for (i, name) in ["AX", "BX", "CX", "DX"].iter().enumerate() {
                        let v: u16 = [0x1234, 0x0042, 0xFFFF, 0x0000][i];
                        let _ = register_cell::show(ui, t, name, v,
                            register_cell::RegKind::Bit16, register_cell::RegCellState::Focused);
                    }
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("Changed-since-step (warning flash)").color(t.text_muted));
                    let _ = register_cell::show(ui, t, "AX", 0x1235,
                        register_cell::RegKind::Bit16, register_cell::RegCellState::ChangedSinceStep);
                });

                section(ui, &t, "FlagBit", |ui, t| {
                    for (flag, name) in Flags::PANE_ORDER {
                        flag_bit::show(ui, t, *flag, name, true);
                    }
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("(cleared)").color(t.text_muted));
                    for (flag, name) in Flags::PANE_ORDER {
                        flag_bit::show(ui, t, *flag, name, false);
                    }
                });

                section(ui, &t, "MemoryCell (segmented view, 16 bytes/row)", |ui, t| {
                    let bytes = *b"\x48\x65\x6C\x6C\x6F\x2C\x20\x57\x6F\x72\x6C\x64\x21\x0A\x00\xFF";
                    memory_cell::row(ui, t, 0x0100, 0x0000, &bytes, Some(4));
                    let bytes2 = *b"\xB8\x34\x12\xBB\x42\x00\xCD\x21\xB4\x09\xBA\x00\x01\xCD\x21\xC3";
                    memory_cell::row(ui, t, 0x0100, 0x0010, &bytes2, None);
                    let bytes3 = *b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
                    memory_cell::row(ui, t, 0x0100, 0x0020, &bytes3, None);
                });

                section(ui, &t, "CodePanel (CS:IP disassembly view)", |ui, t| {
                    use code_panel::CodeRow;
                    let rows = vec![
                        CodeRow { address: 0x0100, bytes: vec![0xBA, 0x0E, 0x01],  mnemonic: "mov dx, 010Eh".into() },
                        CodeRow { address: 0x0103, bytes: vec![0xB4, 0x09],       mnemonic: "mov ah, 009h".into() },
                        CodeRow { address: 0x0105, bytes: vec![0xCD, 0x21],       mnemonic: "int 021h".into() },
                        CodeRow { address: 0x0107, bytes: vec![0xC3],             mnemonic: "ret".into() },
                    ];
                    code_panel::show(ui, t, &rows, Some(0x0105), &[0x0105]);
                });

                section(ui, &t, "PaneTabStrip (Screen / Source / … / Flags)", |ui, t| {
                    let _ = pane_tab_strip::show(ui, t, Pane::Debug);
                });

                section(ui, &t, "BreakpointDot (set / hit)", |ui, t| {
                    breakpoint_dot::show(ui, t, false);
                    ui.add_space(8.0);
                    breakpoint_dot::show(ui, t, true);
                });

                section(ui, &t, "StatusBar (idle / busy / error)", |ui, t| {
                    let s1 = Status {
                        left: "hello.asm".into(),
                        center: "Line 5, Col 8".into(),
                        right: "ready".into(),
                        error: None,
                    };
                    status_bar::show(ui, t, &s1);
                    ui.add_space(4.0);
                    let s2 = Status {
                        left: "hello.asm".into(),
                        center: "Line 5, Col 8".into(),
                        right: "compiling…".into(),
                        error: None,
                    };
                    status_bar::show(ui, t, &s2);
                    ui.add_space(4.0);
                    let s3 = Status {
                        left: "hello.asm".into(),
                        center: "".into(),
                        right: "stopped".into(),
                        error: Some("stack underflow at CS:0107".into()),
                    };
                    status_bar::show(ui, t, &s3);
                });
            });
        });
    }
}

fn section(ui: &mut egui::Ui, t: &Tokens, name: &str, body: impl FnOnce(&mut egui::Ui, &Tokens)) {
    ui.add_space(8.0);
    egui::Frame::group(&egui::Style::default())
        .fill(t.surface)
        .stroke(egui::Stroke::new(1.0_f32, t.border))
        .rounding(egui::Rounding::same(6.0))
        .inner_margin(egui::Margin::same(12.0))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(name).color(t.accent).strong().size(14.0));
            ui.add_space(8.0);
            body(ui, t);
        });
    ui.add_space(4.0);
}

pub fn install(_cc: &eframe::CreationContext<'_>) {}

fn _silence_import_warnings(_t: &Tokens) {
    let _ = apply;
}
