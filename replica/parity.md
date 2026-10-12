# Parity report — 8086Studio vs emu8086

Date: 2026-10-11 · Build: v0.1.0 (milestone 0–1) · Tool: `parity.py` (replica-diff)

## Scores

| metric | score |
| --- | --- |
| **Overall** | **28.4 / 100** |
| Feature parity | 28.4 (44 rows counted) |
| Must-haves done | **3 of 27** (`Single step`, `Run/Stop/Reset`, `Breakpoints`) |
| Must-haves partial | 13 of 27 |
| Must-haves missing | 11 of 27 |

### Feature parity by area (weakest first)

| area | score | rows |
| --- | --- | --- |
| docs | 0.0 | 1 |
| compat | 0.0 | 2 |
| emulator | 9.4 | 7 |
| editor | 16.7 | 7 |
| assembler | 25.0 | 7 |
| examples | 25.0 | 2 |
| cpu | 28.1 | 7 |
| debugger | 53.2 | 11 |

## Verdict

**NOT SHIPPABLE.** 24 of 27 must-have features are not done. This is the
expected, honest state at the end of milestone 0–1 of a 4-milestone build:
the core loop works (write → assemble → run → inspect → break), the error
paths are tested, and the library boundary is proven, but most user-facing
surfaces (syntax highlight, virtual screen, full BIOS family, output formats,
macros) are milestone-2 items. A user could run `hello.asm` and watch
registers move — and not much more.

No open S1 or S2 bugs (the only open bug is S4, cosmetic).

## Layout parity

**Not measured — no display in this sandbox.** `imgdiff.py` needs PNGs of the
clone, and this environment has no X server / virtual framebuffer, so the GUI
can't be run headless for screenshots. The `studio-preview` binary (built,
6.1 MB) renders the entire design system in one window and is the intended
subject of the layout pass. On any desktop:

```
cargo run --release --bin studio-preview   # then screenshot at 1440x900
python3 imgdiff.py replica/screens/emu8086-main.webp replica/clone-screens/S06.png \
    --json > replica/diffs/S06.json
```

This is a deferred step, not a hidden one — recorded here so it is not
forgotten before launch.

## Behaviour diff (flows, from recon F01–F08)

| flow | original does | clone does | keep / fix |
| --- | --- | --- | --- |
| F01 write→assemble→run | ~5 clicks (New, template, type, Compile, Emulate) | ~5 clicks (same, plus Examples picker is 1 extra panel open) | keep — same shape |
| F02 single-step | F8 / toolbar, registers flash on change | Step button; registers show but no change-flash yet | fix (milestone 2: ChangedSinceStep state already in widget) |
| F03 runtime register edit | click cell, type hex, Enter | double-click opens Extended Viewer (hex/dec/signed/bin) | keep — viewer exists; inline edit is milestone 2 |
| F04 breakpoints | margin click, run to BP, pause | code-row click, Run stops before BP, state = BreakpointHit | keep — matches, tested (f04_h1, f04_e1) |
| F05 .bin + .binf | preset registers, load address | not implemented | fix (milestone 2) |
| F06 boot from floppy | Virtual Drive menu, INT 13h | not implemented | fix (milestone 2+) |
| F07 edit + recompile | auto on Compile | Compile reassembles current buffer; e2e-tested (f07_h1) | keep |
| F08 disassembler view | Tools → Disassembler | code panel shows addr/hex/mnemonic from assembled lines | keep (partial; .com loader milestone 2) |

**Error messages**: clone fails loudly with line + message (e.g.
`line 3: unknown label or constant 'NOWHERE'`). Original uses a modal dialog;
clone uses an inline panel — fix to match, or keep as an improvement
(S05 in test-plan).

**What the clone remembers**: source buffer, examples picker state, run
state, breakpoints, output console contents. Not yet: open file list,
window layout, recent files (milestone 2+).

**Output sink**: INT 21h/09h → clone's Output console (persistent log).
Original paints into a virtual 80x25 screen — clone doesn't have one yet
(must-have gap).

## Missing features in build order (from parity.py)

Must-haves, in the order the architecture milestones place them:

1. **BIOS INT family** — INT 21h AH=01h/02h/0Ah (char I/O), INT 10h (video),
   INT 16h (keyboard), INT 1Ah (time). Blocks `keybrd.asm`, visible output.
2. **Virtual 80x25 text-mode screen** — the pane where INT 10h/09h writes land.
3. **Full flag matrix** — shift/rotate, BCD (`SHL/SHR/SAL/SAR/ROL/ROR`, AAA/DAA).
4. **Syntax-highlighted editor** — keyword/directive/comment/string colouring.
5. **`.EXE` / `.BIN` / `.BOOT` writers + `#make_*#` directives** — output formats.
6. Segment-prefix handling; 1 MiB IVT/system-info; `#LOAD_*` directives;
   `emu8086.inc` macro library (PUTC/PRINT/GOTOXY + DEFINE_*).

## Top five things to build next

1. INT 21h char I/O + INT 10h/16h stubs (unblocks visible input/output).
2. Virtual text-mode screen 80x25 (the missing must-have pane).
3. Full flag matrix — shift/rotate + BCD (arithmetic correctness).
4. Syntax highlighting in the editor (the most visible gap).
5. `.exe/.bin/.boot` + `#make_*#` (output formats + template picker).

## Honest notes

- The score is 28.4/100 and that is the number. Milestone 1 of a
  4-milestone plan was never going to be 80+.
- The scoreboard weights must×3 / should×2 / could×1 and counts partial as
  half — debugger is the strongest area (53.2) because breakpoints, single
  step, run/stop/reset and the extended viewer all work; compat and docs are
  zero because those milestones haven't started.
- 6 of the 7 skip rows are deliberate non-features (serial key, branding,
  FASM bundling, DLL engines, device shells, Windows OCX) — correctly left
  out of the score.
