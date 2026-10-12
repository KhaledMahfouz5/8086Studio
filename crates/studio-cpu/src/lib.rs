//! `studio-cpu` — functional 8086 emulator.
//!
//! v1 supports the mnemonics in `architecture.md § 1`. Flag updates
//! follow the Intel 8086 datasheet (OF/SF/ZF/AF/PF/CF set per result).
//! Per the unix covenant: no_std-friendly, no `print!`, no file I/O, no
//! global state. INT dispatch is a callback the host registers; we never
//! touch the outside world directly.

#![forbid(unsafe_code)]

use studio_isa::{Flags, Reg8, Reg16};

/// What `step()` produced.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StepResult {
    Advanced,
    Halted,
    Interrupt(u8),
    StackUnderflow,
    InvalidOpcode(u8),
}

/// Where a modrm `/r/m` field points: a register, a memory address, or
/// (unsupported in v1) nothing.
#[derive(Debug, Clone, Copy)]
enum RmTarget {
    Reg(usize),
    Mem(u16, u16),
    None,
}

/// A function the host registers to handle `INT n` instructions.
pub type IntHandler = Box<dyn FnMut(&mut Cpu8086, u8) -> Result<(), StepResult>>;

pub struct Cpu8086 {
    /// Index by `Reg16 as usize`: AX, CX, DX, BX, SP, BP, SI, DI.
    pub regs: [u16; 8],
    /// Index 0..=3: ES, CS, SS, DS.
    pub segs: [u16; 4],
    pub ip: u16,
    pub flags: Flags,
    /// 1 MiB of flat memory, indexed by `seg:off` per Intel segmentation.
    pub memory: Box<[u8; 0x100000]>,
    pub halted: bool,
    pub last_instr_addr: Option<u16>,
    pub int_handler: Option<IntHandler>,
}

impl Cpu8086 {
    pub fn new() -> Self {
        Self {
            regs: [0u16; 8],
            segs: [0u16; 4],
            ip: 0,
            flags: Flags::empty(),
            memory: Box::new([0u8; 0x100000]),
            halted: false,
            last_instr_addr: None,
            int_handler: None,
        }
    }

    pub fn reg(&self, r: Reg16) -> u16 {
        self.regs[r as usize]
    }

    pub fn set_reg(&mut self, r: Reg16, v: u16) {
        self.regs[r as usize] = v;
    }

    /// `seg:off` linear address — Intel segmentation: `(seg << 4) + off`,
    /// 20-bit, wraps mod 1 MiB.
    pub fn lin(seg: u16, off: u16) -> u32 {
        (((seg as u32) << 4) + off as u32) & 0x000F_FFFF
    }

    pub fn read8(&self, seg: u16, off: u16) -> u8 {
        let a = Self::lin(seg, off) as usize;
        self.memory[a]
    }

    pub fn write8(&mut self, seg: u16, off: u16, v: u8) {
        let a = Self::lin(seg, off) as usize;
        self.memory[a] = v;
    }

    pub fn read16(&self, seg: u16, off: u16) -> u16 {
        u16::from_le_bytes([self.read8(seg, off), self.read8(seg, off.wrapping_add(1))])
    }

    pub fn write16(&mut self, seg: u16, off: u16, v: u16) {
        let lo = v as u8;
        let hi = (v >> 8) as u8;
        self.write8(seg, off, lo);
        self.write8(seg, off.wrapping_add(1), hi);
    }

    /// Load `bytes` into memory starting at `cs:ip`. Advances `IP` past the
    /// program. Mirrors how DOS loads a `.com`: `SS` is set to `CS` and `SP`
    /// to the top of the 64 KiB segment, so `PUSH`/`POP` work out of the box.
    pub fn load(&mut self, cs: u16, ip: u16, bytes: &[u8]) {
        for (i, b) in bytes.iter().enumerate() {
            self.write8(cs, ip.wrapping_add(i as u16), *b);
        }
        self.segs[1] = cs;
        self.segs[2] = cs; // SS = CS, as DOS does for .com
        self.regs[Reg16::SP as usize] = 0xFFFE;
        self.ip = ip;
        self.halted = false;
    }

    /// Decode and execute one instruction at `CS:IP`. Advances `IP` past it.
    pub fn step(&mut self) -> StepResult {
        let cs = self.segs[1];
        let ip = self.ip;
        let op = self.read8(cs, ip);
        self.last_instr_addr = Some(ip);

        match op {
            // MOV reg16, imm16 → B8+rd, lo, hi
            0xB8..=0xBF => {
                let r = (op - 0xB8) as usize;
                let lo = self.read8(cs, ip.wrapping_add(1));
                let hi = self.read8(cs, ip.wrapping_add(2));
                self.regs[r] = u16::from_le_bytes([lo, hi]);
                self.ip = ip.wrapping_add(3);
                StepResult::Advanced
            }
            // MOV reg8, imm8 → B0+rb, imm8
            0xB0..=0xB7 => {
                let r = (op - 0xB0) as usize;
                let imm = self.read8(cs, ip.wrapping_add(1));
                self.write_reg8(Reg8::from_usize(r), imm);
                self.ip = ip.wrapping_add(2);
                StepResult::Advanced
            }
            // ADD/SUB/CMP reg16, reg16 → 01/29/39 + mod_reg_rm(11, s, d)
            0x01 => { self.exec_alu_reg(0x01, ip, |a, b| a.wrapping_add(b), false); StepResult::Advanced }
            0x29 => { self.exec_alu_reg(0x29, ip, |a, b| a.wrapping_sub(b), false); StepResult::Advanced }
            0x39 => { self.exec_alu_reg(0x39, ip, |a, b| a.wrapping_sub(b), true); StepResult::Advanced }
            // MOV r/m8, r8 → 88 /r   |   MOV r8, r/m8 → 8A /r
            0x88 => { self.exec_mov8(ip, false); StepResult::Advanced }
            0x8A => { self.exec_mov8(ip, true); StepResult::Advanced }
            // MOV r/m16, r16 → 89 /r   (op=0x89 mod_reg_rm(11, src, dst))
            0x89 => { self.exec_mov_reg(ip, false); StepResult::Advanced }
            // MOV r16, r/m16 → 8B /r   (op=0x8B)
            0x8B => { self.exec_mov_reg(ip, true); StepResult::Advanced }
            // ADD AX, imm16 → 05, lo, hi
            0x05 => {
                let lo = self.read8(cs, ip.wrapping_add(1));
                let hi = self.read8(cs, ip.wrapping_add(2));
                let imm = u16::from_le_bytes([lo, hi]);
                let a = self.regs[0];
                let r = a.wrapping_add(imm);
                self.regs[0] = r;
                self.set_alu_flags16(a, imm, r, /*sub=*/false);
                self.ip = ip.wrapping_add(3);
                StepResult::Advanced
            }
            // SUB AX, imm16 → 2D, lo, hi
            0x2D => {
                let lo = self.read8(cs, ip.wrapping_add(1));
                let hi = self.read8(cs, ip.wrapping_add(2));
                let imm = u16::from_le_bytes([lo, hi]);
                let a = self.regs[0];
                let r = a.wrapping_sub(imm);
                self.regs[0] = r;
                self.set_alu_flags16(a, imm, r, /*sub=*/true);
                self.ip = ip.wrapping_add(3);
                StepResult::Advanced
            }
            // CMP AX, imm16 → 3D, lo, hi
            0x3D => {
                let lo = self.read8(cs, ip.wrapping_add(1));
                let hi = self.read8(cs, ip.wrapping_add(2));
                let imm = u16::from_le_bytes([lo, hi]);
                let a = self.regs[0];
                let r = a.wrapping_sub(imm);
                self.set_alu_flags16(a, imm, r, /*sub=*/true);
                self.ip = ip.wrapping_add(3);
                StepResult::Advanced
            }
            // ADD/SUB/CMP r/m16, imm16 → 81 /0|5|7, mod_reg_rm, imm16
            0x81 => { self.exec_alu_imm(ip, |a, b| a.wrapping_add(b), false); StepResult::Advanced }
            // INC/DEC reg16 → 40+rd / 48+rd
            0x40..=0x47 => {
                let r = (op - 0x40) as usize;
                let v = self.regs[r].wrapping_add(1);
                self.regs[r] = v;
                self.set_inc_dec_flags16(v, false);
                self.ip = ip.wrapping_add(1);
                StepResult::Advanced
            }
            0x48..=0x4F => {
                let r = (op - 0x48) as usize;
                let v = self.regs[r].wrapping_sub(1);
                self.regs[r] = v;
                self.set_inc_dec_flags16(v, true);
                self.ip = ip.wrapping_add(1);
                StepResult::Advanced
            }
            // PUSH reg16 → 50+rd, POP reg16 → 58+rd
            0x50..=0x57 => {
                let r = (op - 0x50) as usize;
                let v = self.regs[r];
                if !self.push16(v) { return StepResult::StackUnderflow; }
                self.ip = ip.wrapping_add(1);
                StepResult::Advanced
            }
            0x58..=0x5F => {
                let r = (op - 0x58) as usize;
                let v = match self.pop16() { Some(v) => v, None => return StepResult::StackUnderflow };
                self.regs[r] = v;
                self.ip = ip.wrapping_add(1);
                StepResult::Advanced
            }
            // JMP short rel8 → EB, rel8
            0xEB => {
                let rel = self.read8(cs, ip.wrapping_add(1)) as i8 as i16;
                self.ip = ip.wrapping_add(2).wrapping_add(rel as u16);
                StepResult::Advanced
            }
            // JE/JZ short → 74, rel8
            0x74 => {
                let rel = self.read8(cs, ip.wrapping_add(1)) as i8 as i16;
                let next = ip.wrapping_add(2);
                self.ip = if self.flags.contains(Flags::ZF) { next.wrapping_add(rel as u16) } else { next };
                StepResult::Advanced
            }
            // JNE/JNZ short → 75, rel8
            0x75 => {
                let rel = self.read8(cs, ip.wrapping_add(1)) as i8 as i16;
                let next = ip.wrapping_add(2);
                self.ip = if !self.flags.contains(Flags::ZF) { next.wrapping_add(rel as u16) } else { next };
                StepResult::Advanced
            }
            // LOOP short rel8 → E2, rel8: dec CX, jump if CX != 0
            0xE2 => {
                let rel = self.read8(cs, ip.wrapping_add(1)) as i8 as i16;
                let next = ip.wrapping_add(2);
                self.regs[Reg16::CX as usize] = self.regs[Reg16::CX as usize].wrapping_sub(1);
                let cx = self.regs[Reg16::CX as usize];
                self.ip = if cx != 0 { next.wrapping_add(rel as u16) } else { next };
                StepResult::Advanced
            }
            // CALL rel16 → E8, lo, hi  (near, intra-segment)
            0xE8 => {
                let lo = self.read8(cs, ip.wrapping_add(1));
                let hi = self.read8(cs, ip.wrapping_add(2));
                let rel = i16::from_le_bytes([lo, hi]);
                let next = ip.wrapping_add(3);
                let target = next.wrapping_add(rel as u16);
                if !self.push16(next) { return StepResult::StackUnderflow; }
                self.ip = target;
                StepResult::Advanced
            }
            // INT n → CD, n
            0xCD => {
                let n = self.read8(cs, ip.wrapping_add(1));
                self.ip = ip.wrapping_add(2);
                let mut handler = self.int_handler.take();
                let result = if let Some(h) = handler.as_mut() {
                    h(self, n)
                } else {
                    Err(StepResult::Interrupt(n))
                };
                self.int_handler = handler;
                match result {
                    Ok(()) => StepResult::Advanced,
                    Err(StepResult::Interrupt(_)) => StepResult::Interrupt(n),
                    Err(other) => other,
                }
            }
            // RET (near) → C3
            0xC3 => {
                let ret_ip = match self.pop16() { Some(v) => v, None => return StepResult::StackUnderflow };
                self.ip = ret_ip;
                self.halted = true;
                StepResult::Halted
            }
            // HLT → F4
            0xF4 => {
                self.halted = true;
                self.ip = ip.wrapping_add(1);
                StepResult::Halted
            }
            // NOP → 90
            0x90 => {
                self.ip = ip.wrapping_add(1);
                StepResult::Advanced
            }
            other => {
                self.ip = ip.wrapping_add(1);
                StepResult::InvalidOpcode(other)
            }
        }
    }

    fn exec_alu_reg(&mut self, op: u8, ip: u16, f: fn(u16, u16) -> u16, sub: bool) {
        let cs = self.segs[1];
        let modrm = self.read8(cs, ip.wrapping_add(1));
        let reg = ((modrm >> 3) & 7) as usize;
        let rm = (modrm & 7) as usize;
        // Only mod=11 (register-register) is supported in v1.
        let (a, b) = (self.regs[rm], self.regs[reg]);
        let r = f(a, b);
        if op == 0x01 || op == 0x29 {
            self.regs[rm] = r;
        }
        // CMP doesn't write.
        let _ = sub;
        self.set_alu_flags16(a, b, r, op == 0x29 || op == 0x39);
        self.ip = ip.wrapping_add(2);
    }

    fn exec_mov8(&mut self, ip: u16, dir_to_reg: bool) {
        let cs = self.segs[1];
        let modrm = self.read8(cs, ip.wrapping_add(1));
        let reg = ((modrm >> 3) & 7) as u8;
        let mode = (modrm >> 6) & 3;
        let rm = modrm & 7;

        // Resolve the r/m side to either a register byte or a memory byte.
        if mode == 3 {
            // reg-to-reg 8-bit move
            let rmr = Reg8::from_usize(rm as usize);
            let reg = Reg8::from_usize(reg as usize);
            let (src, dst) = if dir_to_reg { (rmr, reg) } else { (reg, rmr) };
            let v = self.read_reg8(src);
            self.write_reg8(dst, v);
            self.ip = ip.wrapping_add(2);
        } else {
            // memory form: only [disp16] (rm=6) and [BX/SI/DI] (rm=7/4/5) are
            // emitted by our assembler; [BP]-relative and SIB are out of v1.
            let seg = self.segs[3]; // DS
            let off = match rm {
                6 => {
                    let lo = self.read8(cs, ip.wrapping_add(2));
                    let hi = self.read8(cs, ip.wrapping_add(3));
                    u16::from_le_bytes([lo, hi])
                }
                7 => self.regs[Reg16::BX as usize],
                4 => self.regs[Reg16::SI as usize],
                5 => self.regs[Reg16::DI as usize],
                _ => return { self.ip = ip.wrapping_add(2); },
            };
            let reg = Reg8::from_usize(reg as usize);
            if dir_to_reg {
                // MOV r8, m8
                let v = self.read8(seg, off);
                self.write_reg8(reg, v);
            } else {
                // MOV m8, r8
                let v = self.read_reg8(reg);
                self.write8(seg, off, v);
            }
            self.ip = ip.wrapping_add(if rm == 6 { 4 } else { 2 });
        }
    }

    fn read_reg8(&self, r: Reg8) -> u8 {
        let w = self.regs[reg16_of(r) as usize];
        match r {
            Reg8::AL | Reg8::CL | Reg8::DL | Reg8::BL => (w & 0xFF) as u8,
            Reg8::AH | Reg8::CH | Reg8::DH | Reg8::BH => (w >> 8) as u8,
        }
    }

    fn write_reg8(&mut self, r: Reg8, v: u8) {
        let w = self.regs[reg16_of(r) as usize];
        let new = match r {
            Reg8::AL | Reg8::CL | Reg8::DL | Reg8::BL => (w & 0xFF00) | (v as u16),
            Reg8::AH | Reg8::CH | Reg8::DH | Reg8::BH => ((v as u16) << 8) | (w & 0x00FF),
        };
        self.regs[reg16_of(r) as usize] = new;
    }

    fn exec_mov_reg(&mut self, ip: u16, dir_to_reg: bool) {
        let cs = self.segs[1];
        let modrm = self.read8(cs, ip.wrapping_add(1));
        let reg = ((modrm >> 3) & 7) as usize;
        let len = rm_instruction_len(modrm);
        let target = self.resolve_rm(modrm, ip);
        let reg_val = self.regs[reg];
        match target {
            RmTarget::Reg(rm) => {
                if dir_to_reg { self.regs[reg] = self.regs[rm]; }
                else { self.regs[rm] = reg_val; }
            }
            RmTarget::Mem(seg, off) => {
                if dir_to_reg { self.regs[reg] = self.read16(seg, off); }
                else { self.write16(seg, off, reg_val); }
            }
            RmTarget::None => {}
        }
        self.ip = ip.wrapping_add(len);
    }

    /// Decode the /r/m half of a modrm byte into either a register index or a
    /// (segment, offset) memory address. v1 supports mod=11 (reg), and
    /// mod=00 rm∈{BX,SI,DI,[disp16]}. Anything else returns `RmTarget::None`.
    fn resolve_rm(&self, modrm: u8, ip: u16) -> RmTarget {
        let mode = (modrm >> 6) & 3;
        let rm = (modrm & 7) as usize;
        let cs = self.segs[1];
        if mode == 3 {
            return RmTarget::Reg(rm);
        }
        let seg = self.segs[3]; // DS
        match rm {
            7 => RmTarget::Mem(seg, self.regs[Reg16::BX as usize]), // [BX]
            4 => RmTarget::Mem(seg, self.regs[Reg16::SI as usize]), // [SI]
            5 => RmTarget::Mem(seg, self.regs[Reg16::DI as usize]), // [DI]
            6 => {                                                   // [disp16]
                let lo = self.read8(cs, ip.wrapping_add(2));
                let hi = self.read8(cs, ip.wrapping_add(3));
                RmTarget::Mem(seg, u16::from_le_bytes([lo, hi]))
            }
            _ => RmTarget::None,
        }
    }

    fn exec_alu_imm(&mut self, ip: u16, f: fn(u16, u16) -> u16, sub: bool) {
        let cs = self.segs[1];
        let modrm = self.read8(cs, ip.wrapping_add(1));
        let op_ext = ((modrm >> 3) & 7) as usize;
        let rm = (modrm & 7) as usize;
        let lo = self.read8(cs, ip.wrapping_add(2));
        let hi = self.read8(cs, ip.wrapping_add(3));
        let imm = u16::from_le_bytes([lo, hi]);
        let a = self.regs[rm];
        let r = f(a, imm);
        if op_ext == 0 || op_ext == 5 { // ADD or SUB
            self.regs[rm] = r;
        }
        self.set_alu_flags16(a, imm, r, op_ext == 5 || op_ext == 7 || sub);
        self.ip = ip.wrapping_add(4);
    }

    fn push16(&mut self, v: u16) -> bool {
        let sp = self.regs[Reg16::SP as usize];
        if sp == 0 {
            return false;
        }
        let new_sp = sp.wrapping_sub(2);
        self.write16(self.segs[2], new_sp, v);
        self.regs[Reg16::SP as usize] = new_sp;
        true
    }

    fn pop16(&mut self) -> Option<u16> {
        let sp = self.regs[Reg16::SP as usize];
        let ss = self.segs[2];
        let v = self.read16(ss, sp);
        let new_sp = sp.wrapping_add(2);
        self.regs[Reg16::SP as usize] = new_sp;
        Some(v)
    }

    /// Update OF/SF/ZF/AF/PF/CF for a 16-bit ALU operation. Preserve DF/IF/TF.
    fn set_alu_flags16(&mut self, a: u16, b: u16, r: u16, sub: bool) {
        let cf;
        let mut f = self.flags & (Flags::DF | Flags::IF | Flags::TF);
        if sub {
            // Borrow for subtraction: CF = (a < b) when viewed unsigned.
            cf = (a as u32) < (b as u32);
        } else {
            cf = (r as u32) < (a as u32);
        }
        if cf { f |= Flags::CF; }
        if r == 0 { f |= Flags::ZF; }
        if (r >> 15) & 1 == 1 { f |= Flags::SF; }
        // AF: carry/borrow out of bit 3.
        let af_in = ((a ^ b) ^ r) & 0x0008 != 0;
        if af_in { f |= Flags::AF; }
        // OF: signed overflow.
        let sa = (a >> 15) & 1;
        let sb = (b >> 15) & 1;
        let sr = (r >> 15) & 1;
        let of = if sub { sa != sb && sr != sa } else { sa == sb && sr != sa };
        if of { f |= Flags::OF; }
        // PF: parity of low byte (even = set).
        if (r & 0xFF).count_ones() % 2 == 0 { f |= Flags::PF; }
        self.flags = f;
    }

    /// INC/DEC flags per Intel: updates OF/SF/ZF/AF/PF, leaves CF untouched.
    fn set_inc_dec_flags16(&mut self, r: u16, dec: bool) {
        let mut f = self.flags & (Flags::CF | Flags::DF | Flags::IF | Flags::TF); // preserve CF + others
        if r == 0 { f |= Flags::ZF; }
        if (r >> 15) & 1 == 1 { f |= Flags::SF; }
        // AF: bit-3 carry when incrementing / borrow when decrementing.
        let lo = r & 0xF;
        if dec {
            if lo == 0xF { f |= Flags::AF; }
            // OF: 0x8000 → 0x7FFF underflow
            if r == 0x7FFF { f |= Flags::OF; }
        } else {
            if lo == 0x0 { f |= Flags::AF; }
            // OF: 0x7FFF → 0x8000 overflow
            if r == 0x8000 { f |= Flags::OF; }
        }
        if (r & 0xFF).count_ones() % 2 == 0 { f |= Flags::PF; }
        self.flags = f;
    }
}

fn reg16_of(r: Reg8) -> Reg16 {
    match r {
        Reg8::AL | Reg8::AH => Reg16::AX,
        Reg8::CL | Reg8::CH => Reg16::CX,
        Reg8::DL | Reg8::DH => Reg16::DX,
        Reg8::BL | Reg8::BH => Reg16::BX,
    }
}

/// Length of an `opcode /r` instruction: 2 bytes for reg/[BX]/[SI]/[DI],
/// 4 bytes when a 16-bit displacement is present (mod=00, rm=110).
fn rm_instruction_len(modrm: u8) -> u16 {
    let mode = (modrm >> 6) & 3;
    let rm = modrm & 7;
    if mode == 0 && rm == 6 { 4 } else { 2 }
}

impl Default for Cpu8086 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segmentation_is_seg_shift_4_plus_off() {
        assert_eq!(Cpu8086::lin(0xFFFF, 0x0000), 0x000F_FFF0);
        assert_eq!(Cpu8086::lin(0x0000, 0x0100), 0x00100);
    }

    #[test]
    fn mov_ax_imm_updates_ax_and_ip() {
        let mut cpu = Cpu8086::new();
        cpu.load(0, 0x0100, &[0xB8, 0x05, 0x00]);
        assert_eq!(cpu.step(), StepResult::Advanced);
        assert_eq!(cpu.reg(Reg16::AX), 5);
        assert_eq!(cpu.ip, 0x0103);
    }

    #[test]
    fn int_21h_fires_handler() {
        let mut cpu = Cpu8086::new();
        cpu.int_handler = Some(Box::new(|_cpu, n| {
            assert_eq!(n, 0x21);
            Ok(())
        }));
        cpu.load(0, 0x0100, &[0xCD, 0x21]);
        assert_eq!(cpu.step(), StepResult::Advanced);
        assert_eq!(cpu.ip, 0x0102);
    }

    #[test]
    fn int_without_handler_returns_interrupt() {
        let mut cpu = Cpu8086::new();
        cpu.load(0, 0x0100, &[0xCD, 0x21]);
        assert_eq!(cpu.step(), StepResult::Interrupt(0x21));
    }

    #[test]
    fn ret_pops_and_jumps() {
        let mut cpu = Cpu8086::new();
        // SP=FFFE, push a return address, RET.
        cpu.set_reg(Reg16::SP, 0xFFFE);
        cpu.write16(0, 0xFFFE, 0x0200);
        cpu.load(0, 0x0100, &[0xC3]);
        assert_eq!(cpu.step(), StepResult::Halted);
        assert_eq!(cpu.ip, 0x0200);
        assert_eq!(cpu.reg(Reg16::SP), 0);
    }

    #[test]
    fn hlt_halts() {
        let mut cpu = Cpu8086::new();
        cpu.load(0, 0x0100, &[0xF4]);
        assert_eq!(cpu.step(), StepResult::Halted);
    }

    #[test]
    fn invalid_opcode_returns_invalid_opcode() {
        let mut cpu = Cpu8086::new();
        cpu.load(0, 0x0100, &[0xFF]);
        assert_eq!(cpu.step(), StepResult::InvalidOpcode(0xFF));
    }

    #[test]
    fn add_ax_imm_updates_ax_and_sets_zf() {
        let mut cpu = Cpu8086::new();
        cpu.load(0, 0x0100, &[0x05, 0x00, 0x00]);
        cpu.set_reg(Reg16::AX, 0);
        assert_eq!(cpu.step(), StepResult::Advanced);
        assert_eq!(cpu.reg(Reg16::AX), 0);
        assert!(cpu.flags.contains(Flags::ZF));
    }

    #[test]
    fn add_sets_cf_on_overflow() {
        let mut cpu = Cpu8086::new();
        cpu.set_reg(Reg16::AX, 0xFFFF);
        cpu.load(0, 0x0100, &[0x05, 0x01, 0x00]);
        cpu.step();
        assert_eq!(cpu.reg(Reg16::AX), 0x0000);
        assert!(cpu.flags.contains(Flags::CF));
        assert!(cpu.flags.contains(Flags::ZF));
    }

    #[test]
    fn sub_sets_zf_and_cf() {
        let mut cpu = Cpu8086::new();
        cpu.set_reg(Reg16::AX, 5);
        cpu.load(0, 0x0100, &[0x2D, 0x05, 0x00]);
        cpu.step();
        assert_eq!(cpu.reg(Reg16::AX), 0);
        assert!(cpu.flags.contains(Flags::ZF));
        assert!(!cpu.flags.contains(Flags::CF));
    }

    #[test]
    fn cmp_does_not_write_but_sets_flags() {
        let mut cpu = Cpu8086::new();
        cpu.set_reg(Reg16::AX, 3);
        cpu.load(0, 0x0100, &[0x3D, 0x05, 0x00]);
        cpu.step();
        assert_eq!(cpu.reg(Reg16::AX), 3); // unchanged
        assert!(cpu.flags.contains(Flags::CF)); // 3 < 5 → borrow
        assert!(!cpu.flags.contains(Flags::ZF));
    }

    #[test]
    fn je_jne_branch_correctly() {
        let mut cpu = Cpu8086::new();
        cpu.set_reg(Reg16::AX, 1);
        cpu.load(0, 0x0100, &[0x3D, 0x01, 0x00, // cmp ax, 1
                              0x74, 0x02,        // je +2 (skip the 2 nops, land on ret)
                              0x90, 0x90,        // nop nop (skipped)
                              0xC3]);            // ret
        cpu.step(); // cmp
        cpu.step(); // je → jumps to 0x0100 + 5 + 2 = 0x0107
        assert_eq!(cpu.ip, 0x0107);
    }

    #[test]
    fn jne_when_zf_clear() {
        let mut cpu = Cpu8086::new();
        cpu.set_reg(Reg16::AX, 2);
        cpu.load(0, 0x0100, &[0x3D, 0x01, 0x00, // cmp ax, 1
                              0x75, 0x02,        // jne +2
                              0x90, 0x90,        // nop nop (skipped)
                              0xC3]);
        cpu.step(); cpu.step();
        assert_eq!(cpu.ip, 0x0107);
    }

    #[test]
    fn inc_leaves_cf_unchanged() {
        let mut cpu = Cpu8086::new();
        cpu.flags |= Flags::CF; // start with CF set
        cpu.load(0, 0x0100, &[0x40]); // inc ax
        cpu.step();
        assert!(cpu.flags.contains(Flags::CF), "CF must be preserved by INC");
    }

    #[test]
    fn push_pop_round_trip() {
        let mut cpu = Cpu8086::new();
        cpu.set_reg(Reg16::SP, 0xFFFE);
        cpu.set_reg(Reg16::AX, 0xBEEF);
        cpu.load(0, 0x0100, &[0x50, // push ax
                              0x5B]); // pop bx
        cpu.step(); cpu.step();
        assert_eq!(cpu.reg(Reg16::BX), 0xBEEF);
        assert_eq!(cpu.reg(Reg16::SP), 0xFFFE);
    }

    #[test]
    fn loop_decrements_cx_and_branches() {
        let mut cpu = Cpu8086::new();
        cpu.set_reg(Reg16::CX, 2);
        cpu.load(0, 0x0100, &[0xE2, 0xFE, // loop -2 (self-loop until cx=0)
                              0xC3]);
        // 1st loop: cx goes 2→1, jumps back to 0x0100
        cpu.step();
        assert_eq!(cpu.reg(Reg16::CX), 1);
        assert_eq!(cpu.ip, 0x0100);
        // 2nd loop: cx goes 1→0, falls through (LOOP: jump if cx≠0)
        cpu.step();
        assert_eq!(cpu.reg(Reg16::CX), 0);
        assert_eq!(cpu.ip, 0x0102);
        // ret
        cpu.step();
        assert!(cpu.halted);
    }

    #[test]
    fn call_ret_round_trip() {
        let mut cpu = Cpu8086::new();
        cpu.set_reg(Reg16::SP, 0xFFFE);
        // CALL +5, NOP, NOP, NOP, NOP, RET
        cpu.load(0, 0x0100, &[0xE8, 0x05, 0x00, // call +5 (target 0x0108)
                              0x90, 0x90, 0x90, 0x90, 0x90, // nops
                              0xC3]);
        // CALL pushes return address (0x0103), jumps to 0x0108
        cpu.step();
        assert_eq!(cpu.ip, 0x0108);
        assert_eq!(cpu.reg(Reg16::SP), 0xFFFC);
        // RET pops 0x0103, jumps back, halts
        cpu.step();
        assert!(cpu.halted);
        assert_eq!(cpu.ip, 0x0103);
    }

    #[test]
    fn nop_does_nothing() {
        let mut cpu = Cpu8086::new();
        cpu.load(0, 0x0100, &[0x90, 0x90]);
        assert_eq!(cpu.step(), StepResult::Advanced);
        assert_eq!(cpu.ip, 0x0101);
        assert_eq!(cpu.step(), StepResult::Advanced);
        assert_eq!(cpu.ip, 0x0102);
    }

    #[test]
    fn mov_ah_imm8() {
        let mut cpu = Cpu8086::new();
        cpu.load(0, 0x0100, &[0xB4, 0x09]); // mov ah, 9
        cpu.step();
        assert_eq!(cpu.reg(Reg16::AX), 0x0900);
    }

    #[test]
    fn mov_al_imm8() {
        let mut cpu = Cpu8086::new();
        cpu.set_reg(Reg16::AX, 0xFF00);
        cpu.load(0, 0x0100, &[0xB0, 0x42]); // mov al, 0x42
        cpu.step();
        assert_eq!(cpu.reg(Reg16::AX), 0xFF42);
    }
}
