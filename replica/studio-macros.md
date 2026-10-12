# Studio macros — emu8086 .inc compatibility surface

The original emu8086 ships an `emu8086.inc` library file that defines a set
of convenience macros and one-shot procedure generators. An emu8086 user
expects these names to work in their `.asm` source. To keep existing
emu8086 tutorials compiling unchanged in 8086Studio, we re-implement
the **same names and signatures** in our own `studio.inc` file
(bundled at `examples/studio.inc`, included with `include 'studio.inc'`).

**Re-implementation rules (binding):**
- We match the **public macro names and signatures only**.
- We do NOT copy the procedure bodies from `emu8086.inc`. Every
  implementation is written from scratch from the Intel 8086 / IBM PC BIOS
  / DOS INT 21h public specs.
- We may use a different internal encoding (e.g. `PRINT` may call `INT 21h/09h`
  in our impl instead of `INT 10h/0Eh` in the original) — the contract is the
  macro's name, parameters, and observable behaviour.
- The file is installed alongside the bundled examples, **not** auto-included.
  User writes `include 'studio.inc'` explicitly (same as `include 'emu8086.inc'`
  in the original).

## Public macros (no DEFINE_ prefix needed)

| macro | params | semantics | v1? |
| --- | --- | --- | --- |
| `PUTC`    | `char`     | write one char to screen (BIOS INT 10h teletype) | must |
| `PRINT`   | `sdat`     | write null-terminated string at current cursor | must |
| `PRINTN`  | `sdat`     | write null-terminated string + CR/LF | must |
| `CURSOROFF` | —        | hide text-mode cursor (INT 10h AH=01h with CH bit 5 set) | should |
| `CURSORON`  | —        | restore standard text-mode cursor (INT 10h AH=01h, CH=6, CL=7) | should |
| `GOTOXY`  | `col, row` | move text-mode cursor (INT 10h AH=02h, BH=page) | must |

## Procedure generators (require DEFINE_… once)

The original macro library uses `DEFINE_X` to inject a procedure into the
program *once*, on demand. We mirror this — the user opts in per function
so the binary only contains what is used.

| DEFINE_… | expands to | calls | v1? |
| --- | --- | --- | --- |
| `DEFINE_SCAN_NUM`        | `SCAN_NUM` proc        | reads signed integer from stdin into AX/CX:DX | could |
| `DEFINE_PRINT_STRING`    | `PRINT_STRING` proc    | prints `$`-terminated string at DS:DX        | must (used by PRINT) |
| `DEFINE_PTHIS`           | `PTHIS` proc           | prints `-terminated string at CS:[BX]       | could |
| `DEFINE_GET_STRING`      | `GET_STRING` proc      | reads line from stdin into buffer at DS:DX  | could |
| `DEFINE_CLEAR_SCREEN`    | `CLEAR_SCREEN` proc    | INT 10h AH=06h scroll-up window             | should |
| `DEFINE_PRINT_NUM`       | `PRINT_NUM` proc       | prints signed AX as decimal at current cursor | could |
| `DEFINE_PRINT_NUM_UNS`   | `PRINT_NUM_UNS` proc   | prints unsigned AX as decimal at current cursor | could |

## Notes from the source

- `PRINT` and `PRINTN` are *expandable* macros (they emit a local-label
  string at the call site via `LOCAL s_dcl, next_char, printed, skip_dcl`).
  Re-implementation must honour the `LOCAL` directive contract so multiple
  `PRINT` calls in the same procedure don't collide labels.
- `PRINTN` differs from `PRINT` only by appending `13, 10` (`\r\n`).
- `PRINT` in the original writes via `INT 10h/0Eh` (BIOS teletype). Our
  re-implementation writes via `INT 21h/09h` (DOS `$`-terminated) because
  v1 ships only that one DOS write; users wanting the BIOS behaviour can
  use `PUTC` instead. We document this divergence in `studio.inc`.
- `PUTC` is unchanged: BIOS `INT 10h AH=0Eh` teletype.
- `GOTOXY` and `CURSORON`/`CURSOROFF` are unchanged: BIOS `INT 10h` calls.

## What we explicitly do NOT carry over

- The original `.inc`'s procedure bodies (SCAN_NUM internals, the PRINT_NUM
  divide-by-10 routine, the GET_STRING line-editing loop, etc.). All of
  these are written fresh in 8086Studio from public IBM-PC BIOS / DOS
  specs.
- The `DEFINE_*` mechanism is preserved (it's an idiom emu8086 users
  expect), but the bodies are ours.
- Any FASM-specific syntax. Our re-implementation is plain MASM/TASM-style
  Intel syntax so the file works in any 8086 assembler the user pairs with
  8086Studio.
