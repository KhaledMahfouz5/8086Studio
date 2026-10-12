# Examples — our own, written fresh under our own name

The original emu8086 ships ~85 bundled `.asm` examples. **We do not ship
those.** The list below is what 8086Studio will write, by topic, derived
from the filenames in `replica/screens/payload-filenames.txt` for naming
consistency only. We do not look at the bundled source code.

Conventions:
- All programs end with `.asm` extension.
- `.com` template (`org 100h`) is the v1 default; `.exe` and `.bin` come later.
- Every program compiles cleanly with our `emu8086-asm` and runs in
  8086Studio's emulator with INT 21h/INT 10h stubs.

## Tier 1 — must ship with v1

| file | purpose | reuses macros |
| --- | --- | --- |
| `hello.asm`            | "Hello, World!" via INT 21h AH=09h, $-terminated | — |
| `HelloWorld.asm`       | same, alternative framing | — |
| `sum.asm`              | sum of two 16-bit numbers, result in AX         | `PRINT_NUM` |
| `1_sample.asm`         | first lesson: registers + MOV + INT 21h/02h     | `PUTC` |
| `add_two_arrays.asm`   | sum two byte arrays, length in CX               | — |
| `loops.asm`            | `MOV CX, n` + `LOOP` body                       | — |
| `int21.asm`            | demo of INT 21h AH=01h (read char), 02h (write char), 09h (write string), 4Ch (exit) | `PUTC PRINT` |
| `keybrd.asm`           | INT 16h AH=00h read keystroke; echo via INT 21h | `PUTC` |
| `count_chars.asm`      | count chars in a $-terminated string            | — |
| `convert_to_upper_case.asm` | lowercase string → uppercase via AND 0xDF  | `PRINT` |

## Tier 2 — should, ship when `.exe` and `.bin` output + INT 13h land

| file | purpose | reuses macros |
| --- | --- | --- |
| `calc.asm`             | expression parser + evaluator (state machine)    | `PRINT_NUM` |
| `factorial.asm`        | recursive factorial via CALL/RET               | `PRINT_NUM` |
| `palindrome.asm`       | palindrome check on a string                   | `PRINT PRINTN` |
| `fahrenheit.asm`       | °F → °C conversion with table lookup           | `PRINT_NUM` |
| `bin2dec.asm`          | binary string → decimal number                 | `PRINT_NUM` |
| `convert_string_number_to_binary.asm` | decimal string → binary representation | `PRINT_NUM` |
| `convert_string_to_packed_bcd.asm`     | decimal string → packed BCD form | — |
| `bcd_aaa.asm`          | AAA after addition; show AF/CF updates         | `PRINT_NUM` |
| `bcd_aas.asm`          | AAS after subtraction; show AF/CF updates       | `PRINT_NUM` |
| `double_word.asm`      | 32-bit arithmetic via DX:AX + CWD              | `PRINT_NUM` |
| `param.asm`            | procedure parameters via stack (CALL/RET)       | `PRINT` |
| `paramexe.asm`         | same, EXE template                              | — |
| `far_call.asm`         | inter-segment CALL; CS:IP pushed               | — |
| `far_call-2.asm`       | far-call with explicit far-return               | — |
| `far_call_advanced.asm`| far-call + LEA + multiple segments             | — |
| `include.asm`          | demo `%include` of `studio.inc` macros         | `PRINT PRINTN` |
| `fasm_compatibility.asm` | side-by-side: works in both NASM and 8086Studio | `PRINT` |
| `cmpsb.asm`            | byte-by-byte string compare via CMPSB + REP    | `PRINT` |
| `cmpsw.asm`            | word-by-word compare via CMPSW                 | `PRINT` |
| `int10_13.asm`         | INT 10h AH=13h write-string at ES:BP           | — |
| `pixel.asm`            | INT 10h AH=0Ch set pixel in 320x200 mode 13h   | `GOTOXY` |
| `pages.asm`            | INT 10h AH=05h select video page               | `GOTOXY` |
| `colors.asm`           | attribute byte (16 colors) write via INT 10h   | `GOTOXY PUTC` |
| `attrib.asm`           | read/write attribute byte at cursor via INT 10h AH=08h/09h | `GOTOXY` |
| `LED_display_test.asm` | pattern write to virtual LED device via OUT    | — |
| `stop_on_condition.asm`| conditional breakpoint demo                    | — |
| `encrypt_subrotine.asm`| XOR-based string encryption                     | `PRINT GET_STRING` |
| `file-operations.asm`  | INT 21h AH=3Ch/3Dh/3Eh/40h/3Fh/42h create/open/write/read/seek/close | `PRINT` |
| `datefile.asm`         | INT 21h AH=2Ah/2Ch write date+time to file     | `PRINT` |
| `exetest.asm`          | EXE template demo (.exe assembly)               | — |
| `bintest.asm`          | BIN template demo with .binf preset state      | — |
| `writebin.asm`         | write a `.bin` to the virtual floppy boot sector | — |

## Tier 3 — could, ship when virtual peripherals or mouse support lands

| file | purpose | depends on |
| --- | --- | --- |
| `traffic.asm`          | traffic-light state machine via OUT port 4     | `OUT 4` device |
| `stepper.asm`          | stepper-motor rotation via OUT port 7          | `OUT 7` device |
| `mouse.asm`            | INT 33h AX=0003h poll mouse position           | `PRINT` |
| `mouse2.asm`           | mouse in graphics mode 13h                      | `OUT` device + graphics |
| `micro-os_loader.asm`  | boot sector that loads a kernel from floppy    | `.boot` + INT 13h + floppy |
| `micro-os_kernel.asm`  | simple kernel that prints and reads kbd        | `INT 21h AH=09h` minimal |
| `0_sample_add_huge_numbers.asm` | 32-bit add with carry | — |
| `0_sample_vga_graphics.asm`     | 320x200 mode 13h demo | graphics |
| `no-close.asm`         | sample that intentionally never exits          | — |
| `2_sample.asm`–`5_sample.asm` | progressive tutorial lessons | — |
| `Custom_Interrupt.asm` | user-installed ISR via INT 21h AH=25h          | — |
| `Count_Key_Presses.asm`| key-press tally via INT 16h                     | `PRINT_NUM` |
| `frmLed.bas` (etc.)    | (skipped — original device shells, VB6 source) | — |

## Naming convention

- lowercase-with-underscores for assembly source. The original uses a mix
  of camelCase and lowercase; we normalize to lowercase-with-underscores
  to keep `ls` output sorted by topic.
- Two files that did the same job in the original (`hello.asm` vs
  `HelloWorld.asm`) are merged into one (`hello.asm`). We don't ship two
  near-duplicates.
- Numbered tutorial lessons (`1_sample.asm` … `5_sample.asm`) become
  `01_intro.asm`, `02_registers.asm`, etc., in a separate `tutorials/`
  subdirectory we add later.

## Tier-4 examples — out of scope (skip)

Anything that depends on the original's proprietary virtual peripheral kit
(`Robot.exe`, `Traffic_Lights.exe`, etc.) is **out of scope** for v1.
We ship a clean-room I/O-port stub layer instead (see `emu8086-bios`),
and users can write their own device programs against `OUT n` once that
stub layer exists.
