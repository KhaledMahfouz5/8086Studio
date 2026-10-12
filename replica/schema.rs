// replica/schema.rs
// The data model for the 8086Studio rebuild, lifted out of architecture.md
// so the v1 skeleton can `include!("../replica/schema.rs")` and start filling
// the structs in. No external deps beyond `bitflags` (a tiny std-only crate).
//
// This is not a database. There is no database. Everything below lives in
// process memory or in `.asm`/`.com`/`.bin`/`.boot` files on disk.

use std::collections::BTreeSet;

// ---------- studio-isa ----------

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Reg16 { AX, CX, DX, BX, SP, BP, SI, DI }

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Reg8 { AL, CL, DL, BL, AH, CH, DH, BH }

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SegReg { ES, CS, SS, DS }

bitflags::bitflags! {
    /// 8086 FLAGS register, active bits only.
    pub struct Flags: u16 {
        const CF = 1 << 0;
        const PF = 1 << 2;
        const AF = 1 << 4;
        const ZF = 1 << 6;
        const SF = 1 << 7;
        const TF = 1 << 8;
        const IF = 1 << 9;
        const DF = 1 << 10;
        const OF = 1 << 11;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Operand {
    Reg16(Reg16),
    Reg8(Reg8),
    Seg(SegReg),
    Imm8(i8),
    Imm16(i16),
    Mem(MemRef),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MemRef {
    Direct(u16),
    Reg(Reg16),
    RegDisp(Reg16, i16),
    RegReg(Reg16, Reg16),
    RegRegDisp(Reg16, Reg16, i16),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Mnemonic {
    Aaa, Aad, Aam, Aas, Adc, Add, And, Call, Cbw, Clc, Cld, Cli, Cmc, Cmp,
    Cmpsb, Cmpsw, Cwd, Daa, Das, Dec, Div, Hlt, Idiv, Imul, In, Inc, Int,
    Into, Iret, Ja, Jae, Jb, Jbe, Jc, Jcxz, Je, Jg, Jge, Jl, Jle, Jmp,
    Jna, Jnae, Jnb, Jnbe, Jnc, Jne, Jng, Jnge, Jnl, Jnle, Jno, Jnp, Jns,
    Jnz, Jo, Jp, Jpe, Jpo, Js, Jz, Lahf, Lds, Lea, Les, Lodsb, Lodsw, Loop,
    Loope, Loopne, Loopnz, Loopz, Mov, Movsb, Movsw, Mul, Neg, Nop, Not, Or,
    Out, Pop, Popa, Popf, Push, Pusha, Pushf, Rcl, Rcr, Rep, Repe, Repne,
    Repnz, Repz, Ret, Retf, Rol, Ror, Sahf, Sal, Sar, Sbb, Scasb, Scasw,
    Shl, Shr, Stc, Std, Sti, Stosb, Stosw, Sub, Test, Xchg, Xlatb, Xor,
}

#[derive(Clone, Debug)]
pub struct Instruction {
    pub mnemonic: Mnemonic,
    pub operands: Vec<Operand>,
    pub length_bytes: u8,
    pub raw: [u8; 6], // up to 6 bytes; rest unused
}

// ---------- studio-cpu ----------

pub struct Cpu8086 {
    /// Indexed by `Reg16 as usize`.
    pub regs: [u16; 8],
    /// Indexed by `SegReg as usize` in order ES, CS, SS, DS.
    pub segs: [u16; 4],
    pub ip: u16,
    pub flags: Flags,
    /// 1 MiB flat byte-addressable memory.
    pub memory: Box<[u8; 0x100000]>,
    pub halted: bool,
    pub last_instr: Option<Instruction>,
    pub last_source_line: Option<u32>,
}

pub enum StepResult {
    Ok,
    Halted,
    Breakpoint,
    DivByZero,
    StackUnderflow,
    UnsupportedInstruction(u8),
}

impl Cpu8086 {
    pub fn new() -> Self { todo!() }
    pub fn reset(&mut self) { todo!() }
    pub fn load(&mut self, prog: &Program) { todo!() }
    pub fn step(&mut self) -> StepResult { todo!() }
    pub fn run_until(&mut self, bp: &Breakpoints, max_steps: u64) -> StepResult { todo!() }
    pub fn read_reg16(&self, r: Reg16) -> u16 { todo!() }
    pub fn write_reg16(&mut self, r: Reg16, v: u16) { todo!() }
    pub fn read_reg8(&self, r: Reg8) -> u8 { todo!() }
    pub fn write_reg8(&mut self, r: Reg8, v: u8) { todo!() }
    pub fn read_seg(&self, s: SegReg) -> u16 { todo!() }
    pub fn write_seg(&mut self, s: SegReg, v: u16) -> Result<(), &'static str> { todo!() }
    pub fn read_mem(&self, seg: u16, off: u16) -> u8 { todo!() }
    pub fn write_mem(&mut self, seg: u16, off: u16, v: u8) { todo!() }
    pub fn snapshot(&self) -> CpuSnap { todo!() }
    pub fn restore(&mut self, snap: CpuSnap) { todo!() }
}

#[derive(Clone, Debug)]
pub struct CpuSnap {
    pub regs: [u16; 8],
    pub segs: [u16; 4],
    pub ip: u16,
    pub flags: Flags,
}

// ---------- studio-asm ----------

#[derive(Clone, Debug)]
pub struct AsmResult {
    pub bytes: Vec<u8>,
    pub entry_cs_ip: (u16, u16),
    pub source_map: SourceMap,
    pub symbols: Vec<(String, u16)>,
    pub binf: Option<Binf>,
}

#[derive(Debug)]
pub enum AsmError {
    Lex(String),
    Parse { line: u32, msg: String },
    UnknownMnemonic(String),
    BadOperandCount { line: u32, mnemonic: String },
    BadOperand { line: u32, msg: String },
    LabelRedefined { name: String },
    UnsupportedDirective(String),
}

pub fn assemble(source: &str) -> Result<AsmResult, AsmError> { todo!() }

// ---------- studio-debug ----------

#[derive(Default, Clone, Debug)]
pub struct SourceMap {
    pub line_to_addr: Vec<(u32, u16)>,
    pub addr_to_line: Vec<(u16, u32)>,
}

impl SourceMap {
    pub fn line_for(&self, _addr: u16) -> Option<u32> { todo!() }
    pub fn addr_for(&self, _line: u32) -> Option<u16> { todo!() }
}

#[derive(Default, Clone, Debug)]
pub struct Breakpoints {
    pub addrs: BTreeSet<u16>,
}

// ---------- studio-fmt ----------

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OutputFormat { Com, Exe, Bin, Boot }

#[derive(Clone, Debug)]
pub struct Program {
    pub format: OutputFormat,
    pub bytes: Vec<u8>,
    pub entry: (u16, u16),
    pub binf: Option<Binf>,
}

#[derive(Default, Clone, Debug)]
pub struct Binf {
    pub regs: [u16; 8],
    pub segs: [u16; 4],
    pub ip: u16,
    pub load_seg: u16,
    pub load_off: u16,
    pub mem_prefill: Vec<(u32, Vec<u8>)>,
}

pub mod fmt {
    use super::*;
    pub fn write_com(_path: &std::path::Path, _bytes: &[u8]) -> std::io::Result<()> { todo!() }
    pub fn write_exe(_path: &std::path::Path, _asm: &AsmResult) -> std::io::Result<()> { todo!() }
    pub fn write_bin(_path: &std::path::Path, _asm: &AsmResult) -> std::io::Result<()> { todo!() }
    pub fn write_boot(_path: &std::path::Path, _asm: &AsmResult) -> std::io::Result<()> { todo!() }
}

// ---------- studio-bios ----------

pub fn dispatch_int(cpu: &mut Cpu8086, int_no: u8) { todo!() }

// ---------- studio-gui (the only crate that knows about pixels) ----------

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RunState { Idle, Running, Paused, Halted, BreakpointHit }

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Pane { Editor, Emulator, Memory, Flags, Stack, Help, Variables, Disasm }

pub struct App {
    pub editor_text: String,
    pub editor_path: Option<std::path::PathBuf>,
    pub cpu: Cpu8086,
    pub last_asm: Option<AsmResult>,
    pub last_error: Option<String>,
    pub breakpoints: Breakpoints,
    pub run_state: RunState,
    pub current_pane: Pane,
    pub history: Vec<CpuSnap>, // for "step backward"
}
