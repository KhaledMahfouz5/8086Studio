//! `studio-bios` — BIOS and DOS INT dispatch stubs.
//!
//! v0 (vertical slice): installs a handler for `INT 21h` only. AH=09h
//! reads the `$`-terminated string at `DS:DX` and returns it via a
//! `&mut Vec<String>` sink the GUI can later render to its stdout pane.
//! AH=4Ch marks the program as exited. All other `INT n` calls are left
//! to the caller (we don't intercept them).

#![forbid(unsafe_code)]

use studio_cpu::{Cpu8086, StepResult};

/// What the BIOS layer asked the host to do. The GUI decides what to do
/// with each entry (push to a stdout pane, terminate, etc.).
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum HostEvent {
    WriteString(String),
    Exit,
}

/// Convenience: install a v0 handler on a CPU that pushes HostEvents
/// into `sink`. Returns the installed handler so callers can compose.
pub fn install_v0(cpu: &mut Cpu8086, sink: std::sync::Arc<std::sync::Mutex<Vec<HostEvent>>>) {
    cpu.int_handler = Some(Box::new(move |cpu, n| {
        if n != 0x21 {
            // Anything else: don't claim it; the caller will see Interrupt(n).
            return Err(StepResult::Interrupt(n));
        }
        let ah = (cpu.reg(studio_isa::Reg16::AX) >> 8) as u8;
        match ah {
            0x09 => {
                let ds = cpu.segs[3];
                let dx = cpu.reg(studio_isa::Reg16::DX);
                let mut out = Vec::new();
                for i in 0..u16::MAX {
                    let b = cpu.read8(ds, dx.wrapping_add(i));
                    if b == b'$' { break; }
                    out.push(b);
                }
                let s = String::from_utf8_lossy(&out).into_owned();
                sink.lock()
                    .expect("known invariant: the host sink mutex is only locked from the single emulator thread")
                    .push(HostEvent::WriteString(s));
                Ok(())
            }
            0x4C => {
                sink.lock()
                    .expect("known invariant: the host sink mutex is only locked from the single emulator thread")
                    .push(HostEvent::Exit);
                cpu.halted = true;
                Ok(())
            }
            _ => Err(StepResult::Interrupt(0x21)),
        }
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use studio_isa::Reg16;
    use std::sync::{Arc, Mutex};

    #[test]
    fn int_21_ah_09_writes_dollar_terminated_string() {
        let mut cpu = Cpu8086::new();
        // Plant "HELLO$" at DS:DX = 0:0x0200
        cpu.segs[3] = 0;
        for (i, b) in b"HELLO$".iter().enumerate() {
            cpu.write8(0, 0x0200 + i as u16, *b);
        }
        cpu.set_reg(Reg16::DX, 0x0200);
        cpu.set_reg(Reg16::AX, 0x0900);

        let sink = Arc::new(Mutex::new(Vec::new()));
        install_v0(&mut cpu, sink.clone());
        // Plant INT 21h at CS:IP = 0:0x0100
        cpu.load(0, 0x0100, &[0xCD, 0x21]);
        let r = cpu.step();
        assert_eq!(r, StepResult::Advanced);
        let events = sink.lock().unwrap().clone();
        assert_eq!(events, vec![HostEvent::WriteString("HELLO".into())]);
    }
}
