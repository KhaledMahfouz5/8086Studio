# TODO – 8086Studio missing features

This file tracks the features that are present in the original **emu8086** but are not yet implemented (or only partially) in our Rust clone. Each line shows the priority, a short description, and the current status.

| Priority | Feature | Status |
|---|---|---|
| **MUST** | Syntax‑highlighted source editor (advanced coloring) | **Done** – enabled via `egui::TextEdit::code_editor()` (basic highlighting). |
| **MUST** | Disassembler view (stand‑alone screen) | Pending – need UI pane to load arbitrary binaries and display address/hex/mnemonic. |
| **MUST** | `.exe`/`.bin`/`.boot` output writers (`#make_*#` directives) | Pending – `studio_fmt::write_exe` stub to be added. |
| **MUST** | Virtual 80×25 text‑mode screen (INT 10h output) | Pending – implement screen buffer and render in GUI. |
| **MUST** | Full BIOS INT family (INT 21h 01/02/0A, INT 16h, INT 1Ah, etc.) | Partially done – only INT 21h 09h and 4Ch. |
| **MUST** | Compile‑error panel with click‑to‑jump‑to‑line | Pending – UI interaction to map error line to editor cursor. |
| **MUST** | Register/flag editing inline | Pending – allow editing registers/flags directly in the UI. |
| **MUST** | Stack panel (SS:SP) live view | Pending – display current stack pointer and memory region. |
| **MUST** | Extended viewer ASCII/binary forms | Partially done – hex/dec/signed present, ASCII pending. |
| **SHOULD** | Step‑back (history snapshot) | Pending – implement CPU snapshot/restore. |
| **SHOULD** | Virtual floppy drives & INT 13h | Pending – floppy image handling. |
| **SHOULD** | Virtual peripherals (LED, traffic‑light, motor) | Pending – expose `OUT n` stubs and example demos. |
| **COULD** | Syntax‑highlight improvements (custom assembly lexer) | Future work. |
| **COULD** | ALU operation visualization diagram | Future work. |
| **COULD** | `.lst` source listing file | Future work. |

*Each commit will target one **MUST** or **SHOULD** item, updating the status accordingly.*