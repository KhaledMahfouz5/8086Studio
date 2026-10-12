//! The v1 `App` — the working emulator UI.
//!
//! Layout:
//! - Top: toolbar (New / Open / Save / Assemble / Emulate / Run / Step / Stop / Reset)
//! - Left: code panel (S06) — disassembly view with current line + BPs
//! - Right: registers (AX..DI + AH/AL/etc.) + flags + memory (DS:0..DS:256 hex) + status
//! - Bottom: status bar (file path, status, INT events)
//! - Tabs at the bottom of the right pane switch between "Registers+Flags" and "Memory"

use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use eframe::egui;

use studio_asm::{assemble, AsmError, AsmResult, OutputFormat};
use studio_bios::{install_v0, HostEvent};
use studio_cpu::{Cpu8086, StepResult};
use studio_isa::{Flags, Reg16};

use crate::examples;

#[derive(Default)]
pub struct App {
    pub source: String,
    pub path: Option<PathBuf>,
    pub path_input: String,
    pub examples_open: bool,
    pub last_asm: Option<AsmResult>,
    pub last_error: Option<AsmError>,
    pub status: String,
    pub cpu: Option<Box<Cpu8086>>,
    pub host_events: Arc<Mutex<Vec<HostEvent>>>,
    pub run_state: RunState,
    pub current_line: Option<u32>,
    pub last_run_step_count: u64,
    pub viewer: Option<(Reg16, u16)>,
    pub breakpoints: studio_debug::Breakpoints,
    pub output: Vec<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RunState { Idle, Running, Paused, Halted, BreakpointHit }

impl Default for RunState {
    fn default() -> Self { RunState::Idle }
}

impl App {
    pub fn new() -> Self {
        Self {
            source: default_source(),
            path: None,
            path_input: "hello.asm".into(),
            examples_open: false,
            last_asm: None,
            last_error: None,
            status: "ready — write some 8086 and hit Assemble".into(),
            cpu: None,
            host_events: Arc::new(Mutex::new(Vec::new())),
            run_state: RunState::Idle,
            current_line: None,
            last_run_step_count: 0,
            viewer: None,
            breakpoints: studio_debug::Breakpoints::default(),
            output: Vec::new(),
        }
    }

    fn on_new(&mut self) {
        self.source = default_source();
        self.path = None;
        self.cpu = None;
        self.last_asm = None;
        self.last_error = None;
        self.run_state = RunState::Idle;
        self.output.clear();
        self.current_line = None;
        self.status = "new — .com template loaded".into();
    }

    /// S03 — load one of the bundled examples into the editor.
    fn on_load_example(&mut self, name: &str) {
        if let Some(src) = examples::load_builtin(name) {
            self.source = src.to_owned();
            self.path = Some(PathBuf::from(name));
            self.cpu = None;
            self.last_asm = None;
            self.last_error = None;
            self.run_state = RunState::Idle;
            self.status = format!("Loaded {name}.");
        } else {
            self.status = format!("Unknown example {name}.");
        }
    }

    fn on_open(&mut self) {
        let p = PathBuf::from(&self.path_input);
        let label = p.display().to_string();
        match std::fs::read_to_string(&p) {
            Ok(src) => {
                self.source = src;
                self.path = Some(p);
                self.cpu = None;
                self.last_asm = None;
                self.last_error = None;
                self.run_state = RunState::Idle;
                self.status = format!("opened {label}");
            }
            Err(e) => self.status = format!("open failed: {e}"),
        }
    }

    fn on_save(&mut self) {
        let target = self.path
            .clone()
            .unwrap_or_else(|| PathBuf::from(&self.path_input));
        match std::fs::File::create(&target) {
            Ok(mut f) => match f.write_all(self.source.as_bytes()) {
                Ok(()) => { self.path = Some(target.clone()); self.status = format!("saved {}", target.display()); }
                Err(e) => self.status = format!("save failed: {e}"),
            },
            Err(e) => self.status = format!("save failed: {e}"),
        }
    }

    /// S04 — write the assembled output as a `.com` next to the source.
    fn on_compile_save_com(&mut self) {
        let Some(r) = self.last_asm.clone() else {
            self.status = "Assemble first.".into();
            return;
        };
        let target = self.path
            .as_ref()
            .map(|p| p.with_extension("com"))
            .unwrap_or_else(|| PathBuf::from("output.com"));
        match studio_fmt::write_com(&target, &r.bytes) {
            Ok(()) => self.status = format!("wrote {}", target.display()),
            Err(e) => self.status = format!("write failed: {e}"),
        }
    }

    fn on_compile(&mut self) {
        match assemble(&self.source, OutputFormat::Com) {
            Ok(r) => {
                self.last_asm = Some(r.clone());
                self.last_error = None;
                self.status = format!(
                    "Assembled {} bytes at {:04X}:{:04X}.",
                    r.bytes.len(), r.entry_cs, r.entry_ip
                );
                // Build CPU and load.
                let mut cpu = Cpu8086::new();
                install_v0(&mut cpu, self.host_events.clone());
                let ip = r.entry_ip;
                cpu.load(r.entry_cs, ip, &r.bytes);
                self.cpu = Some(Box::new(cpu));
                self.run_state = RunState::Idle;
                self.output.clear();
                self.current_line = r.lines.first().map(|l| l.line);
            }
            Err(e) => {
                self.last_error = Some(e.clone());
                self.status = format!("Line {}: {}", e.line, e.message);
            }
        }
    }

    fn on_step(&mut self) {
        let (new_state, new_line) = {
            let Some(cpu) = self.cpu.as_mut() else {
                self.status = "Assemble first.".into();
                return;
            };
            let r = cpu.step();
            let state = match r {
                StepResult::Halted => RunState::Halted,
                StepResult::Interrupt(_) => RunState::Paused,
                StepResult::StackUnderflow => RunState::Halted,
                StepResult::InvalidOpcode(_) => RunState::Halted,
                StepResult::Advanced => RunState::Paused,
            };
            (state, cpu.ip)
        };
        self.run_state = new_state;
        self.current_line = self.line_for_ip(new_line);
        self.drain_events_and_update_status();
        if self.run_state == RunState::Halted {
            self.status = format!("Halted at {new_line:04X}.");
        }
    }

    fn on_run(&mut self, max_steps: u64) {
        let mut count = 0u64;
        let mut last_ip = 0u16;
        let bps: std::collections::HashSet<u16> = self.breakpoints.addrs.iter().copied().collect();
        let final_state = loop {
            let Some(cpu) = self.cpu.as_mut() else { break RunState::Idle };
            if cpu.halted { break RunState::Halted; }
            // Stop *before* executing at a breakpointed address (after the
            // first step so a BP at the entry still runs the entry once).
            if count > 0 && bps.contains(&cpu.ip) { break RunState::BreakpointHit; }
            let r = cpu.step();
            last_ip = cpu.ip;
            count += 1;
            match r {
                StepResult::Halted => break RunState::Halted,
                StepResult::Interrupt(_) => break RunState::Paused,
                StepResult::Advanced => RunState::Running,
                StepResult::StackUnderflow => break RunState::Halted,
                StepResult::InvalidOpcode(_) => break RunState::Halted,
            };
            if count >= max_steps { break RunState::Paused; }
        };
        self.run_state = final_state;
        self.last_run_step_count = count;
        self.current_line = self.line_for_ip(last_ip);
        self.drain_events_and_update_status();
        let stop_ip = self.cpu.as_ref().map(|c| c.ip).unwrap_or(last_ip);
        match self.run_state {
            RunState::Halted => self.status = format!("Halted at {stop_ip:04X}."),
            RunState::BreakpointHit => self.status = format!("Stopped at breakpoint {stop_ip:04X}."),
            _ => {}
        }
    }

    fn drain_events_and_update_status(&mut self) {
        let events = {
            let mut g = self.host_events
                .lock()
                .expect("known invariant: host_events mutex is only locked from the single GUI thread");
            std::mem::take(&mut *g)
        };
        if !events.is_empty() {
            let mut parts = Vec::new();
            for e in &events {
                match e {
                    HostEvent::WriteString(s) => { self.output.push(s.clone()); parts.push(format!("`{s}`")); }
                    HostEvent::Exit => { parts.push("exit".to_string()); }
                }
            }
            self.status = format!("BIOS: {}", parts.join(", "));
        }
    }

    fn on_reset(&mut self) {
        if let Some(r) = self.last_asm.clone() {
            let mut cpu = Cpu8086::new();
            install_v0(&mut cpu, self.host_events.clone());
            cpu.load(r.entry_cs, r.entry_ip, &r.bytes);
            self.cpu = Some(Box::new(cpu));
            self.host_events
                .lock()
                .expect("known invariant: host_events mutex is only locked from the single GUI thread")
                .clear();
            self.run_state = RunState::Idle;
            self.output.clear();
            self.current_line = r.lines.first().map(|l| l.line);
            self.last_run_step_count = 0;
            self.status = "reset".into();
        }
    }

    fn line_for_ip(&self, ip: u16) -> Option<u32> {
        let r = self.last_asm.as_ref()?;
        // Pick the line whose address is the closest below-or-equal to ip,
        // so a `mov dx, msg` jumps to `msg`'s line on its first byte.
        let mut best: Option<(u32, u16)> = None;
        for l in &r.lines {
            if l.address <= ip && best.map_or(true, |(_, a)| a < l.address) {
                best = Some((l.line, l.address));
            }
        }
        best.map(|(line, _)| line)
    }

    fn toggle_breakpoint(&mut self, addr: u16) {
        if self.breakpoints.contains(addr) {
            self.breakpoints.clear(addr);
            self.status = format!("Breakpoint cleared at {addr:04X}.");
        } else {
            self.breakpoints.set(addr);
            self.status = format!("Breakpoint set at {addr:04X}.");
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let t = crate::theme::DEFAULT_TOKENS;
        // Top: toolbar.
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                if crate::widgets::button::show(ui, &t, "New",     crate::widgets::button::ButtonVariant::Secondary, crate::widgets::button::ButtonSize::Sm).clicked() {
                    self.on_new();
                }
                ui.add_space(4.0);
                if crate::widgets::button::show(ui, &t, "Open",    crate::widgets::button::ButtonVariant::Secondary, crate::widgets::button::ButtonSize::Sm).clicked() {
                    self.on_open();
                }
                ui.add_space(4.0);
                if crate::widgets::button::show(ui, &t, "Save",    crate::widgets::button::ButtonVariant::Secondary, crate::widgets::button::ButtonSize::Sm).clicked() {
                    self.on_save();
                }
                ui.add_space(4.0);
                if crate::widgets::button::show(ui, &t, "Examples", crate::widgets::button::ButtonVariant::Ghost,    crate::widgets::button::ButtonSize::Sm).clicked() {
                    self.examples_open = !self.examples_open;
                }
                ui.add_space(8.0);
                ui.add_sized([180.0, 24.0],
                    egui::TextEdit::singleline(&mut self.path_input)
                        .font(egui::TextStyle::Monospace)
                        .hint_text("path.asm"));
                ui.add_space(8.0);
                ui.label(egui::RichText::new("|").color(t.border));
                ui.add_space(8.0);
                if crate::widgets::button::show(ui, &t, "Assemble", crate::widgets::button::ButtonVariant::Primary,   crate::widgets::button::ButtonSize::Sm).clicked() {
                    self.on_compile();
                }
                ui.add_space(4.0);
                if crate::widgets::button::show(ui, &t, "Save .com", crate::widgets::button::ButtonVariant::Secondary, crate::widgets::button::ButtonSize::Sm).clicked() {
                    self.on_compile_save_com();
                }
                ui.add_space(4.0);
                if crate::widgets::button::show(ui, &t, "Step",    crate::widgets::button::ButtonVariant::Secondary, crate::widgets::button::ButtonSize::Sm).clicked() {
                    self.on_step();
                }
                ui.add_space(4.0);
                if crate::widgets::button::show(ui, &t, "Run",     crate::widgets::button::ButtonVariant::Primary,   crate::widgets::button::ButtonSize::Sm).clicked() {
                    self.on_run(10_000);
                }
                ui.add_space(4.0);
                if crate::widgets::button::show(ui, &t, "Stop",    crate::widgets::button::ButtonVariant::Danger,    crate::widgets::button::ButtonSize::Sm).clicked() {
                    self.run_state = RunState::Paused;
                    self.status = "stopped".into();
                }
                ui.add_space(4.0);
                if crate::widgets::button::show(ui, &t, "Reset",   crate::widgets::button::ButtonVariant::Ghost,     crate::widgets::button::ButtonSize::Sm).clicked() {
                    self.on_reset();
                }
                ui.add_space(12.0);
                ui.label(egui::RichText::new(format!("state: {:?}", self.run_state)).color(t.text_muted).size(11.0));
            });
            if self.examples_open {
                ui.add_space(2.0);
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Examples:").color(t.text_muted).size(11.0));
                    for (name, blurb) in examples::EMBEDDED {
                        let resp = ui.button(
                            egui::RichText::new(*name).color(t.accent).monospace()
                        ).on_hover_text(*blurb);
                        if resp.clicked() {
                            self.on_load_example(name);
                            self.examples_open = false;
                        }
                    }
                });
            }
            ui.add_space(4.0);
        });

        // Bottom: status bar.
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            let status = self.status.clone();
            let err = self.last_error.as_ref().map(|e| e.message.clone());
            let sb = crate::widgets::status_bar::Status {
                left: self.path.as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "(unsaved)".into()),
                center: "8086Studio".into(),
                right: status.clone(),
                error: err,
            };
            crate::widgets::status_bar::show(ui, &t, &sb);
        });

        // Main: split into editor (top-left), emulator (right), errors (below editor).
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Left: editor + errors.
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("Source").color(t.accent).strong().size(12.0));
                    ui.add_space(2.0);
                    egui::Frame::group(&egui::Style::default())
                        .fill(t.surface)
                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::same(8.0))
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(280.0)
                                .show(ui, |ui| {
                                    ui.add(
                                        egui::TextEdit::multiline(&mut self.source)
                                            .font(egui::TextStyle::Monospace)
                                            .code_editor()
                                            .desired_rows(12)
                                            .desired_width(f32::INFINITY),
                                    );
                                });
                        });
                    ui.add_space(8.0);
                    // Errors panel (S05).
                    if let Some(e) = &self.last_error {
                        ui.label(egui::RichText::new("Assembly errors").color(t.danger).strong().size(12.0));
                        egui::Frame::group(&egui::Style::default())
                            .fill(t.surface_raised)
                            .stroke(egui::Stroke::new(1.0_f32, t.danger))
                            .rounding(egui::Rounding::same(4.0))
                            .inner_margin(egui::Margin::same(8.0))
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new(format!("Line {}: {}", e.line, e.message))
                                    .color(t.danger)
                                    .size(12.0));
                            });
                    }
                });
                ui.add_space(8.0);

                // Right: emulator pane.
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new("Emulator").color(t.accent).strong().size(12.0));
                    ui.add_space(2.0);

                    // Code panel (S06).
                    egui::Frame::group(&egui::Style::default())
                        .fill(t.surface)
                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::same(8.0))
                        .show(ui, |ui| {
                            if let Some(r) = &self.last_asm {
                                use crate::widgets::code_panel::{self, CodeRow};
                                let cpu_ip = self.cpu.as_ref().map(|c| c.ip).unwrap_or(0);
                                let rows: Vec<CodeRow> = r.lines.iter().map(|l| CodeRow {
                                    address: l.address as u32,
                                    bytes: l.bytes.clone(),
                                    mnemonic: l.source.clone(),
                                }).collect();
                                let bps: Vec<u32> = self.breakpoints.addrs.iter().map(|a| *a as u32).collect();
                                let current = if self.run_state != RunState::Idle { Some(cpu_ip as u32) } else { None };
                                if let Some(clicked) = code_panel::show(ui, &t, &rows, current, &bps) {
                                    self.toggle_breakpoint(clicked as u16);
                                }
                            } else {
                                ui.label(egui::RichText::new("(no program loaded)").color(t.text_muted).size(11.0));
                            }
                        });
                    ui.add_space(8.0);

                    // Registers + Flags (S06 + S07 partially).
                    egui::Frame::group(&egui::Style::default())
                        .fill(t.surface)
                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::same(8.0))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Registers + Flags").color(t.accent).strong().size(11.0));
                            ui.add_space(4.0);
                            // Gather data first so the immutable borrow of
                            // `self.cpu` ends before we mutate `self.viewer`.
                            let snapshot = self.cpu.as_ref().map(|cpu| (cpu.regs, cpu.flags, cpu.ip));
                            if let Some((regs, flags, ip)) = snapshot {
                                let mut clicked: Option<(Reg16, u16)> = None;
                                for (i, name) in ["AX", "CX", "DX", "BX", "SP", "BP", "SI", "DI"].iter().enumerate() {
                                    let v = regs[i];
                                    let state = if self.viewer.map(|(r, _)| r == Reg16::ALL[i]).unwrap_or(false) {
                                        crate::widgets::register_cell::RegCellState::Focused
                                    } else {
                                        crate::widgets::register_cell::RegCellState::Idle
                                    };
                                    if crate::widgets::register_cell::show(ui, &t, name, v,
                                        crate::widgets::register_cell::RegKind::Bit16, state)
                                        .response.double_clicked()
                                    {
                                        clicked = Some((Reg16::ALL[i], v));
                                    }
                                }
                                if let Some(sel) = clicked { self.viewer = Some(sel); }
                                ui.add_space(6.0);
                                for (flag, name) in Flags::PANE_ORDER {
                                    crate::widgets::flag_bit::show(ui, &t, *flag, name, flags.contains(*flag));
                                }
                                ui.add_space(6.0);
                                ui.label(egui::RichText::new(format!("IP = {ip:04X}"))
                                    .color(t.text).size(12.0));
                            } else {
                                ui.label(egui::RichText::new("(assemble to populate)").color(t.text_muted).size(11.0));
                            }
                        });
                    ui.add_space(8.0);

                    // Memory (S06).
                    egui::Frame::group(&egui::Style::default())
                        .fill(t.surface)
                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::same(8.0))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Memory (DS:0x0000, 16 bytes per row)").color(t.accent).strong().size(11.0));
                            ui.add_space(4.0);
                            if let Some(cpu) = &self.cpu {
                                for row in 0..4u16 {
                                    let addr = row * 16;
                                    let mut bytes = [0u8; 16];
                                    for (i, b) in bytes.iter_mut().enumerate() {
                                        *b = cpu.read8(0, addr + i as u16);
                                    }
                                    crate::widgets::memory_cell::row(ui, &t, 0, addr, &bytes, None);
                                }
                            } else {
                                ui.label(egui::RichText::new("(no memory)").color(t.text_muted).size(11.0));
                            }
                        });
                    ui.add_space(8.0);

                    // BIOS output console — where INT 21h/09h writes land.
                    egui::Frame::group(&egui::Style::default())
                        .fill(t.mem_bg)
                        .stroke(egui::Stroke::new(1.0_f32, t.border))
                        .rounding(egui::Rounding::same(4.0))
                        .inner_margin(egui::Margin::same(8.0))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Output (INT 21h)").color(t.accent).strong().size(11.0));
                            ui.add_space(4.0);
                            if self.output.is_empty() {
                                ui.label(egui::RichText::new("Run a program to see its output.").color(t.text_muted).size(12.0));
                            } else {
                                for line in &self.output {
                                    ui.label(egui::RichText::new(line).color(t.mem_fg).monospace().size(13.0));
                                }
                            }
                        });

                    // Extended Viewer (S07) — double-click a register to open.
                    if let Some((reg, val)) = self.viewer {
                        ui.add_space(8.0);
                        egui::Frame::group(&egui::Style::default())
                            .fill(t.surface_raised)
                            .stroke(egui::Stroke::new(1.0_f32, t.accent))
                            .rounding(egui::Rounding::same(4.0))
                            .inner_margin(egui::Margin::same(8.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(format!("{} Extended Viewer", reg.name()))
                                        .color(t.accent).strong().size(12.0));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("close").clicked() { self.viewer = None; }
                                    });
                                });
                                ui.add_space(4.0);
                                let signed = val as i16;
                                let bin = (0..16).rev().map(|b| if (val >> b) & 1 == 1 { '1' } else { '0' })
                                    .enumerate().map(|(i, c)| if i > 0 && i % 4 == 0 { format!(" {c}") } else { c.to_string() })
                                    .collect::<String>();
                                ui.label(egui::RichText::new(format!("hex:     {val:#06X}")).color(t.text).monospace().size(12.0));
                                ui.label(egui::RichText::new(format!("dec:     {val}")).color(t.text).monospace().size(12.0));
                                ui.label(egui::RichText::new(format!("signed:  {signed}")).color(t.text).monospace().size(12.0));
                                ui.label(egui::RichText::new(format!("binary:  {bin}")).color(t.text).monospace().size(12.0));
                            });
                    }
                });
            });
        });
    }
}

fn default_source() -> String {
    "; the simplest hello-world. assemble, then run.\n\
mov dx, msg\n\
mov ah, 9\n\
int 21h\n\
ret\n\
msg db \"hello, world$\"\n"
        .to_owned()
}
