# 8086Studio

A clean-room Rust reimplementation of an 8086 (x86, 16-bit) development environment: type assembly, assemble to a `.com` binary, run it through a cycle-emulated CPU with an emulated BIOS, and inspect registers, flags, memory, and single-stepping in a desktop debugger.

The classic flow — **write → assemble → run → inspect** — with zero dependencies on the original source.

## Quick start

```bash
cargo run --release --bin studio
```

Or for a headless build/run:

```bash
cargo build --release --bin studio
./target/release/studio --run examples/hello.asm
```

## What you get

| Component | Crate | Job |
| --- | --- | --- |
| ISA | `studio-isa` | Registers, flags, addressing modes (data, not logic) |
| Assembler | `studio-asm` | Parser + encoder, label resolution, diagnostic errors |
| CPU | `studio-cpu` | Cycle-emulated 8086 core, flags, interrupts, no global state |
| BIOS | `studio-bios` | Emulated INT 10h/21h/24h routines, dispatch |
| FMT | `studio-fmt` | `.com`/`.exe`/`.bin`/`.binf` writers |
| Debug | `studio-debug` | Breakpoints, source map, **8086 stand‑alone disassembler** |
| GUI | `studio-gui` | Desktop editor, code panel, disassembler view, registers, memory, console |

## Examples

```bash
examples/hello.asm   # INT 21h 09h — prints "hello, world"
examples/loops.asm   # LOOP, flags, register decay
examples/add_two_arrays.asm
examples/pushpop.asm
```

## Architecture notes

- 7 independent crates, one job each. No `print!` in library crates, no file I/O in `studio-cpu`, no pixels in `studio-asm`.
- `studio-cpu` is `no_std`-friendly so the GUI can be swapped for a TUI, CLI, or WASM build.
- Test plan: `replica/test-plan.md` (27 cases, 16 automated — all passing).

## License

This project is released under the GNU General Public License v3.0 (GPL-3.0). See `LICENSE` for full terms.
