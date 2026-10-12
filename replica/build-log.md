# Build log — 8086Studio

One entry per screen/milestone. `done` = every recon state + empty/error
built; `partial` = the happy path works but some documented states are
missing; `no` = not started. Dates are build-session dates.

The vertical slice (architecture build order § 0) landed first, then the
v1 must-have stack (asm → cpu → bios → fmt → debug) was extended to run the
bundled examples, then the GUI panes.

| # | ID | date | status | what works | what is missing / harder than expected |
| - | -- | ---- | ------ | ---------- | ---------------------------------------- |
| 0 | slice | 2026-10-11 | done | TextEdit + Compile + Step + `AX=` label across isa/asm/cpu/gui. Proved the library↔GUI boundary. | n/a (intentionally ugly) |
| 1 | asm   | 2026-10-11 | done | Two-pass assembler: 22 mnemonics + `ORG/DB/DW/EQU/END`, forward refs, `[label+disp]`, `[BX/SI/DI]`, `OFFSET`, char/string literals, 58 unit+integration tests. | `#make_*#` directives and `.lst` emit deferred. |
| 2 | cpu   | 2026-10-11 | done | Registers/flags/1 MiB segmented memory, decode+execute for the v1 set, Intel flag matrices (INC/DEC keep CF), PUSH/POP, CALL/RET, LOOP, short Jcc, INT dispatch callback. `.com` load sets SS=CS, SP=0xFFFE. | MUL/DIV/IN/OUT/string/shift/BCD are milestone 2. |
| 3 | bios  | 2026-10-11 | done | INT 21h/09h (`$`-string → host event) and INT 21h/4Ch (exit); `install_v0` handler. | INT 21h/01h,02h,0Ah; INT 10h/13h/16h/19h/1Ah milestone 2. |
| 4 | fmt   | 2026-10-11 | partial | `write_com` (raw bytes) + wired to a "Save .com" button. | `.exe/.bin/.boot` writers milestone 2. |
| 5 | debug | 2026-10-11 | partial | `SourceMap` (line↔addr) + `Breakpoints` set; drive the code-panel current-line + BP dots. | on-disk `.debug`/`.symbol` files, variables window. |
| S01 | Source editor | 2026-10-11 | partial | Editable monospace buffer, gutter, New/Open/Save, Examples picker, dirty-free. | Syntax colouring, find/replace, go-to-line, block-comment toggle (milestone 2). |
| S02 | New/template picker | 2026-10-11 | partial | "New" loads the `.com` template. | The exe/bin/boot/clean template chooser milestone 2. |
| S03 | Examples browser | 2026-10-11 | done | Toolbar "Examples" panel lists all 10 embedded `.asm` with tooltips; click loads. | n/a |
| S04 | Compile | 2026-10-11 | partial | Compile button assembles + loads a fresh CPU, status shows byte/line count and entry CS:IP; Save .com writes output. | The full "save as" dialog with format dropdown milestone 2. |
| S05 | Compile errors panel | 2026-10-11 | partial | On assembly error, a red panel shows `line: message`. | Double-click jump-to-line in the editor milestone 2. |
| S06 | Emulator main view | 2026-10-11 | partial | Code panel (current line + BP dots + click-to-toggle), registers, live flags, DS memory hex+ASCII dump, Run/Step/Stop/Reset, Output console, status bar, run-until-breakpoint. | Stack panel, virtual text-mode screen, floppy widget, register/memory/flag *editing* milestone 2. |
| S07 | Extended Viewer | 2026-10-11 | partial | Double-click a register → panel with hex/dec/signed/binary. | ASCII form; double-click on a memory cell to open the viewer. |
| S09 | Virtual drive manager | — | no | — | Needs `.bin/.boot` + INT 13h; out of v1. |

## Headline notes (bigger than they looked)

- **Two-pass sizing vs encoding.** The hardest part of the assembler was
  making pass 1's address advance agree *exactly* with what pass 2 emits —
  a `[label+disp16]` operand is 4 bytes but a `[reg]` operand is 2, and if
  pass 1 guesses wrong every label after it lands at the wrong address. The
  fix that actually holds: pass 1 calls the *real* encoder (discarding the
  bytes, keeping the length), so the two passes can never disagree. This
  is the "forward references / multi-pass" risk from `architecture.md` § The
  parts that bite, and it bit.
- **`.com` stack init.** A fresh `Cpu8086` with `SP=0` makes every `PUSH`
  underflow instantly. The correct behaviour is DOS's: on load set `SS=CS`
  and `SP=0xFFFE`. Cost a debugging cycle, now fixed and tested
  (`pushpop.asm`).
- **8-bit MOV addressing.** The v1 CPU needed `MOV r8, m8` / `MOV m8, r8`
  (`0x88`/`0x8A`) once examples started touching memory byte-wise. Added a
  shared `resolve_rm` modrm decoder used by both the 8- and 16-bit MOV paths.
- **Screenshots.** The `studio` binary builds and runs on any host with a
  display, but this build sandbox has no X server, so per-screen
  `replica/clone-screens/SNN.png` were not captured. `studio-preview` (the
  design-system page from `/replica-design`) is the closest artefact and
  renders every widget. Screenshots are a one-command step on a real desktop:
  `cargo run --release --bin studio`.

## Feature matrix score

- must: 22 rows → 6 `yes`, 13 `partial`, 3 `no`. The 3 `no` musts (syntax
  highlight, BIOS INT family, virtual screen) are all milestone-2 UI/BIOS
  items; the *core loop* (write → assemble → run → inspect) is fully working.
- The library boundary (`asm → cpu → bios → fmt`) is proven by 5 headless
  end-to-end tests in `crates/emu8086-bios/tests/e2e_hello.rs`, including
  `hello.asm` printing `hello, world`.

## Next

- `/replica-test` → **done**. See `replica/test-plan.md` (27 cases, 16
  automated), `replica/bugs.md` (1 open S4, 3 "to check"), and
  `crates/emu8086-bios/tests/e2e_errors.rs` (12 error-path specs).
- Milestone 2: syntax highlight, the full flag matrix (shift/rotate/BCD),
  INT 21h char I/O + INT 10h/16h, `.exe/.bin/.boot`, macro preprocessor +
  `studio.inc`, pane-tab layout, register/memory editing.

## UNIX-principles audit (2026-10-11)

| principle | status | evidence |
| --- | --- | --- |
| Modularity | ✅ | 7 crates, one job each; asm split into `parse.rs`/`encode.rs`/`lib.rs` (310+391+404 lines) |
| Clarity | ✅ | No clever tricks; `label_refs` macro avoids closure-borrow noise |
| Composition | ✅ | `assemble → cpu::load → cpu::step → bios::dispatch` — values in, values out |
| Separation | ✅ | No file I/O in `cpu`/`isa`/`debug`; no pixels in `asm`/`cpu`/`bios` |
| Simplicity / Parsimony | ✅ | `resolve_addr` no-op removed; dead `estimate_size` deleted; no speculative abstractions |
| Transparency | ✅ | `AsmError` carries line + message; `StepResult` enum covers every outcome |
| Representation | ✅ | Opcode dispatch is `match` on `u8`; flags are `bitflags`; tokens are `const` data |
| Silence | ✅ | Zero `print!`/`println!`/`eprintln!` in library code |
| Repair | ✅ | `validate_labels_resolved` fails loudly on unknown labels; no silent fallbacks |
| Economy | ✅ | No `unwrap()` in production; `expect` only with spelled-out invariants |
| Least Surprise | ✅ | `INC`/`DEC` preserve `CF` (Intel quirk); `.com` loads with `SP=0xFFFE` (DOS convention) |

Resolved this session: 4 PROD `unwrap()` violations, 1 no-op `resolve_addr`,
1 dead `estimate_size`, 1 god module (`asm` 1045 → 3 files < 500 each).
Remaining: `cpu/lib.rs` 747 lines (all `Cpu8086` methods — one job);
`gui/app.rs` 567 lines (policy crate, panels inline in `update()`).
