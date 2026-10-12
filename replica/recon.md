# Recon map: emu8086 (desktop)

Scope: write 8086 assembly → assemble → emulate on an 8086 CPU → debug. Clean-room Rust port. Excludes the original "micro-OS / virtual peripheral" hardware kit, the bundled Windows installer story, and the licensed code-example library.
For: educational tool / Rust-native 8086 learning environment (open source).
Date: 2026-10-11

## Sources

| # | source | URL | notes |
| --- | --- | --- | --- |
| 1 | Intel 8086 — Wikipedia | https://en.wikipedia.org/wiki/Intel_8086 | Public ISA spec: registers, flags, segmentation, interrupts, prefetch queue, addressing modes, execution timings. Public, decades-old. |
| 2 | x86 assembly language — Wikipedia | https://en.wikipedia.org/wiki/X86_assembly_language | Mnemonics, Intel vs AT&T syntax, reserved-word rules, addressing modes, register roles. |
| 3 | Assembly language — Wikipedia | https://en.wikipedia.org/wiki/Assembly_language | One-pass vs multi-pass assemblers, directive semantics, macro processors. |
| 4 | emu8086 product page (EmuApps mirror) | https://emu8086.emuapps.com/ | Third-party description of the public-facing product: editor, assembler, disassembler, emulator, virtual devices, ALU viz, virtual-machine isolation. |
| 5 | emu8086 user manual mirror (yassinebridi) | https://yassinebridi.github.io/asm-docs/help.html | Community-restored copy of the original v4 manual (editor.html, compiler.html, emulator.html, memory.html, custom_memory_map.html, io.html, 8086_instruction_set.html, 8086_bios_and_dos_interrupts.html). Describes documented user-facing behavior only. |
| 6 | SourceForge project page | https://sourceforge.net/app/emu8086-microprocessor-emulator/ | Public overview, features, license ("Free Trial" — proprietary), Windows-only. |
| 7 | GitHub topic: 8086-emulator | https://github.com/topics/8086-emulator | Public reference implementations in Rust, C++, WebAssembly, JS — sanity check that the ISA can be re-implemented. Notable Rust projects: `Lichtso/DOS-Emulator`, `aidlx/core86`, `retrohun/8086tiny`, `Thraetaona/EXACT` (WASM). |
| 8 | the user's own project folder | /home/khaled/.../8086Studio/ | Owns a `emu8086v408r11.zip` (readme reveals it is a "Repack for Windows 10" with someone else's serial key). Treated as NOT a source — see "Out of scope / Terms" below. |
| 9 | Intel 8086 / 8088 datasheet (public PDFs, mirrors) | http://www.nj7p.info/Manuals/PDFs/Intel/MANUAL/ and others | Intel's original datasheets are public. Used to confirm flag layout, segment shift, queue, instruction timings. |

## Core loop

Write 8086 assembly → assemble to machine code → load into a CPU model → step / run → watch registers, flags, memory and stack change after every instruction.

## Screens

| ID | screen | route / how to reach | purpose | key components | states seen |
| --- | --- | --- | --- | --- | --- |
| S01 | Source editor | main window after "New" / open .asm | write / edit assembly | syntax-highlight editor, line gutter, bookmarks, find/replace (regex), split view, Go-to-line, block comment toggle (Ctrl+Q / Ctrl+W), tabs | empty, modified, dirty-on-disk, file open |
| S02 | New file picker | File → New | start a project | templates: `com template`, `exe template`, `bin template`, `boot template`, `Clean (no template)` | n/a |
| S03 | Examples browser | File → Examples | load bundled sample | tree of .asm examples with comments | n/a |
| S04 | Compile dialog | click Compile button | save compiled output | "save as" with .com / .exe / .bin / .boot dropdown; auto-detected from `org 100h` or `stack` segment or `.BINF` directives | success, error |
| S05 | Compile errors panel | modal after compile failure | show assembly errors | line-numbered error list, double-click to jump | empty, filled, single-error, multi-error |
| S06 | Emulator main view | click Emulate / Emulator → Show emulator | run the program on a virtual 8086 | code panel (CS:IP line highlighted), registers panel (AX/BX/CX/DX + SP/BP/SI/DI + AH/AL/CH/CL/DH/DL/BH/BL), flags panel, memory panel (segmented view), stack panel, virtual screen, floppy drives widget, toolbar (Run / Single Step / Stop / Reset / Flags) | idle, running, paused, halted, after-step, breakpoint-hit, runtime-register-edit |
| S07 | Extended Viewer | double-click on any register or memory cell | show value in all formats | binary / hex / decimal / signed / ASCII, editable | read-only, editable |
| S08 | Variables window | Emulator menu when .symbol file present | show named symbols | symbol name → address + value | empty, filled |
| S09 | Virtual drive manager | Emulator → Virtual Drive menu | attach floppy images | FLOPPY_0..FLOPPY_3 file list, "create new floppy", "write 512 bytes at 7C00 to boot sector", "boot from floppy" | no drives, one drive, max four drives |
| S10 | Custom Memory Map editor | file: `c:\emu8086\custom_memory_map.inf` | override BIOS/ROM/memory layout | "address - filename" rows + optional `NO_SYS_INFO` directive | valid, parse error |
| S11 | Settings / Options | menu | configure emulator | screen size, step delay, default template, color theme | n/a |
| S12 | Help / Tutorial index | Help menu | learn the tool | tree of HTML pages mirroring the manual | n/a |
| S13 | Disassembler view | Tools → Disassembler (or auto when loading .com/.exe/.bin) | show machine code alongside source | address column, hex bytes column, mnemonic column, label column | empty, partial, full |
| S14 | ALU internal visualization | a feature flag in some 4.x versions | teach the datapath | animated diagram of registers, buses, ALU, flags updating per cycle | step-paused |

## Flows

```
F01 Open template, write, assemble, run, step
    S02 -> S01 -> S04 -> S06
    happy path clicks: New -> pick template -> type -> Compile -> Emulate -> Single Step (N times)
    edge: org directive missing for .com; example not found; .bin without .binf

F02 Single-step through a loop, watch registers / flags change
    S06 (single step) -> S07 (extended viewer) -> S06 (single step)
    happy path clicks: 1 click per instruction; double-click a register opens extended viewer
    edge: instruction pointer moves into interrupt stub; stack underflow; DIV by 0 (INT 0)

F03 Edit a register at runtime to test a hypothesis
    S06 (click register text-box, type new hex) -> S06 (resume Run / Single Step)
    happy path clicks: 2 clicks + a typed value + 1 click to resume
    edge: invalid hex; changing CS:IP mid-instruction; segment-register writes ignored

F04 Set a breakpoint, run, observe halt
    S06 (click in left margin of code panel to set BP) -> S06 (Run) -> S06 (paused at BP)
    happy path clicks: 1 click margin + 1 click Run
    edge: BP on data; BP in prefix segment; BP never hit (wrong address)

F05 Load a .bin with a .binf file (load address + initial register state)
    S06 (File -> Load .bin) -> S06 (loaded at LOAD_SEGMENT:LOAD_OFFSET with .binf register state)
    happy path clicks: 1 click to load
    edge: .binf missing (fallback to default.binf); hex parsing error; CS:IP outside free memory

F06 Boot a .boot image from the virtual floppy
    S01 (write boot sector .asm with org 7c00h) -> S04 (compile to .boot) -> S06 (Virtual Drive -> write 512 bytes to boot sector) -> S06 (Virtual Drive -> boot from floppy)
    happy path clicks: compile + 2 menu clicks + 1 boot
    edge: image > 512 bytes; CS:IP not 0000:7C00; FLOPPY_0 missing

F07 Edit a .asm example, recompile, re-emulate
    S03 (load example) -> S01 (edit) -> S04 (compile) -> S06 (re-emulate)
    happy path clicks: open + edit + compile + emulate
    edge: example uses directives the assembler rejects (e.g. MASM-only)

F08 Disassemble an existing .com / .exe
    S13 (open) -> S13 (browse) -> S06 (load into emulator)
    happy path clicks: open disasm + load
    edge: header stripped; relocation table present; segment != CS
```

Happy-path click count to beat: **4** (New → write → Compile → Emulate → Single Step). Emu8086 itself sits at that number.

## Components

| component | variants | states | used on |
| --- | --- | --- | --- |
| Source editor | com / exe / bin / boot / clean template | empty, dirty, readonly, syntax-highlight on/off | S01 |
| Register cell | 16-bit (AX/BX/CX/DX/SP/BP/SI/DI), 8-bit halves (AH/AL…), segment (CS/DS/ES/SS) | normal, focused (editable), changed-since-step (flash) | S06, S07 |
| Flags panel | OF / DF / IF / TF / SF / ZF / AF / PF / CF | bit-set (1), bit-clear (0), individually toggleable | S06 |
| Memory panel | segmented (CS: / DS: / ES: / SS:), byte cells, address column | scroll, hex view, ASCII view, edit on click | S06, S07 |
| Stack panel | SS:SP view, top-down growth | underflow highlight, sp-edit | S06 |
| Code panel | CS:IP view, source-mapping (via .debug file) | current-line highlight, breakpoint marker, instruction bytes tooltip | S06, S13 |
| Toolbar | Run / Single Step / Stop / Reset / Flags / Back Step | enabled / disabled per state | S06 |
| Virtual screen | 80x25 text / 320x200 13h / 40x25 CGA | render to host canvas, video page selector (INT 10h AH=05h) | S06 |
| Virtual floppy widget | FLOPPY_0..FLOPPY_3 file icons | mounted, write-protected, boot-from-here | S06, S09 |
| Breakpoint dot | none, set | toggle on line | S06 |
| Disassembler view | address / hex bytes / mnemonic / source comment | per-instruction | S13 |
| Variables list | name, address, current value | updated each step when .symbol loaded | S08 |
| Extended Viewer modal | bin / hex / dec / signed-dec / ASCII | read-only, editable | S07 |
| Error list | line, column, message | empty, multi | S05 |
| Help tree | hierarchical HTML | n/a | S12 |

## Inferred data model

```
Cpu8086   14 16-bit regs (AX,BX,CX,DX,SP,BP,SI,DI,IP,FLAGS,CS,DS,ES,SS),
          4 segment regs, IP, FLAGS (9 active flag bits: OF DF IF TF SF ZF AF PF CF),
          instruction pointer, prefetch queue (up to 6 bytes),
          clock-cycle counter (optional, for timing-accurate mode),
          halted flag, last-executed-instruction size,
          interrupt mask, pending-interrupt queue
          evidence: Intel 8086 Wikipedia, emu8086 emulator.html (registers/flags panels), 8086_instruction_set.html
          confidence: high

Memory    1 MiB flat byte-addressable, but accessed as segment:offset
          (physical = segment << 4 + offset, 20-bit),
          4 pages x 4096 bytes video RAM at 0xB8000,
          ROM/BIOS at 0xF4000 (INT_VECT, BIOS_ROM, default stubs),
          IVT at 0x00000-0x003FF (256 x 4-byte CS:IP entries),
          "system information area" 0x00400-0x00500
          evidence: emu8086 memory.html, custom_memory_map.html, Wikipedia segmentation + interrupts
          confidence: high

Instruction  opcode (1-6 bytes), mnemonic, addressing-mode form,
              operand-resolver fn (reg/rm/imm/seg/rel8/rel16/moffs),
              cycle count (best/worst), length-in-bytes,
              flag-effects mask (which of OF/SF/ZF/AF/PF/CF are set/cleared/unchanged),
              privileged?, prefix-handling (segment-override, REP, LOCK)
              evidence: x86 assembly language Wikipedia, 8086_instruction_set.html, Intel datasheets
              confidence: high

Program    path, format (com | exe | bin | boot),
           entry CS:IP, stack SS:SP,
           size, optional .binf overlay (preset regs + memory bytes),
           optional .debug (source-line -> address map),
           optional .symbol (label -> address map)
           evidence: emu8086 compiler.html (the four directives, .binf/.debug/.symbol files)
           confidence: high

Breakpoint  address (CS:IP), enabled, hit-count (optional)
           evidence: F04, emu8086 emulator.html ("breakpoints" implied by single-step + left-margin click)
           confidence: medium  (count / conditional BPs inferred, not explicitly documented)

SourceMap   line -> address (many-to-one), address -> line (one-to-many),
           label -> address
           evidence: emu8086 compiler.html ("*.debug file has information that enables the emulator
                      select lines of original source code while running the machine code")
           confidence: high

Example     bundled .asm samples (micro-os_loader, micro-os_kernel, writebin, leds, traffic light, etc.)
           evidence: emu8086 compiler.html + SourceForge feature list ("Sample programs and tutorials library")
           confidence: high
```

Relationships: `Cpu8086` owns `Memory`; `Program` references one `Cpu8086` state on load; `Breakpoint` references one `Program` address; `SourceMap` references one `Program`. Examples are `Program` templates.

## Feature matrix

See `features.csv`. Counts: must 12, should 6, could 5, skip 4.

## Out of scope (cannot or should not be cloned)

- **Proprietary installer, bundled examples, and original branding.** emu8086 is "Free Trial" (proprietary, SourceForge). The "Repack" zip in the user's folder contains someone else's serial key — it is not a legitimate license and must not be treated as a source of trade secrets. The clone ships under our own name and our own example programs; we cannot redistribute the original emu8086 examples verbatim.
- **Windows-only distribution story** (CMax20.ocx, registry, COM-based installer). The clone targets cross-platform; the original's installer mechanics are irrelevant.
- **The original "micro-OS" / "virtual peripheral" hardware kit** (LED panels, traffic lights, robotic units, stepper motors). These are bespoke visual demos tied to the proprietary product. Useful as *inspiration* for example programs but not part of the clone's MVP.
- **Original user manual prose** (the .chm / HTML tree shipped inside emu8086). We write our own; we can quote brief structural references (e.g. how `INT 13h` is documented) but the full manual is not ours to ship.

## Size

- Screens 14, flows 8, entities 6. Hard parts: (1) the 8086 prefetch queue + per-instruction cycle accounting, (2) faithful flag-update rules across 100+ mnemonics (esp. shift/rotate corner cases), (3) source-line <-> address debug-map generation across single + multi-pass assembly and macro expansion.
- Size: **M** (a few weeks of focused Rust work). Without cycles/timing accuracy, drops to S.
