//! Stand-alone 8086 disassembler.
//!
//! Covers exactly the v1 instruction set implemented by `studio-cpu::step`.
//! Given a segment base (CS or DS) and a byte buffer, produces a `DisasmList`
//! of `addr | bytes | text` lines until the buffer is exhausted or an
//! unrecognised opcode stops the walk. Bytes per instruction come from the
//! same `mod-reg-rm` decoding the encoder uses, so the round-trip property
//! holds for any program that successfully ran in the emulator.
//!
//! This is a *library* — no GUI, no std I/O. The GUI's Disassembler widget
//! (and a future CLI) call into it.

use studio_isa::{Reg16, Reg8};

/// One decoded instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisasmLine {
    /// Linear address of the first byte of the instruction.
    pub addr: u16,
    /// Raw instruction bytes (1..=6).
    pub bytes: Vec<u8>,
    /// Mnemonic in upper case, e.g. `"MOV"`, `"JNZ"`.
    pub mnemonic: String,
    /// Argument text, e.g. `"AX, 5"` or `"[BX+SI+0x10]"`. Empty if none.
    pub args: String,
}

impl DisasmLine {
    /// Format like `0100  B8 05 00   MOV   AX, 5`.
    pub fn format(&self) -> String {
        let bytes = self
            .bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        // Pad bytes to 12 chars (max 6 bytes + 5 spaces).
        let bytes_pad = format!("{:<12}", bytes);
        let args = if self.args.is_empty() {
            String::new()
        } else {
            format!(" {}", self.args)
        };
        format!(
            "{:04X}  {}   {:<6}{}",
            self.addr, bytes_pad, self.mnemonic, args
        )
    }
}

/// Whole disassembly result. `truncated = true` means the walk stopped early
/// (out of bytes or unknown opcode); the caller can show "(end of buffer)".
#[derive(Debug, Clone, Default)]
pub struct DisasmList {
    pub lines: Vec<DisasmLine>,
    pub truncated: bool,
}

/// Decode `bytes` starting at logical address `base` (e.g. `0x0100` for a
/// `.com` image). Stops at the first byte that cannot be decoded or at the
/// end of the buffer.
pub fn disassemble(base: u16, bytes: &[u8]) -> DisasmList {
    let mut out = DisasmList::default();
    let mut ip = base;
    let mut off: usize = 0;
    while off < bytes.len() {
        let start_ip = ip;
        let start_off = off;
        let op = bytes[off];
        let mut consumed: Option<usize> = None;
        let mut line: Option<DisasmLine> = None;

        // 1-byte fixed instructions first.
        match op {
            0x90 => { line = Some(simple(start_ip, &[op], "NOP", "")); consumed = Some(1); }
            0xC3 => { line = Some(simple(start_ip, &[op], "RET", "")); consumed = Some(1); }
            0xF4 => { line = Some(simple(start_ip, &[op], "HLT", "")); consumed = Some(1); }
            0x40..=0x47 => { line = Some(simple(start_ip, &[op], "INC", Reg16::ALL[(op - 0x40) as usize].name())); consumed = Some(1); }
            0x48..=0x4F => { line = Some(simple(start_ip, &[op], "DEC", Reg16::ALL[(op - 0x48) as usize].name())); consumed = Some(1); }
            0x50..=0x57 => { line = Some(simple(start_ip, &[op], "PUSH", Reg16::ALL[(op - 0x50) as usize].name())); consumed = Some(1); }
            0x58..=0x5F => { line = Some(simple(start_ip, &[op], "POP", Reg16::ALL[(op - 0x58) as usize].name())); consumed = Some(1); }
            _ => {}
        }

        // MOV r8, imm8 (B0..=B7)
        if line.is_none() && matches!(op, 0xB0..=0xB7) {
            if let Some(imm) = take_u8(bytes, off + 1) {
                let r = Reg8::from_usize((op - 0xB0) as usize);
                line = Some(simple(
                    start_ip,
                    &[op, imm],
                    "MOV",
                    &format!("{}, {}", r.name(), hex8(imm)),
                ));
                consumed = Some(2);
            }
        }
        // MOV r16, imm16 (B8..=BF)
        if line.is_none() && matches!(op, 0xB8..=0xBF) {
            if let Some(imm) = take_u16(bytes, off + 1) {
                let r = Reg16::ALL[(op - 0xB8) as usize];
                line = Some(simple(
                    start_ip,
                    &[op, (imm & 0xFF) as u8, (imm >> 8) as u8],
                    "MOV",
                    &format!("{}, {}", r.name(), hex16(imm)),
                ));
                consumed = Some(3);
            }
        }
        // ADD/SUB/CMP AX, imm16 (05/2D/3D)
        if line.is_none() {
            let mn = match op {
                0x05 => "ADD",
                0x2D => "SUB",
                0x3D => "CMP",
                _ => "",
            };
            if !mn.is_empty() {
                if let Some(imm) = take_u16(bytes, off + 1) {
                    line = Some(simple(
                        start_ip,
                        &[op, (imm & 0xFF) as u8, (imm >> 8) as u8],
                        mn,
                        &format!("AX, {}", hex16(imm)),
                    ));
                    consumed = Some(3);
                }
            }
        }
        // INT n (CD imm8)
        if line.is_none() && op == 0xCD {
            if let Some(n) = take_u8(bytes, off + 1) {
                line = Some(simple(start_ip, &[op, n], "INT", &hex8(n)));
                consumed = Some(2);
            }
        }
        // JMP rel8 (EB)
        if line.is_none() && op == 0xEB {
            if let Some(rel) = take_i8(bytes, off + 1) {
                let target = start_ip.wrapping_add(2).wrapping_add(rel as u16);
                line = Some(simple(
                    start_ip,
                    &[op, rel as u8],
                    "JMP",
                    &format!("SHORT {}", hex16(target)),
                ));
                consumed = Some(2);
            }
        }
        // JZ/JNZ rel8 (74/75)
        if line.is_none() && (op == 0x74 || op == 0x75) {
            if let Some(rel) = take_i8(bytes, off + 1) {
                let target = start_ip.wrapping_add(2).wrapping_add(rel as u16);
                let mn = if op == 0x74 { "JZ" } else { "JNZ" };
                line = Some(simple(
                    start_ip,
                    &[op, rel as u8],
                    mn,
                    &format!("SHORT {}", hex16(target)),
                ));
                consumed = Some(2);
            }
        }
        // LOOP rel8 (E2)
        if line.is_none() && op == 0xE2 {
            if let Some(rel) = take_i8(bytes, off + 1) {
                let target = start_ip.wrapping_add(2).wrapping_add(rel as u16);
                line = Some(simple(
                    start_ip,
                    &[op, rel as u8],
                    "LOOP",
                    &format!("SHORT {}", hex16(target)),
                ));
                consumed = Some(2);
            }
        }
        // CALL rel16 (E8)
        if line.is_none() && op == 0xE8 {
            if let Some(rel) = take_i16(bytes, off + 1) {
                let target = start_ip.wrapping_add(3).wrapping_add(rel as u16);
                line = Some(simple(
                    start_ip,
                    &[op, (rel & 0xFF) as u8, (rel >> 8) as u8],
                    "CALL",
                    &hex16(target),
                ));
                consumed = Some(3);
            }
        }
        // Mod-reg-rm group.
        if line.is_none() {
            let modrm: ModRm = match ModRm::parse(bytes, off + 1) {
                Some(m) => m,
                None => {
                    out.truncated = true;
                    out.lines.push(DisasmLine {
                        addr: start_ip,
                        bytes: vec![op],
                        mnemonic: "DB".into(),
                        args: format!("0x{:02X}", op),
                    });
                    break;
                }
            };
            match op {
                0x01 => { line = alu_rr(start_ip, op, "ADD", &modrm); consumed = Some(1 + modrm.size); }
                0x29 => { line = alu_rr(start_ip, op, "SUB", &modrm); consumed = Some(1 + modrm.size); }
                0x39 => { line = alu_rr(start_ip, op, "CMP", &modrm); consumed = Some(1 + modrm.size); }
                0x88 => { line = mov_r8(start_ip, op, &modrm, false); consumed = Some(1 + modrm.size); }
                0x8A => { line = mov_r8(start_ip, op, &modrm, true);  consumed = Some(1 + modrm.size); }
                0x89 => { line = mov_r16(start_ip, op, &modrm, false); consumed = Some(1 + modrm.size); }
                0x8B => { line = mov_r16(start_ip, op, &modrm, true);  consumed = Some(1 + modrm.size); }
                0x81 => {
                    // ALU r/m16, imm16 — reg-field selects op.
                    if modrm.mod_ == 0b11 {
                        // Register form: r16 +/-/? imm16
                        let mn = match modrm.reg {
                            0 => "ADD",
                            5 => "SUB",
                            7 => "CMP",
                            _ => "",
                        };
                        if !mn.is_empty() {
                            let need = 1 + modrm.size + 2;
                            if bytes.len() >= off + need {
                                let imm = u16::from_le_bytes([bytes[off + 1 + modrm.size], bytes[off + 2 + modrm.size]]);
                                let r = Reg16::ALL[modrm.rm as usize];
                                let whole = bytes.get(off..off + need).map(|s| s.to_vec()).unwrap_or_default();
                                line = Some(simple(start_ip, &whole, mn, &format!("{}, {}", r.name(), hex16(imm))));
                                consumed = Some(need);
                            }
                        }
                        // Unknown sub-op (e.g. ADC/SBB/OR/AND/XOR/...) → falls through to "DB".
                    }
                    // Memory form not used in v1.
                }
                _ => {}
            }
        }

        match (line.take(), consumed) {
            (Some(l), Some(n)) => {
                out.lines.push(l);
                ip = ip.wrapping_add(n as u16);
                off += n;
            }
            _ => {
                out.truncated = true;
                out.lines.push(DisasmLine {
                    addr: start_ip,
                    bytes: bytes.get(start_off..).map(|s| s.to_vec()).unwrap_or_default(),
                    mnemonic: "DB".into(),
                    args: format!("0x{:02X}", op),
                });
                break;
            }
        }
    }
    out
}

// ─────────────── helpers ───────────────

fn simple(addr: u16, bytes: &[u8], mn: &str, args: &str) -> DisasmLine {
    DisasmLine {
        addr,
        bytes: bytes.to_vec(),
        mnemonic: mn.into(),
        args: args.into(),
    }
}

fn take_u8(b: &[u8], i: usize) -> Option<u8> {
    b.get(i).copied()
}
fn take_u16(b: &[u8], i: usize) -> Option<u16> {
    if b.len() < i + 2 {
        None
    } else {
        Some(u16::from_le_bytes([b[i], b[i + 1]]))
    }
}
fn take_i8(b: &[u8], i: usize) -> Option<i8> {
    b.get(i).copied().map(|x| x as i8)
}
fn take_i16(b: &[u8], i: usize) -> Option<i16> {
    take_u16(b, i).map(|x| x as i16)
}
fn hex8(v: u8) -> String { format!("{:02X}h", v) }
fn hex16(v: u16) -> String { format!("{:04X}h", v) }

/// Decoded ModR/M byte plus optional SIB/disp16/disp8 tail.
struct ModRm {
    mod_: u8,
    reg: u8,
    rm: u8,
    /// Total bytes including the mod/rm byte itself (1, 3, or 5).
    size: usize,
    /// Operand text for the r/m side (memory text, or "REG3" placeholder when mod=11).
    #[allow(dead_code)]
    rm_text: String,
    /// Operand text for the reg side.
    reg_text_r8: String,
    reg_text_r16: String,
    /// When mod=11, the resolved 8/16-bit register name for the rm field.
    rm_text_r8: String,
    rm_text_r16: String,
}

impl ModRm {
    fn parse(b: &[u8], off: usize) -> Option<Self> {
        let raw = *b.get(off)?;
        let mod_ = (raw >> 6) & 0b11;
        let reg = (raw >> 3) & 0b111;
        let rm = raw & 0b111;
        let mut size = 1usize;
        let rm_text = match (mod_, rm) {
            (0b00, 0b000) => "[BX+SI]".into(),
            (0b00, 0b001) => "[BX+DI]".into(),
            (0b00, 0b010) => "[BP+SI]".into(),
            (0b00, 0b011) => "[BP+DI]".into(),
            (0b00, 0b100) => "[SI]".into(),
            (0b00, 0b101) => "[DI]".into(),
            (0b00, 0b110) => {
                // direct [disp16]
                if b.len() < off + 3 { return None; }
                let lo = b[off + 1];
                let hi = b[off + 2];
                let disp = u16::from_le_bytes([lo, hi]);
                size += 2;
                format!("[{}]", hex16(disp))
            }
            (0b00, 0b111) => "[BX]".into(),
            (0b01, r) => {
                if b.len() < off + 2 { return None; }
                let d = b[off + 1] as i8;
                size += 1;
                let base = match r {
                    0b000 => "BX+SI", 0b001 => "BX+DI", 0b010 => "BP+SI", 0b011 => "BP+DI",
                    0b100 => "SI",     0b101 => "DI",     0b110 => "BP",     0b111 => "BX",
                    _ => unreachable!(),
                };
                if d == 0 { format!("[{}]", base) }
                else if d > 0 { format!("[{}+{}]", base, d) }
                else { format!("[{}{}]", base, d) }
            }
            (0b10, r) => {
                if b.len() < off + 3 { return None; }
                let lo = b[off + 1];
                let hi = b[off + 2];
                let d = i16::from_le_bytes([lo, hi]);
                size += 2;
                let base = match r {
                    0b000 => "BX+SI", 0b001 => "BX+DI", 0b010 => "BP+SI", 0b011 => "BP+DI",
                    0b100 => "SI",     0b101 => "DI",     0b110 => "BP",     0b111 => "BX",
                    _ => unreachable!(),
                };
                if d == 0 { format!("[{}]", base) }
                else if d > 0 { format!("[{}+{}]", base, d) }
                else { format!("[{}{}]", base, d) }
            }
            (0b11, r) => {
                // Register-direct — no memory operand; caller picks 8/16-bit name.
                size += 0;
                format!("REG{}", r)
            }
            _ => unreachable!(),
        };
        let reg_text_r8 = Reg8::from_usize(reg as usize).name().to_string();
        let reg_text_r16 = Reg16::ALL[reg as usize].name().to_string();
        let rm_text_r8 = Reg8::from_usize(rm as usize).name().to_string();
        let rm_text_r16 = Reg16::ALL[rm as usize].name().to_string();
        Some(Self {
            mod_, reg, rm, size, rm_text, reg_text_r8, reg_text_r16, rm_text_r8, rm_text_r16,
        })
    }
}

fn alu_rr(addr: u16, op: u8, mn: &str, m: &ModRm) -> Option<DisasmLine> {
    if m.mod_ != 0b11 { return None; }
    let bytes = vec![op, m_as_byte(m)];
    Some(simple(addr, &bytes, mn, &format!("{}, {}", m.reg_text_r16, m.rm_text_r16)))
}

fn mov_r8(addr: u16, op: u8, m: &ModRm, dir_to_reg: bool) -> Option<DisasmLine> {
    if m.mod_ != 0b11 { return None; }
    let bytes = vec![op, m_as_byte(m)];
    let (dst, src) = if dir_to_reg {
        (m.reg_text_r8.clone(), m.rm_text_r8.clone())
    } else {
        (m.rm_text_r8.clone(), m.reg_text_r8.clone())
    };
    Some(simple(addr, &bytes, "MOV", &format!("{}, {}", dst, src)))
}

fn mov_r16(addr: u16, op: u8, m: &ModRm, dir_to_reg: bool) -> Option<DisasmLine> {
    if m.mod_ != 0b11 { return None; }
    let bytes = vec![op, m_as_byte(m)];
    let (dst, src) = if dir_to_reg {
        (m.reg_text_r16.clone(), m.rm_text_r16.clone())
    } else {
        (m.rm_text_r16.clone(), m.reg_text_r16.clone())
    };
    Some(simple(addr, &bytes, "MOV", &format!("{}, {}", dst, src)))
}

fn m_as_byte(m: &ModRm) -> u8 { (m.mod_ << 6) | (m.reg << 3) | m.rm }

#[cfg(test)]
mod tests {
    use super::*;

    fn fmt(lines: &[DisasmLine]) -> String {
        lines.iter().map(|l| l.format()).collect::<Vec<_>>().join("\n")
    }

    #[test]
    fn mov_ax_imm16() {
        let list = disassemble(0x0100, &[0xB8, 0x05, 0x00]);
        assert_eq!(list.lines.len(), 1);
        assert_eq!(list.lines[0].mnemonic, "MOV");
        assert_eq!(list.lines[0].args, "AX, 0005h");
        assert_eq!(list.lines[0].addr, 0x0100);
    }

    #[test]
    fn mov_al_imm8() {
        let list = disassemble(0x0100, &[0xB0, 0x2A]);
        assert_eq!(list.lines[0].mnemonic, "MOV");
        assert_eq!(list.lines[0].args, "AL, 2Ah");
    }

    #[test]
    fn int_and_ret_and_hlt() {
        let list = disassemble(0x0100, &[0xCD, 0x21, 0xC3, 0xF4]);
        assert_eq!(list.lines[0].mnemonic, "INT");
        assert_eq!(list.lines[0].args, "21h");
        assert_eq!(list.lines[1].mnemonic, "RET");
        assert_eq!(list.lines[2].mnemonic, "HLT");
    }

    #[test]
    fn jnz_short() {
        // JNZ +2 → target = (0102) + 2 = 0104
        let list = disassemble(0x0100, &[0x75, 0x02]);
        assert_eq!(list.lines[0].mnemonic, "JNZ");
        assert_eq!(list.lines[0].args, "SHORT 0104h");
    }

    #[test]
    fn loop_short() {
        let list = disassemble(0x0200, &[0xE2, 0xFE]); // -2 → 0200
        assert_eq!(list.lines[0].mnemonic, "LOOP");
        assert_eq!(list.lines[0].args, "SHORT 0200h");
    }

    #[test]
    fn call_rel16() {
        let list = disassemble(0x0100, &[0xE8, 0x00, 0x01]); // +256 → 0203
        assert_eq!(list.lines[0].mnemonic, "CALL");
        assert_eq!(list.lines[0].args, "0203h");
    }

    #[test]
    fn push_pop_inc_dec() {
        let list = disassemble(0x0100, &[
            0x50,       // PUSH AX
            0x58,       // POP  AX
            0x40,       // INC  AX
            0x48,       // DEC  AX
        ]);
        assert_eq!(list.lines[0].mnemonic, "PUSH");
        assert_eq!(list.lines[0].args, "AX");
        assert_eq!(list.lines[1].mnemonic, "POP");
        assert_eq!(list.lines[2].mnemonic, "INC");
        assert_eq!(list.lines[3].mnemonic, "DEC");
    }

    #[test]
    fn add_sub_cmp_ax_imm16() {
        let list = disassemble(0x0100, &[
            0x05, 0x01, 0x00, // ADD AX, 1
            0x2D, 0x02, 0x00, // SUB AX, 2
            0x3D, 0x03, 0x00, // CMP AX, 3
        ]);
        assert_eq!(list.lines[0].args, "AX, 0001h");
        assert_eq!(list.lines[1].args, "AX, 0002h");
        assert_eq!(list.lines[2].args, "AX, 0003h");
    }

    #[test]
    fn add_reg_reg() {
        // ADD AX, CX  → 01 C1
        let list = disassemble(0x0100, &[0x01, 0xC1]);
        assert_eq!(list.lines[0].mnemonic, "ADD");
        assert_eq!(list.lines[0].args, "AX, CX");
    }

    #[test]
    fn mov_reg_reg_r16() {
        // MOV BX, CX → 89 CB (mov r/m ← r): r/m=dst(BX), reg=src(CX)
        // MOV CX, BX → 8B CB (mov r ← r/m): r=dst(CX), r/m=src(BX)
        let list = disassemble(0x0100, &[0x89, 0xCB]);
        assert_eq!(list.lines[0].mnemonic, "MOV");
        assert_eq!(list.lines[0].args, "BX, CX");
        let list = disassemble(0x0100, &[0x8B, 0xCB]);
        assert_eq!(list.lines[0].args, "CX, BX");
    }

    #[test]
    fn add_imm_to_reg() {
        // ADD CX, 0x10  → 81 C1 10 00
        let list = disassemble(0x0100, &[0x81, 0xC1, 0x10, 0x00]);
        assert_eq!(list.lines[0].mnemonic, "ADD");
        assert_eq!(list.lines[0].args, "CX, 0010h");
    }

    #[test]
    fn sub_imm_to_reg() {
        // SUB DX, 0x20 → 81 EA 20 00
        let list = disassemble(0x0100, &[0x81, 0xEA, 0x20, 0x00]);
        assert_eq!(list.lines[0].mnemonic, "SUB");
        assert_eq!(list.lines[0].args, "DX, 0020h");
    }

    #[test]
    fn unknown_opcode_stops() {
        // 0xFF is not in v1
        let list = disassemble(0x0100, &[0xB8, 0x01, 0x00, 0xFF]);
        assert!(list.truncated);
        assert_eq!(list.lines.len(), 2);
        assert_eq!(list.lines[1].mnemonic, "DB");
    }

    #[test]
    fn format_includes_addr_and_bytes() {
        let list = disassemble(0x0100, &[0xB8, 0x05, 0x00]);
        let s = fmt(&list.lines);
        assert!(s.contains("0100"), "got: {s}");
        assert!(s.contains("B8 05 00"), "got: {s}");
        assert!(s.contains("MOV"), "got: {s}");
        assert!(s.contains("AX"), "got: {s}");
    }
}
