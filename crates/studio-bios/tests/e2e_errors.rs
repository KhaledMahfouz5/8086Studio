//! Error-path and edge-case end-to-end tests.
//!
//! Each case maps to `replica/test-plan.md` (F01-N1 … F04-E1). These run
//! through the same `assemble → load → step/run` pipeline as the happy-path
//! tests in `e2e_hello.rs`; they assert the failure contract (the error
//! message and the source line) rather than just "something went wrong".

use std::sync::{Arc, Mutex};

use studio_asm::{assemble, OutputFormat};
use studio_bios::{install_v0, HostEvent};
use studio_cpu::{Cpu8086, StepResult};
use studio_isa::Reg16;

fn run(source: &str, max_steps: u64) -> (Vec<HostEvent>, Cpu8086) {
    let r = assemble(source, OutputFormat::Com).expect("assembles");
    let sink = Arc::new(Mutex::new(Vec::new()));
    let mut cpu = Cpu8086::new();
    install_v0(&mut cpu, sink.clone());
    cpu.load(r.entry_cs, r.entry_ip, &r.bytes);
    for _ in 0..max_steps {
        if cpu.halted { break; }
        if let StepResult::InvalidOpcode(b) = cpu.step() {
            panic!("invalid opcode {b:#04X} at IP={:04X}", cpu.ip);
        }
    }
    let events = std::mem::take(&mut *sink.lock().unwrap());
    (events, cpu)
}

// ---- F01: empty / trivial input ----

#[test]
fn f01_e1_empty_source_assembles_to_zero_bytes() {
    let r = assemble("", OutputFormat::Com).expect("ok");
    assert!(r.bytes.is_empty());
    assert_eq!(r.entry_ip, 0x0100);
}

#[test]
fn f01_e4_org_only_produces_zero_bytes() {
    let r = assemble("org 100h", OutputFormat::Com).expect("ok");
    assert!(r.bytes.is_empty());
    assert_eq!(r.entry_ip, 0x0100);
}

// ---- F01: negative cases (error message + line number) ----

#[test]
fn f01_n1_unknown_mnemonic_reports_line() {
    let e = assemble("wat ax, 5", OutputFormat::Com).expect_err("should fail");
    assert_eq!(e.line, 1);
    assert!(e.message.contains("WAT"), "message: {}", e.message);
}

#[test]
fn f01_n2_missing_operand_reports_mnemonic() {
    let e = assemble("mov ax", OutputFormat::Com).expect_err("should fail");
    assert_eq!(e.line, 1);
    assert!(e.message.contains("MOV"), "message: {}", e.message);
}

#[test]
fn f01_n3_bad_register_reports_operand() {
    let e = assemble("mov qq, 5", OutputFormat::Com).expect_err("should fail");
    assert_eq!(e.line, 1);
    assert!(e.message.contains("unsupported MOV form"), "message: {}", e.message);
}

#[test]
fn f01_n4_bad_numeric_literal_reports_value() {
    let e = assemble("mov ax, 0xGG", OutputFormat::Com).expect_err("should fail");
    // `0xGG` is not a valid hex literal; the parser treats it as a label,
    // which the validation pass rejects as unknown.
    assert!(e.message.contains("0XGG"), "message: {}", e.message);
}

#[test]
fn f01_n5_undefined_label_reports_name() {
    let e = assemble("jmp nowhere", OutputFormat::Com).expect_err("should fail");
    assert!(e.message.contains("NOWHERE"), "message: {}", e.message);
}

#[test]
fn f01_n6_short_branch_out_of_range_reports_rel() {
    // 200 NOPs after `jmp far` puts the target ~200 bytes away — well past
    // the ±127 rel8 range.
    let mut src = String::from("jmp far\n");
    for _ in 0..200 { src.push_str("nop\n"); }
    src.push_str("far:\nret\n");
    let e = assemble(&src, OutputFormat::Com).expect_err("should fail");
    assert!(e.message.contains("out of range"), "message: {}", e.message);
}

// ---- F04: breakpoints ----

#[test]
fn f04_h1_breakpoint_stops_before_executing_target() {
    let src = "org 100h
mov ax, 1
mov bx, 2
mov cx, 3
ret
";
    let result = assemble(src, OutputFormat::Com).expect("ok");
    // `org` produces no EncodedLine; lines[0]=mov ax, lines[1]=mov bx.
    let bp_addr = result.lines[1].address; // `mov bx, 2`
    let sink = Arc::new(Mutex::new(Vec::new()));
    let mut cpu = Cpu8086::new();
    install_v0(&mut cpu, sink);
    cpu.load(0, result.entry_ip, &result.bytes);

    // Step once past the entry, then stop before the breakpoint target.
    cpu.step(); // mov ax, 1
    let mut hit = false;
    for _ in 0..100 {
        if cpu.halted { break; }
        if cpu.ip == bp_addr { hit = true; break; }
        cpu.step();
    }
    assert!(hit, "should stop at the breakpoint address {bp_addr:#06X}");
    assert_eq!(cpu.reg(Reg16::AX), 1, "first instruction ran before the BP");
    assert_eq!(cpu.reg(Reg16::BX), 0, "BP target not yet executed");
}

#[test]
fn f04_e1_breakpoint_at_entry_runs_once_then_stops() {
    // A self-looping program so IP returns to the entry-point BP.
    let src = "org 100h
top:
mov ax, 42
jmp top
";
    let result = assemble(src, OutputFormat::Com).expect("ok");
    let entry = result.entry_ip;
    let sink = Arc::new(Mutex::new(Vec::new()));
    let mut cpu = Cpu8086::new();
    install_v0(&mut cpu, sink);
    cpu.load(0, result.entry_ip, &result.bytes);

    // First step always runs (BP at entry doesn't block the very first
    // instruction). The loop then sees IP==entry on the next visit.
    cpu.step(); // mov ax, 42
    let mut stopped = false;
    for _ in 0..100 {
        if cpu.halted { break; }
        if cpu.ip == entry { stopped = true; break; }
        cpu.step();
    }
    assert!(stopped, "loop should hit the entry-point BP");
    assert_eq!(cpu.reg(Reg16::AX), 42, "entry instruction ran once");
}

// ---- F07: edit + recompile ----

#[test]
fn f07_h1_edit_string_then_recompile_shows_new_text() {
    let original = "org 100h
mov dx, msg
mov ah, 9
int 21h
ret
msg db \"hello$\"
";
    let (events, _cpu) = run(original, 100);
    let printed: Vec<&str> = events.iter().filter_map(|e| match e {
        HostEvent::WriteString(s) => Some(s.as_str()),
        _ => None,
    }).collect();
    assert_eq!(printed, vec!["hello"]);

    // "Edit" the string and recompile.
    let edited = original.replace("hello", "world");
    let (events2, _cpu2) = run(&edited, 100);
    let printed2: Vec<&str> = events2.iter().filter_map(|e| match e {
        HostEvent::WriteString(s) => Some(s.as_str()),
        _ => None,
    }).collect();
    assert_eq!(printed2, vec!["world"]);
}

// ---- F02: register / flag update during single-step ----

#[test]
fn f02_h1_single_step_loop_updates_cx_each_iteration() {
    let src = "org 100h
mov cx, 3
top:
dec cx
jne top
ret
";
    let r = assemble(src, OutputFormat::Com).expect("ok");
    let sink = Arc::new(Mutex::new(Vec::new()));
    let mut cpu = Cpu8086::new();
    install_v0(&mut cpu, sink);
    cpu.load(0, r.entry_ip, &r.bytes);

    // mov cx, 3
    cpu.step();
    assert_eq!(cpu.reg(Reg16::CX), 3);
    // dec cx → CX=2
    cpu.step();
    assert_eq!(cpu.reg(Reg16::CX), 2);
    assert!(!cpu.flags.contains(studio_isa::Flags::ZF));
    // jne top → jumps
    cpu.step();
    assert_eq!(cpu.ip, r.lines[1].address, "jumps to top");
    // dec cx → CX=1
    cpu.step();
    assert_eq!(cpu.reg(Reg16::CX), 1);
    // jne top → jumps
    cpu.step();
    // dec cx → CX=0, ZF set
    cpu.step();
    assert_eq!(cpu.reg(Reg16::CX), 0);
    assert!(cpu.flags.contains(studio_isa::Flags::ZF));
}
