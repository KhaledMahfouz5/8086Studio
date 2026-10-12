# Test plan: 8086Studio

Build: v0.1.0 (milestone 0–1)  Date: 2026-10-11  Env: local, headless + desktop

Desktop tool (no browser/server). Automated cases are Rust integration
tests in `crates/emu8086-bios/tests/`; manual cases need a display.

| case | flow | type | steps | expected | auto | result |
| --- | --- | --- | --- | --- | --- | --- |
| F01-H1 | open template, write, assemble, run, step | happy | New → source → Compile → Run | registers show result, output console shows INT 21h text, halted at RET | e2e (hello) | pass |
| F01-H2 | assemble `mov ax,5; int 21h; ret` | happy | Compile → Step | AX=5 after step, IP advances 3 | e2e (asm+cpu) | pass |
| F01-H3 | save compiled output | happy | Compile → Save .com | `.com` file written, byte-for-byte match | e2e (fmt) | pass |
| F01-E1 | source is empty | edge | Compile with blank buffer | assembler returns empty result, no crash | e2e | pass |
| F01-E2 | very long source (1000 lines) | edge | Compile | assembles; 0 warnings; no panic | manual | |
| F01-E3 | emoji in comment | edge | `; 🎉` then Compile | comment stripped; assembler OK | manual | |
| F01-E4 | file with only `org 100h` | edge | Compile | 0 bytes, entry 0100h | e2e | pass |
| F01-N1 | unknown mnemonic `wat` | negative | Compile | error: "unknown mnemonic `wat`" at line 1 | e2e | pass |
| F01-N2 | missing operand `mov ax` | negative | Compile | error: "`MOV` needs an operand" | e2e | pass |
| F01-N3 | bad register `mov qq,5` | negative | Compile | error: unsupported MOV form | e2e | pass |
| F01-N4 | bad numeric `mov ax,0xGG` | negative | Compile | error: "unknown label or constant `0XGG`" (parsed as label) | e2e | pass |
| F01-N5 | undefined label `jmp nowhere` | negative | Compile | error: "unknown label or constant `NOWHERE`" | e2e | pass |
| F01-N6 | short branch out of range | negative | `jmp target` 200 bytes away | error: "short branch out of range" | e2e | pass |
| F02-H1 | single-step through a loop | happy | Compile loops.asm → Step ×N | CX decrements each LOOP, flags update, IP lands at `ret` | e2e (loops) | pass |
| F02-E1 | register edit mid-run | edge | Step → edit AX → Run | CPU resumes with edited AX | manual | |
| F02-E2 | stack underflow | edge | pop with SP=0 | StepResult::StackUnderflow (or wrap — documented) | manual | |
| F02-N1 | DIV by zero (INT 0) | negative | INT 0 instruction | handler fires or StepResult::Interrupt(0) | manual | |
| F03-H1 | edit register at runtime | happy | double-click AX → type `BEEF` → Enter | AX = 0xBEEF | manual | |
| F03-N1 | invalid hex in register field | negative | type `XYZ` | error shown, value unchanged | manual | |
| F04-H1 | set breakpoint, run, observe halt | happy | click code row → Run | Run stops before executing at BP, RunState=BreakpointHit | e2e (bp test) | pass |
| F04-E1 | breakpoint at entry point | edge | click first instruction → Run | runs once, then stops (not stuck at entry) | e2e | pass |
| F04-N1 | breakpoint on non-instruction row | negative | click `db` row | BP set at data address, ignored during execution | manual | |
| F05-H1 | load .bin with .binf | happy | File → Load .bin | registers preset per .binf, memory loaded at load address | manual | |
| F05-N1 | .bin without .binf | negative | Load .bin alone | default load at 0000:0100h, all regs 0 | manual | |
| F06-H1 | boot from .boot image | happy | Virtual Drive → Boot from floppy | CPU starts at 0000:7C00h | manual | |
| F07-H1 | edit example, recompile | happy | load hello.asm → edit string → Compile → Run | output shows edited string | e2e | pass |
| F08-H1 | disassemble a .com | happy | open .com in code panel | address/hex/mnemonic columns populated | manual | |

## Coverage by flow

| flow | happy | edge | negative | automated | total |
| --- | --- | --- | --- | --- | --- |
| F01 | 3 | 4 | 6 | 12 | 13 |
| F02 | 1 | 2 | 1 | 1 | 4 |
| F03 | 1 | 0 | 1 | 0 | 2 |
| F04 | 1 | 1 | 1 | 2 | 3 |
| F05 | 1 | 0 | 1 | 0 | 2 |
| F06 | 1 | 0 | 0 | 0 | 1 |
| F07 | 1 | 0 | 0 | 1 | 1 |
| F08 | 1 | 0 | 0 | 0 | 1 |
| **total** | **10** | **7** | **10** | **16** | **27** |

Automated = 16 cases (59%). The remaining 11 are manual: runtime editing,
interrupt behaviour, .bin/.boot loading, disassembly view — all need a
display or milestone-2 features not yet wired.
