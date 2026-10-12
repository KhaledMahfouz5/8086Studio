# Bugs — 8086Studio

Only reproduced issues are listed. "To check" items go at the bottom.

---

### BUG-001: `mov ax, 0xGG` reports "unknown label" instead of "bad numeric literal"

- Severity: S4 (cosmetic — confusing error message, no wrong behaviour)
- Flow / case: F01-N4
- Screen: S05 (compile errors panel)
- Build: v0.1.0 (milestone 0–1)

Steps
1. Type `mov ax, 0xGG` in the editor.
2. Press Compile.

Expected: error message mentioning "bad numeric literal" or "invalid hex value" — the `0x` prefix is an unambiguous number marker.
Actual: error says `unknown label or constant `0XGG`` — the parser falls through to label lookup after `parse_imm` fails.
Evidence: `crates/emu8086-bios/tests/e2e_errors.rs::f01_n4_bad_numeric_literal_reports_value`
Suspected cause: `parse_operand` tries `parse_imm` first; on failure it falls through to `Operand::Label` without checking whether the token looked like a number (`0x…`, `…h`, digits).
Status: open

---

## To check (not yet reproduced)

- `line_for_ip` in the GUI may misattribute the current-line highlight when an
  instruction spans multiple source addresses (multi-byte labels). Needs a
  desktop session to verify visually.
- Short-branch out-of-range message says `use a near form (TODO)` — the
  "near" form is not implemented yet. This is a known limitation, not a bug.
- `LOOP` with `CX=0` wraps to 65535 and loops 65536 times. This is real 8086
  behaviour (Intel manual) but may surprise users. Document, don't change.
