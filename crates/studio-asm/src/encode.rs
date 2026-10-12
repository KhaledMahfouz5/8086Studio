//! AST → machine bytes: opcode encoding, label resolution, validation.

use std::collections::HashMap;
use studio_isa::{Reg8, Reg16};
use super::*;

pub(crate) fn encode_stmt(it: &Item, labels: &HashMap<String, u16>, consts: &HashMap<String, u16>) -> Result<Vec<u8>, AsmError> {
    match &it.stmt {
        Statement::Directive(d) => match d {
            Directive::Org(_) | Directive::Equ(_, _) | Directive::End => Ok(Vec::new()),
            Directive::Db(v) => Ok(v.clone()),
            Directive::Dw(v) => {
                let mut out = Vec::new();
                for w in v { out.push(*w as u8); out.push((*w >> 8) as u8); }
                Ok(out)
            }
        },
        Statement::Instruction(i) => encode_instr(it.line, it.addr, i, labels, consts),
    }
}

fn encode_instr(
    line: u32,
    here: u16,
    i: &Instruction,
    labels: &HashMap<String, u16>,
    consts: &HashMap<String, u16>,
) -> Result<Vec<u8>, AsmError> {
    Ok(match i {
        Instruction::Mov { dst, src } => encode_mov(line, dst, src, labels, consts)?,
        Instruction::Add { dst, src } => encode_alu(line, 0x01, 0x03, 0x05, dst, src, labels, consts)?,
        Instruction::Sub { dst, src } => encode_alu(line, 0x29, 0x2B, 0x2D, dst, src, labels, consts)?,
        Instruction::Cmp { dst, src } => encode_alu(line, 0x39, 0x3B, 0x3D, dst, src, labels, consts)?,
        Instruction::And { dst, src } => encode_alu(line, 0x21, 0x23, 0x25, dst, src, labels, consts)?,
        Instruction::Or  { dst, src } => encode_alu(line, 0x09, 0x0B, 0x0D, dst, src, labels, consts)?,
        Instruction::Xor { dst, src } => encode_alu(line, 0x31, 0x33, 0x35, dst, src, labels, consts)?,
        Instruction::Inc(o) => match o {
            Operand::Reg16(r) => vec![0x40 | reg16_id(*r)],
            _ => return Err(AsmError { line, message: "INC needs a register".into() }),
        },
        Instruction::Dec(o) => match o {
            Operand::Reg16(r) => vec![0x48 | reg16_id(*r)],
            _ => return Err(AsmError { line, message: "DEC needs a register".into() }),
        },
        Instruction::Push(r) => vec![0x50 | reg16_id(*r)],
        Instruction::Pop(r)  => vec![0x58 | reg16_id(*r)],
        Instruction::JmpShort(o) => short_branch(line, here, 0xEB, o, labels, consts)?,
        Instruction::JeShort(o)  => short_branch(line, here, 0x74, o, labels, consts)?,
        Instruction::JneShort(o) => short_branch(line, here, 0x75, o, labels, consts)?,
        Instruction::JaShort(o)  => short_branch(line, here, 0x77, o, labels, consts)?,
        Instruction::JaeShort(o) => short_branch(line, here, 0x73, o, labels, consts)?,
        Instruction::JbShort(o)  => short_branch(line, here, 0x72, o, labels, consts)?,
        Instruction::JbeShort(o) => short_branch(line, here, 0x76, o, labels, consts)?,
        Instruction::JgeShort(o) => short_branch(line, here, 0x7D, o, labels, consts)?,
        Instruction::JlShort(o)  => short_branch(line, here, 0x7C, o, labels, consts)?,
        Instruction::LoopShort(o)=> short_branch(line, here, 0xE2, o, labels, consts)?,
        Instruction::Lea(dst, src) => {
            // LEA reg16, mem → 8D /r  mod=00, reg=dst, rm=110 (direct [disp16]) or 100/101/111
            let dst = resolve_label_op(line, dst.clone(), labels, consts);
            let src = resolve_label_op(line, src.clone(), labels, consts);
            let dst_reg = match dst {
                Operand::Reg16(r) => reg16_id(r),
                _ => return Err(AsmError { line, message: "LEA needs a 16-bit register".into() }),
            };
            match src {
                Operand::Mem16(addr) => vec![0x8D, mod_reg_rm(0, dst_reg, 6), addr as u8, (addr >> 8) as u8],
                Operand::MemReg(r) => {
                    let rm = mem_reg_rm(r).ok_or_else(|| AsmError {
                        line, message: format!("LEA [{}] not directly addressable in v1", r.name()),
                    })?;
                    vec![0x8D, mod_reg_rm(0, dst_reg, rm)]
                }
                _ => return Err(AsmError { line, message: "LEA needs a memory operand".into() }),
            }
        }
        Instruction::Shr(dst, count) => {
            // SHR rm8, 1 → D0 /5; SHR rm16, 1 → D1 /5
            // SHR rm, imm8 → C0 /5 imm8 or C1 /5 imm8
            let dst = resolve_label_op(line, dst.clone(), labels, consts);
            let count = resolve_label_op(line, count.clone(), labels, consts);
            let n = match count {
                Operand::Imm16(v) => v,
                _ => return Err(AsmError { line, message: "SHR expects an immediate count".into() }),
            };
            match dst {
                Operand::Reg16(r) => {
                    if n == 1 {
                        vec![0xD1, mod_reg_rm(3, 5, reg16_id(r))]
                    } else {
                        vec![0xC1, mod_reg_rm(3, 5, reg16_id(r)), n as u8]
                    }
                }
                Operand::Reg8(r) => {
                    if n == 1 {
                        vec![0xD0, mod_reg_rm(3, 5, reg8_id(r))]
                    } else {
                        vec![0xC0, mod_reg_rm(3, 5, reg8_id(r)), n as u8]
                    }
                }
                _ => return Err(AsmError { line, message: "SHR expects a register".into() }),
            }
        }
        Instruction::Call(o) => {
            // Placeholder 0 for a forward label; `validate_labels_resolved`
            // rejects any label that is never defined before pass 2 emits.
            let target = resolve_label_lenient(o, labels, consts);
            vec![0xE8, target as u8, (target >> 8) as u8]
        }
        Instruction::Int(n) => vec![0xCD, *n],
        Instruction::Ret => vec![0xC3],
        Instruction::Hlt => vec![0xF4],
        Instruction::Nop => vec![0x90],
    })
}

fn encode_mov(
    line: u32,
    dst: &Operand,
    src: &Operand,
    labels: &HashMap<String, u16>,
    consts: &HashMap<String, u16>,
) -> Result<Vec<u8>, AsmError> {
    // Normalize `Label` → `Imm16` (a label's value is its address).
    let dst = resolve_label_op(line, dst.clone(), labels, consts);
    let src = resolve_label_op(line, src.clone(), labels, consts);
    match (dst, src) {
        (Operand::Reg16(d), Operand::Imm16(s)) => Ok(vec![0xB8 | reg16_id(d), s as u8, (s >> 8) as u8]),
        (Operand::Reg8(d), Operand::Imm16(s)) => {
            if s > 0xFF {
                return Err(AsmError { line, message: format!("MOV {}, {:#X} doesn't fit in 8 bits", reg8_name(d), s) });
            }
            Ok(vec![0xB0 | reg8_id(d), s as u8])
        }
        (Operand::Reg8(d), Operand::Reg8(s)) => {
            // 88 /r: MOV r/m8, r8   dst=rm, src=reg
            Ok(vec![0x88, mod_reg_rm(0, reg8_id(s), reg8_id(d))])
        }
        (Operand::Reg16(d), Operand::Reg16(s)) => Ok(vec![0x89, 0xC0 | (reg16_id(s) << 3) | reg16_id(d)]),
        (Operand::Reg16(d), Operand::Mem16(addr)) => {
            Ok(vec![0x8B, mod_reg_rm(0, reg16_id(d), 6), addr as u8, (addr >> 8) as u8])
        }
        (Operand::Reg16(d), Operand::MemReg(s)) => {
            let rm = mem_reg_rm(s).ok_or_else(|| AsmError {
                line, message: format!("[{}] not directly addressable in v1 (use [disp16])", s.name()),
            })?;
            Ok(vec![0x8B, mod_reg_rm(0, reg16_id(d), rm)])
        }
        // MOV reg8, [reg] → 8A /r mod=00 rm=s
        (Operand::Reg8(d), Operand::MemReg(s)) => {
            let rm = mem_reg_rm(s).ok_or_else(|| AsmError {
                line, message: format!("[{}] not directly addressable in v1", s.name()),
            })?;
            Ok(vec![0x8A, mod_reg_rm(0, reg8_id(d), rm)])
        }
        (Operand::Reg8(d), Operand::Mem16(addr)) => {
            Ok(vec![0x8A, mod_reg_rm(0, reg8_id(d), 6), addr as u8, (addr >> 8) as u8])
        }
        // MOV [reg], reg8 → 88 /r
        (Operand::MemReg(arm), Operand::Reg8(s)) => {
            let rm = mem_reg_rm(arm).ok_or_else(|| AsmError {
                line, message: format!("[{}] not directly addressable in v1", arm.name()),
            })?;
            Ok(vec![0x88, mod_reg_rm(0, reg8_id(s), rm)])
        }
        (Operand::Mem16(addr), Operand::Reg8(s)) => {
            Ok(vec![0x88, mod_reg_rm(0, reg8_id(s), 6), addr as u8, (addr >> 8) as u8])
        }
        (Operand::Mem16(addr), Operand::Reg16(s)) => {
            Ok(vec![0x89, mod_reg_rm(0, reg16_id(s), 6), addr as u8, (addr >> 8) as u8])
        }
        (Operand::MemReg(d), Operand::Reg16(s)) => {
            let rm = mem_reg_rm(d).ok_or_else(|| AsmError {
                line, message: format!("[{}] not directly addressable in v1 (use [disp16])", d.name()),
            })?;
            Ok(vec![0x89, mod_reg_rm(0, reg16_id(s), rm)])
        }
        (Operand::Mem16(addr), Operand::Imm16(v)) => {
            Ok(vec![0xC7, mod_reg_rm(0, 0, 6), addr as u8, (addr >> 8) as u8, v as u8, (v >> 8) as u8])
        }
        _ => Err(AsmError { line, message: "unsupported MOV form".into() }),
    }
}

/// Return the mod=00 rm encoding for a register used as a memory pointer.
fn mem_reg_rm(r: Reg16) -> Option<u8> {
    match r {
        Reg16::BX => Some(7),
        Reg16::SI => Some(4),
        Reg16::DI => Some(5),
        _ => None,
    }
}

fn reg8_name(r: Reg8) -> &'static str {
    r.name()
}

fn reg16_id(r: Reg16) -> u8 {
    match r {
        Reg16::AX => 0, Reg16::CX => 1, Reg16::DX => 2, Reg16::BX => 3,
        Reg16::SP => 4, Reg16::BP => 5, Reg16::SI => 6, Reg16::DI => 7,
    }
}

fn reg8_id(r: Reg8) -> u8 {
    match r {
        Reg8::AL => 0, Reg8::CL => 1, Reg8::DL => 2, Reg8::BL => 3,
        Reg8::AH => 4, Reg8::CH => 5, Reg8::DH => 6, Reg8::BH => 7,
    }
}

/// `Operand::Label(name)` → `Operand::Imm16(address)` once we've seen all labels.
fn resolve_label_op(
    _line: u32,
    op: Operand,
    labels: &HashMap<String, u16>,
    consts: &HashMap<String, u16>,
) -> Operand {
    match op {
        Operand::Label(n) => match labels.get(&n).copied().or_else(|| consts.get(&n).copied()) {
            Some(v) => Operand::Imm16(v),
            None => Operand::Imm16(0), // forward ref: value filled in pass 2, shape/length is what matters here
        },
        Operand::MemLabelOff(n, d) => {
            let base = labels.get(&n).copied().or_else(|| consts.get(&n).copied()).unwrap_or(0);
            Operand::Mem16(base.wrapping_add(d))
        }
        other => other,
    }
}

fn encode_alu(
    line: u32,
    op_rm_reg: u8,
    op_reg_rm: u8,
    op_acc_imm: u8,
    dst: &Operand,
    src: &Operand,
    labels: &HashMap<String, u16>,
    consts: &HashMap<String, u16>,
) -> Result<Vec<u8>, AsmError> {
    let dst = resolve_label_op(line, dst.clone(), labels, consts);
    let src = resolve_label_op(line, src.clone(), labels, consts);
    match (dst, src) {
        // ADD/SUB/CMP AX, imm16 → 0x05/0x2D/0x3D + imm16
        (Operand::Reg16(d), Operand::Imm16(v)) if reg16_id(d) == 0 => {
            Ok(vec![op_acc_imm, v as u8, (v >> 8) as u8])
        }
        // ADD/SUB/CMP reg16, imm16 → 81 /0|5|7 + reg + imm16
        (Operand::Reg16(d), Operand::Imm16(v)) => {
            let sub = alu_subfield(op_rm_reg);
            Ok(vec![0x81, mod_reg_rm(3, sub, reg16_id(d)), v as u8, (v >> 8) as u8])
        }
        // ADD/SUB/CMP reg16, reg16 → 01/29/39 + mod_reg_rm
        (Operand::Reg16(d), Operand::Reg16(s)) => {
            Ok(vec![op_rm_reg, mod_reg_rm(3, reg16_id(s), reg16_id(d))])
        }
        // ADD/SUB/CMP reg16, [mem16] → 03/2B/3B + mod_reg_rm(0, d, 6) + disp16
        (Operand::Reg16(d), Operand::Mem16(disp)) => {
            Ok(vec![op_reg_rm, mod_reg_rm(0, reg16_id(d), 6), disp as u8, (disp >> 8) as u8])
        }
        // ADD/SUB/CMP reg16, [reg] (v1: BX/SI/DI)
        (Operand::Reg16(d), Operand::MemReg(s)) => {
            let rm = mem_reg_rm(s).ok_or_else(|| AsmError {
                line, message: format!("[{}] not directly addressable in v1", s.name()),
            })?;
            Ok(vec![op_reg_rm, mod_reg_rm(0, reg16_id(d), rm)])
        }
        // ADD/SUB/CMP reg8, [reg] — needs 8-bit ALU extension, not in v1
        // CMP al, [si] is special: 8A /r reads memory, then CMP AL, value isn't right.
        // The Intel 8-bit forms are 38/3A + mod_reg_rm + (disp).
        (Operand::Reg8(d), Operand::MemReg(s)) => {
            let rm = mem_reg_rm(s).ok_or_else(|| AsmError {
                line, message: format!("[{}] not directly addressable in v1", s.name()),
            })?;
            // 3A /r (CMP r8, r/m8) — dst=reg, src=mem
            // But 3A expects the /reg field to be the dst register, /rm to be mem.
            // For mod=00 rm=rm, reg=dst.
            Ok(vec![if op_reg_rm == 0x3B { 0x3A } else { 0x3A },
                    mod_reg_rm(0, reg8_id(d), rm)])
        }
        (Operand::Reg8(d), Operand::Mem16(addr)) => {
            Ok(vec![0x3A, mod_reg_rm(0, reg8_id(d), 6), addr as u8, (addr >> 8) as u8])
        }
        _ => Err(AsmError { line, message: "unsupported ALU form".into() }),
    }
}

/// The /subfield for `81 /n r/m, imm16` — the reg bits of the mod-reg-rm.
/// ADD=0, OR=1, ADC=2, SBB=3, AND=4, SUB=5, XOR=6, CMP=7.
fn alu_subfield(op_rm_reg: u8) -> u8 {
    match op_rm_reg {
        0x01 => 0, // ADD
        0x09 => 1, // OR
        0x21 => 4, // AND
        0x29 => 5, // SUB
        0x31 => 6, // XOR
        0x39 => 7, // CMP
        _ => 0,
    }
}

fn mod_reg_rm(mod_: u8, reg: u8, rm: u8) -> u8 {
    ((mod_ & 3) << 6) | ((reg & 7) << 3) | (rm & 7)
}

fn short_branch(
    line: u32,
    here: u16,
    op: u8,
    o: &Operand,
    labels: &HashMap<String, u16>,
    consts: &HashMap<String, u16>,
) -> Result<Vec<u8>, AsmError> {
    // Placeholder 0 for a not-yet-known forward label: the rel8 field exists
    // either way, so the length is correct in pass 1. `validate_labels_resolved`
    // runs before pass 2 and rejects any label that never got defined.
    let known = match o {
        Operand::Label(n) => labels.contains_key(n) || consts.contains_key(n),
        _ => true,
    };
    let target = resolve_label_lenient(o, labels, consts);
    let next_ip = here.wrapping_add(2); // short branch is 2 bytes: op + rel8
    let rel = (target as i32 - next_ip as i32) as i16;
    if known && (rel < -128 || rel > 127) {
        return Err(AsmError { line, message: format!("short branch out of range (rel={rel}); use a `near` form (TODO)") });
    }
    Ok(vec![op, rel as u8])
}

fn resolve_label_lenient(
    o: &Operand,
    labels: &HashMap<String, u16>,
    consts: &HashMap<String, u16>,
) -> u16 {
    match o {
        Operand::Label(n) => labels.get(n).copied().or_else(|| consts.get(n).copied()).unwrap_or(0),
        Operand::Imm16(v) => *v,
        _ => 0,
    }
}

/// Collect every label referenced by an item's operands. Used by the
/// validation pass between layout and emission.
fn label_refs(item: &Item) -> Vec<&str> {
    use Instruction as I;
    macro_rules! name {
        ($o:expr) => {
            match $o {
                Operand::Label(n) | Operand::MemLabelOff(n, _) => vec![n.as_str()],
                _ => vec![],
            }
        };
    }
    let (a, b) = match &item.stmt {
        Statement::Directive(_) => (vec![], vec![]),
        Statement::Instruction(i) => match i {
            I::Mov { dst, src } | I::Add { dst, src } | I::Sub { dst, src }
            | I::Cmp { dst, src } | I::And { dst, src } | I::Or { dst, src }
            | I::Xor { dst, src } | I::Lea(dst, src) | I::Shr(dst, src) => {
                (name!(dst), name!(src))
            }
            I::Inc(o) | I::Dec(o) | I::JmpShort(o) | I::JeShort(o) | I::JneShort(o)
            | I::JaShort(o) | I::JaeShort(o) | I::JbShort(o) | I::JbeShort(o)
            | I::JgeShort(o) | I::JlShort(o) | I::Call(o) | I::LoopShort(o) => {
                (name!(o), vec![])
            }
            I::Push(_) | I::Pop(_) | I::Int(_) | I::Ret | I::Hlt | I::Nop => (vec![], vec![]),
        },
    };
    a.into_iter().chain(b).collect()
}

/// After pass 1 has fixed every label's address, reject any reference to a
/// name that was never defined. Runs before emission so a typo'd label fails
/// with its source line instead of silently jumping to 0.
pub(crate) fn validate_labels_resolved(
    item: &Item,
    labels: &HashMap<String, u16>,
    consts: &HashMap<String, u16>,
) -> Result<(), AsmError> {
    for name in label_refs(item) {
        if !labels.contains_key(name) && !consts.contains_key(name) {
            return Err(AsmError {
                line: item.line,
                message: format!("unknown label or constant `{name}`"),
            });
        }
    }
    Ok(())
}
