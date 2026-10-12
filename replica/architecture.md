# Architecture: 8086Studio (a clean-room rebuild of emu8086's core loop)

Desktop IDE: cross-platform GUI in Rust that lets a user write 8086 assembly, assemble it, run it on a virtual 8086, and inspect registers, flags, memory and stack while stepping. v1 ships a hello-world over `INT 21h / AH=09h`. The original emu8086 is a Windows-only proprietary product; we ship our own, free, open-source, cross-platform.

The default web-stack table does not apply (no Postgres, no auth, no payments, no email, no jobs). Schema → Rust structs. API → library public functions. Webhooks/jobs → not applicable. The unix-philosophy shape is one thin binary + a small set of single-purpose crates, mechanism separated from policy.

## Stack

| layer | choice | why |
| --- | --- | --- |
| language | Rust 1.8x+ | user is in a Rust project; emulator cores benefit from `&mut` borrowing and `enum` for CPU state |
| GUI | `eframe` + `egui` (immediate-mode) | pure-Rust, no system webview, runs on Linux/macOS/Windows, fastest to iterate on, easy to ship single binary |
| code editor widget | `egui`'s built-in `TextEdit` + line-number gutter | v1 doesn't need full IDE features; adding `egui_code_editor` later is a 1-crate swap |
| syntax highlight | `syntect` or hand-rolled mini highlighter for ~20 mnemonics | syntect is heavy; hand-rolled wins for v1 |
| font rendering | `egui`'s built-in `FontDefinitions` with monospace + a small proportional set | one font keeps the binary small |
| build / distribution | `cargo build --release` → single static-ish binary per target | `cargo install` works; AppImage/.msi later |
| tests | `cargo test` per crate, plus a fixture corpus of `.asm` + expected `.com` outputs | regression net is more valuable than coverage tooling |
| CI | none for v1 | add when there's a second contributor |

No microservices. No database. One process, one binary.

## Crate layout (workspace)

```
8086Studio/
  Cargo.toml            (workspace)
  crates/
    studio-isa/        pure data: instructions, opcodes, regs, flags, addressing modes
    studio-asm/        text → bytes + .debug/.symbol/.binf side-files
    studio-cpu/        the Cpu8086 model + step/run/reset
    studio-bios/       INT 10h/13h/16h/19h/1Ah/20h/21h stubs
    studio-fmt/        .com / .exe / .bin / .boot writers (and loaders)
    studio-debug/      SourceMap + Breakpoint types
    studio-gui/        eframe app: editor + emulator + memory + flags panes
  src/main.rs           thin shim: eframe::run_native with emu8086_gui::App
  examples/             one bundled hello-world.asm + maybe a sum-an-array.asm
  replica/              this folder
```

Rules of the layout:
- Each crate has one job. `studio-cpu` does not know what a file is. `studio-asm` does not know what a register is — only that "AX" is a token whose encoding the ISA crate knows.
- The GUI is the only crate that knows about pixels, file paths and user preferences.
- All non-GUI crates are `no_std`-friendly where possible so they can be reused in tests, a future CLI, a future web build (WASM), or a future TUI.

## Data model (Rust structs — no SQL)

No database. All state lives in the running process and in `.asm`/`.com`/`.exe`/`.bin`/`.boot` files on disk. The data model becomes a set of structs shared between crates.

```rust
// studio-isa
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Reg16 { AX, CX, DX, BX, SP, BP, SI, DI }
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SegReg { ES, CS, SS, DS }
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Reg8 { AL, CL, DL, BL, AH, CH, DH, BH }

bitflags! {
    pub struct Flags: u16 {
        const CF = 1 << 0; const PF = 1 << 2; const AF = 1 << 4;
        const ZF = 1 << 6; const SF = 1 << 7; const TF = 1 << 8;
        const IF = 1 << 9; const DF = 1 << 10; const OF = 1 << 11;
    }
}

pub enum Operand { Reg16(Reg16), Reg8(Reg8), Seg(SegReg),
                   Imm8(i8), Imm16(i16), Mem(MemRef) }
pub enum MemRef { Direct(u16), Reg(Reg16), RegDisp(Reg16,i16),
                  RegReg(Reg16,Reg16), RegRegDisp(Reg16,Reg16,i16) }

pub struct Instruction {
    pub mnemonic: Mnemonic,
    pub operands: Vec<Operand>,
    pub length_bytes: u8,
    pub raw: [u8; 6],     // up to 6 bytes; rest unused
}

// studio-cpu
pub struct Cpu8086 {
    pub regs: [u16; 8],       // AX..DI, indexed by Reg16
    pub segs: [u16; 4],       // ES,CS,SS,DS
    pub ip: u16,
    pub flags: Flags,
    pub memory: Box<[u8; 0x100000]>,
    pub halted: bool,
    pub last_instr: Option<Instruction>,
    pub last_source_line: Option<u32>,
}

// studio-asm
pub struct AsmResult {
    pub bytes: Vec<u8>,            // assembled machine code
    pub entry_cs_ip: (u16, u16),
    pub source_map: SourceMap,     // for debugger
    pub symbols: Vec<(String, u16)>,
    pub binf: Option<Binf>,        // present iff source used #make_bin# / #make_boot#
}

// studio-debug
pub struct SourceMap { pub line_to_addr: Vec<(u32, u16)>, pub addr_to_line: Vec<(u16, u32)> }
pub struct Breakpoints { pub addrs: BTreeSet<u16> }

// studio-fmt
pub enum OutputFormat { Com, Exe, Bin, Boot }
pub struct Binf { pub regs: [u16;8], pub segs: [u16;4], pub ip: u16,
                  pub load_seg: u16, pub load_off: u16,
                  pub mem_prefill: Vec<(u32, Vec<u8>)> }

// studio-gui  (the only crate with user-visible state)
pub struct App {
    pub editor_text: String,
    pub editor_path: Option<PathBuf>,
    pub cpu: Cpu8086,
    pub last_asm: Option<AsmResult>,
    pub last_error: Option<String>,
    pub breakpoints: Breakpoints,
    pub run_state: RunState,    // Idle, Running, Paused, Halted, BreakpointHit
    pub current_pane: Pane,     // Editor, Emulator, Memory, Flags, Stack, Help
}
```

## API (library public functions — replaces HTTP routes)

Desktop tools have no HTTP API; the equivalent is the library surface that the GUI and a future CLI both call. One function per flow from the recon map.

| fn | what it does | caller | input | output | flow |
| --- | --- | --- | --- | --- | --- |
| `asm::assemble(source, format_hint)` | turn source into bytes + side-files | GUI | `&str`, `OutputFormat` | `Result<AsmResult, AsmError>` | F01, F07 |
| `cpu::Cpu8086::load(prog)` | copy bytes into memory at entry CS:IP | GUI | `&Program` | `()` | F01, F05, F08 |
| `cpu::Cpu8086::reset()` | zero state + reload last program | GUI | self | `()` | F01 |
| `cpu::Cpu8086::step()` | execute one instruction | GUI | self | `StepResult` | F02 |
| `cpu::Cpu8086::run_until(bp, max_steps)` | run with breakpoint | GUI | self, &Breakpoints, u64 | `StepResult` | F04 |
| `cpu::Cpu8086::write_reg/write_seg/write_flags` | runtime patch | GUI | self, ... | `Result<(), RegError>` | F03 |
| `cpu::Cpu8086::read_mem(seg, off)` | memory read for the Memory pane | GUI | self, u16, u16 | `u8` | F02 |
| `bios::dispatch(cpu, int_no)` | handle INT n | CPU internals | &mut Cpu8086, u8 | `()` | F01 (for hello via INT 21h) |
| `fmt::write_com(path, bytes)` | save .com | GUI | &Path, &[u8] | `io::Result<()>` | F01, F04 (output side) |
| `fmt::write_exe(path, asm_result)` | save .exe | GUI | &Path, &AsmResult | `io::Result<()>` | F01, F04 |
| `fmt::write_bin(path, asm_result)` | save .bin + .binf | GUI | &Path, &AsmResult | `io::Result<()>` | F05 |
| `fmt::write_boot(path, asm_result)` | save .boot (forces CS=0 IP=7C00) | GUI | &Path, &AsmResult | `io::Result<()>` | F06 |
| `debug::SourceMap::line_for(addr)` / `addr_for(line)` | source-line ↔ address lookup | GUI | ... | `Option<u32> / Option<u16>` | F02 (highlight current line) |
| `cpu::Cpu8086::snapshot()` / `restore(snap)` | history buffer for "step backward" | GUI | self | `CpuSnap` | F02 backward (should-tier) |
| `examples::load_builtin(name)` | pull a bundled `.asm` from the `examples/` dir | GUI | &str | `String` | F07 |

Webhooks in/out: **none**.  Jobs: **none**.  Both N/A for a local desktop tool with no network use.

## The parts that bite

- **Flag-update rules across instructions.** esp. `INC`/`DEC` leave CF unchanged (per Intel); `SHL/SHR/SAR/SAL/ROL/ROR/RCL/RCR` have a 6-row flag matrix; `DAA/DAS/AAA/AAD/AAM/AAS` are conditional, not pure arithmetic. Test fixture: every instruction × every operand form × known flag input → known flag output. Use the yassinebridi mirror's per-instruction flag matrix as the oracle.
- **Segment-override prefix on string ops.** A `REP MOVSB` must read `DS:[SI]` (or the segment in the prefix byte) and write `ES:[DI]` (or the prefix-segment if present), with `DF` controlling direction. Easy to get wrong; common test target.
- **Forward references / multi-pass.** emu8086 is multi-pass. `JMP start` before `start:` is in the source must encode the right offset. Our assembler needs at least two passes OR a fixup-resolution pass over an intermediate representation.
- **Source-line ↔ address map.** A single source line can produce several instructions (e.g. `MOV AL, [BX+SI+5]` → 3 bytes). The `.debug` file format we emit must let the GUI highlight the line whose range contains the current `CS:IP`.
- **Stack underflow / SP wrap.** Real 8086 happily wraps `SP` to 0xFFFF. We should trap (return `StepResult::StackUnderflow`) rather than silently corrupt memory.
- **`DIV` / `IDIV` by zero → INT 0.** Real hardware raises interrupt 0. The BIOS dispatch table must wire `INT 0` to a visible error in the GUI, not a panic.
- **`INT 21h / AH=09h` needs a `$`-terminator.** The DOS string write convention is `$`-terminated, not NUL-terminated. Trivial to miss; trivial to test.
- **`.com` files load at `CS:IP = 0x0100`.** The ISA spec, not a quirk. The assembler must add `org 100h` automatically when `org 100h` is missing but `make_com` is selected, or error out — emu8086's choice; we mirror it.
- **No cycle accounting.** We chose functional-only. Don't accidentally implement "best-effort" timing that is neither accurate nor absent — pick absent and document it.

## Build order

### 0. Vertical slice (the "ugly end-to-end")
- One `eframe` window with: a TextEdit for source, a "Compile" button, a "Step" button, and a label showing `AX`.
- `studio-isa` defines just `MOV`.
- `studio-asm` lexes/parses `MOV AX, 5` → 3 bytes `B8 05 00`.
- `studio-cpu::step()` decodes those 3 bytes and updates `AX`.
- The label shows `AX = 5`. Ugly. Proves the stack + the workspace + the GUI-to-library call boundary.

### 1. v1 must-haves (the thing the user can actually use)
- ISA: extend to `MOV, ADD, SUB, CMP, JMP, JE, JNE, INT, RET, HLT, INC, DEC, PUSH, POP, CALL, LOOP, NOP`. (~16 mnemonics.)
- Assembler: lex/parse the whole 8086 Intel-syntax surface used by `hello-world` + `sum-an-array` examples. Directives: `ORG`, `DB`, `DW`, `EQU`, `END`, plus the four `#make_*#` directives and `#MEM=`.
- **Macro library** at `examples/studio.inc` exposing `PUTC`, `PRINT`, `PRINTN`, `CURSORON`, `CURSOROFF`, `GOTOXY`, plus `DEFINE_PRINT_STRING` (others optional). Spec: `replica/studio-macros.md`. Reuses `INT 21h/09h` and `INT 10h/0Eh` per public BIOS/DOS contracts.
- Emulator: full flag-update matrix for the above. Trap DIV-by-zero (INT 0). Trap stack underflow.
- BIOS stub: `INT 21h / AH=09h` (write `$`-terminated string at DS:DX to the GUI's stdout pane).
- FMT: `.com` writer.
- GUI panes: editor, registers, flags, memory (segmented view), stack, code-highlight (via .debug map).
- Buttons: New / Open / Save / Compile / Emulate / Run / Single Step / Stop / Reset.
- Screens: S01, S04, S05, S06, S07. Tables/routes: all of `asm::assemble`, `cpu::step`, `cpu::run_until`, `fmt::write_com`, `bios::dispatch`.
- Bundled examples (per `replica/examples-todo.md` Tier 1): `hello.asm`, `sum.asm`, `1_sample.asm`, `int21.asm`, `keybrd.asm`, `count_chars.asm`, `loops.asm`, `add_two_arrays.asm`, `convert_to_upper_case.asm`, `include.asm`.

### 2. Must-haves (rest of features.csv `must`)
- ISA: complete 8086 set (data movement, arithmetic, logical, shift/rotate, BCD, string, control flow, flags, processor control). ~100 mnemonics.
- FMT: `.exe`, `.bin` + `.binf`, `.boot` writers/loaders.
- BIOS: `INT 10h` video (00h set mode, 02h set cursor, 0Eh teletype, 13h write string), `INT 13h` read/write sectors (stub against virtual floppy), `INT 16h` kbd, `INT 19h` reboot, `INT 1Ah` time, `INT 20h` terminate.
- GUI: New → template picker (com/exe/bin/boot), Examples browser, Variables window, Disassembler view, Help tree (rebuilt from our own docs).
- Screens: S02, S03, S08, S13, S12. Routes: `examples::load_builtin`, `debug::SourceMap::line_for`.

### 3. Should-haves
- Source templates (S02 already shipped with com/exe/bin/boot) fleshed out.
- Step backward (cpu::snapshot/restore history buffer).
- Virtual floppy drives + `INT 13h` real backing.
- Custom memory map (`c:\emu8086\custom_memory_map.inf` equivalent — feature gated, opt-in).
- Variables window live values.
- Two more bundled examples: `sum-an-array.asm`, `led-blink-via-INT 13h.asm` (or `traffic-lights-via-OUT-4.asm` if we ship virtual peripherals).

### 4. Could-haves (only after the rest works)
- Cycle-accurate timing (deferred by v1 choice — revisit if anyone ports timing-sensitive demos).
- ALU internal-operation visualization (S14) — animated data-path view.
- Full examples library (LED, traffic light, motor, micro-OS) — we write our own.
- `.lst` listing files with addresses + bytes.
- TASM/MASM-compat macros from the original.
- `.chm`-equivalent offline help.
- Web build via WASM (`studio-cpu` is already `no_std`-friendly).
- TUI build via `ratatui` (same library, different front end).

### 5. Fixes from replica-entrepreneur
- Run that skill when the v1 is in users' hands. Add the resulting fixes as milestones, ranked by frequency in real complaints.

## Out-of-scope reminders (carry-over from recon)
- The "Repack" emu8086 zip with someone else's serial key — not a source, not used, not redistributed.
- Original emu8086 branding, logo, examples library, `.chm` help — all written fresh under our name.
- Virtual peripheral kit visuals (LED panels, traffic lights, stepper motors, robot) — design original if we ever ship them; do not copy the original's art or example programs.

## Files written this step
- `replica/architecture.md` — this file
- `replica/schema.rs` — the Rust structs above as a single module the v1 skeleton can `include!`
- `replica/studio-macros.md` — the spec for our re-implementation of emu8086's `PRINT`/`PRINTN`/`PUTC`/etc. macro surface (names + signatures only, bodies written fresh)
- `replica/examples-todo.md` — our own bundled example library, derived from the filename index but written from scratch
- `replica/screens/payload-filenames.txt` — names-only index of the original's installer payload (311 files), for cross-reference

## Design status (after `/replica-design`)
- `replica/design/tokens.json` — 26 color roles, 32 contrast pairs, 0 failing AA. The `egui_visuals_overrides` bridge keys (`window_stroke`, `selection.bg_fill`, `selection.stroke`, plus the widget-state table) match `theme.rs::apply()` field-for-field.
- `replica/design/tokens.css` — custom properties for every color role + type tokens + motion tokens.
- `replica/design/components.md` — 19 components documented, each with variants/sizes/states/tokens/a11y/screens. 11 are shipped in v1 (`Button`, `ToolButton`, `Editor` read-only, `LineGutter`, `RegisterCell`, `FlagBit`, `MemoryCell`, `CodePanel`, `BreakpointDot`, `Toolbar`, `PaneTabStrip`, `StatusBar`); 8 are deferred to later milestones. Deferred = documented but no `widgets/<name>.rs` file.
- `crates/studio-gui/src/theme.rs` — the Rust port of the JSON; `DEFAULT_TOKENS` + `Tokens::from_json` + `apply(&mut Style, &Tokens)`.
- `crates/studio-gui/src/widgets/{button,tool_button,register_cell,flag_bit,memory_cell,breakpoint_dot,line_gutter,toolbar,pane_tab_strip,status_bar,code_panel,editor}.rs` — 11 widget files, one job each, no hardcoded colors.
- `crates/studio-gui/src/preview.rs` — renders every shipped widget in every documented state, in a single window. `cargo run --release --bin studio-preview` is the design-system screenshot check.
- `crates/studio-gui/tests/tokens_consistent.rs` — drift detector. Fails the build if `DEFAULT_TOKENS` and `tokens.json` diverge.
- `cargo test --workspace` → 5 passed, 0 failed.
- `python3 ~/.config/opencode/skills/replica-design/contrast.py replica/design/tokens.json` → 32 pairs, 0 failing AA.

## Build status (after `/replica-build` milestone 0–1)
- All 7 crates build with **0 warnings**. `cargo test --workspace` → **58
  passed**, including 6 headless end-to-end tests in
  `crates/studio-bios/tests/e2e_hello.rs` that run `hello.asm`, `sum.asm`,
  `loops`, `pushpop`, `add_two_arrays`, `cmp` through the real
  `asm → cpu → bios` stack (`hello.asm` prints `hello, world`).
- Release binaries: `target/release/studio` (the app, 6.3 MB) and
  `target/release/studio-preview` (the design page, 6.1 MB).
- `examples/` ships **10** Tier-1 `.asm` files; all assemble and run headless.
- Milestones § 0 (vertical slice) and § 1 (v1 core loop) are **done**; § 1
  UI-only musts (syntax highlight, virtual screen, full BIOS INT family,
  format selection) are deferred to milestone 2.
- See `replica/build-log.md` for per-screen status and the two parts that
  were harder than expected (two-pass sizing/encoding agreement; `.com`
  stack-pointer init).
