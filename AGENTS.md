# AGENTS.md

## unix-philosophy covenant (binding for every agent working on this project)

This is a clean-room Rust clone of the 8086 assembler/emulator documented
in `replica/recon.md`. The 7-crate workspace
(`studio-isa`, `studio-asm`, `studio-cpu`, `studio-bios`, `studio-fmt`,
`studio-debug`, `studio-gui`) enforces these rules — they are not
aspirational, they are the architecture:

- **Modularity.** Each crate has one job. The GUI is the only crate that
  knows about pixels, file paths, and user preferences. If a function's
  name contains "and" / "or" / "also", the boundary is wrong.
- **Separation of policy from mechanism.** Lower crates return values;
  upper crates decide what to do with them. No `print!`/`log::info!` in
  library crates. No file I/O in the CPU crate.
- **Simplicity / Parsimony / YAGNI.** No plugin system. No cycle-accurate
  timing. No virtual peripheral kit. Add a thing only when a real user
  needs it. If a crate grows past ~500 lines, it is probably doing two
  jobs.
- **Representation.** Opcode tables, flag-update matrices, mnemonic parse
  tables are *data* (const tables + match), not logic (long if/else
  chains).
- **Composition.** Functions compose: `asm::assemble → cpu::load →
  cpu::step → bios::dispatch_int`. Each returns a value or `Result`.
  No callbacks. No hidden side effects. No global mutable state.
- **Transparency / Repair.** No silent error paths. `StepResult` and
  `AsmError` carry the information the caller needs to show the user a
  real message. No `unwrap()` in library crates; only
  `expect("known invariant: …")` with the invariant spelled out.
- **Silence.** The GUI does not pop "Success!" modals. The assembler does
  not write to stdout. No progress spinners on the happy path.
- **Least Surprise.** Users of the original IDE get the same pane names,
  same file extensions (`.asm`/`.com`/`.exe`/`.bin`/`.boot`), same
  `$`-terminated INT 21h/09h convention, same flag-update quirks
  (INC/DEC leave CF unchanged). The whole point is familiarity.
- **Diversity.** `studio-cpu` is `no_std`-friendly so the GUI can be
  swapped for a TUI, CLI, or WASM build without touching the mechanism.
- **Economy.** Programmer time beats machine time. Write the obvious
  match; add a comment `// OPTIMIZE: only if profiling shows it matters`
  if you must. Never the other way around.

## scope reminder

The repack zip under `replica/` is a cracked release of the original
tool with someone else's serial key. It is **not** a source. Do not
decompile it. Do not use the serial. Do not redistribute the bundled
examples. All example programs and the user manual are written fresh
under our own name.

## recon + architecture

- `replica/recon.md` — the screen/flow/feature recon
- `replica/features.csv` — feature matrix
- `replica/architecture.md` — stack, crate layout, build order
- `replica/schema.rs` — the data model structs (`include!` from v1 skeleton)

Do not start a build milestone that hasn't been declared in
`architecture.md` § Build order. If a milestone proves wrong, edit the
doc first, then the code.
