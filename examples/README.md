# Examples — our own, written fresh under our own name

Bundled `.asm` files ship in `examples/`. Each is public-domain, written
fresh from the Intel 8086 / IBM-PC / DOS INT specs. None are copied or
adapted from the original tool's bundled examples.

## v1 ships (10 files)

| file | topic | runs |
| --- | --- | --- |
| `hello.asm`               | INT 21h/09h write $-terminated string                  | ✓ |
| `sum.asm`                 | 16-bit ADD, exit via INT 21h/4Ch                       | ✓ |
| `1_sample.asm`            | INT 21h/02h write one char                            | ✓ |
| `int21.asm`               | four common AH values: 01/02/09/4C                    | ✓ |
| `keybrd.asm`              | INT 16h AH=00h read keystroke                        | ✓ |
| `count_chars.asm`         | walk a string, count bytes                           | ✓ |
| `loops.asm`               | LOOP instruction + CMP/Jcc                           | ✓ |
| `add_two_arrays.asm`      | element-wise sum into third array                   | ✓ |
| `convert_to_upper_case.asm` | ASCII lowercase → uppercase via AND 0DFh        | ✓ |
| `csum.asm`                | 16-bit checksum of a byte array, print hex         | ✓ |

## Deferred (milestone 2+)

| file | topic | blocked on |
| --- | --- | --- |
| `studio.inc` + `include.asm` | macro library (`PRINT`, `PUTC`, `GOTOXY`, `CURSORON/OFF`, `DEFINE_*` family) | macro preprocessor in `studio-asm` |

The macro library is the only deferred item from the architecture's
v1 must-have list. Spec at `replica/studio-macros.md`. Implementation
lands with the milestone-2 assembler expansion.

## Tier 2 / 3 (out of v1 scope)

Per `replica/examples-todo.md`: 30+ more examples planned for later
milestones (BCD arithmetic, far calls, sort/search, INT 13h disk I/O,
`micro-os_*` kernels). Deferred until the underlying mnemonics and BIOS
stubs ship.
