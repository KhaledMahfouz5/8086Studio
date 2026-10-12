//! End-to-end composition test — the vertical slice through the real stack.
//!
//! `asm::assemble → cpu::load → cpu::step → bios::dispatch (INT 21h/09h)`
//! with no GUI in sight. If this passes, the library boundary is proven:
//! the exact same calls the `studio` binary makes produce the expected
//! registers, flags, memory and stdout.

use std::sync::{Arc, Mutex};

use studio_asm::{assemble, AsmResult, OutputFormat};
use studio_bios::{install_v0, HostEvent};
use studio_cpu::{Cpu8086, StepResult};
use studio_isa::Reg16;

/// Assemble once, load a fresh CPU, run to completion (or `max_steps`).
/// Returns the host events, and the final CPU.
fn run_asm(result: &AsmResult, max_steps: u64) -> (Vec<HostEvent>, Cpu8086) {
    let sink = Arc::new(Mutex::new(Vec::new()));
    let mut cpu = Cpu8086::new();
    install_v0(&mut cpu, sink.clone());
    cpu.load(result.entry_cs, result.entry_ip, &result.bytes);
    for _ in 0..max_steps {
        if cpu.halted { break; }
        let r = cpu.step();
        if let StepResult::InvalidOpcode(b) = r {
            panic!("invalid opcode {b:#04X} at IP={:04X}", cpu.ip);
        }
    }
    let events = std::mem::take(&mut *sink.lock().unwrap());
    (events, cpu)
}

fn run(source: &str, max_steps: u64) -> (Vec<HostEvent>, Cpu8086) {
    let result = assemble(source, OutputFormat::Com).expect("assembles");
    run_asm(&result, max_steps)
}

#[test]
fn hello_world_prints_hello_world() {
    let (events, cpu) = run(include_str!("../../../examples/hello.asm"), 1_000);
    let printed: Vec<&str> = events.iter().filter_map(|e| match e {
        HostEvent::WriteString(s) => Some(s.as_str()),
        _ => None,
    }).collect();
    assert_eq!(printed, vec!["hello, world"]);
    assert!(cpu.halted, "program should have halted at RET");
}

#[test]
fn sum_adds_two_immediates() {
    // 1234h + 5678h = 68ACh. sum.asm then sets AH=4Ch to exit, so the low
    // byte (AC) is what we check — it must be the sum's low byte.
    let (events, cpu) = run(include_str!("../../../examples/sum.asm"), 1_000);
    assert_eq!((cpu.reg(Reg16::AX) & 0xFF) as u8, 0xAC, "low byte of 1234h+5678h");
    assert!(events.contains(&HostEvent::Exit), "expected exit event");
    assert!(cpu.halted);
}

#[test]
fn loop_instruction_counts_down() {
    let src = "org 100h
mov cx, 5
top:
mov ah, 2
mov dl, 41h
int 21h
loop top
mov ax, 4C00h
int 21h
";
    let (events, cpu) = run(src, 1_000);
    assert_eq!(cpu.reg(Reg16::CX), 0);
    assert!(events.contains(&HostEvent::Exit));
}

#[test]
fn push_pop_round_trips_registers() {
    // pushpop.asm: push 100/50, clobber, pop, add → AX low byte = 150 (0x96).
    let (events, cpu) = run(include_str!("../../../examples/pushpop.asm"), 1_000);
    assert_eq!((cpu.reg(Reg16::AX) & 0xFF) as u8, 150 & 0xFF, "100+50 low byte");
    assert_eq!((cpu.reg(Reg16::AX) >> 8) as u8, 0x4C, "exit code AH=4Ch overwrote high byte");
    assert_eq!(cpu.reg(Reg16::BX), 50, "restored from stack");
    assert!(events.contains(&HostEvent::Exit));
}

#[test]
fn add_two_arrays_sums_elementwise() {
    // Runs the shipped example: a=[1..5], b=[10,50,90,…] → c=[11,22,33,44,55].
    let result = assemble(include_str!("../../../examples/add_two_arrays.asm"), OutputFormat::Com).expect("assembles");
    let c_addr = result.labels.iter()
        .find(|(n, _)| n == "C")
        .map(|(_, a)| *a)
        .expect("label C");
    let (_events, cpu) = run_asm(&result, 1_000);
    let got: Vec<u8> = (0..5).map(|i| cpu.read8(0, c_addr + i)).collect();
    assert_eq!(got, vec![11, 22, 33, 44, 55]);
}

#[test]
fn flags_update_after_cmp() {
    // cmp sets ZF when equal.
    let src = "org 100h
mov ax, 5
cmp ax, 5
ret
";
    let (_events, cpu) = run(src, 100);
    assert!(cpu.flags.contains(studio_isa::Flags::ZF), "CMP equal must set ZF");
}
