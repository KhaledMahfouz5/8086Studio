# Components — 8086Studio

Every UI primitive in one place. Each block lists the variants, sizes, states,
the token roles it consumes, the keyboard/screen-reader contract, and the
screens it appears on (S-IDs from `replica/recon.md`). States include the ones
the recon saw and the ones the spec needs anyway: focus, disabled, error, empty.

The tokens live in `replica/design/tokens.json`. Nothing in this document
uses raw hex. Names stay; values change when `replica-brand` runs.

---

## Button

```
variants   primary | secondary | ghost | danger
sizes      sm 24px, md 30px (toolbar default), lg 40px (main actions)
states     default, hover, active, focus-visible (2px accent ring),
           disabled (50% opacity, no hover), loading (spinner, label kept)
tokens     bg accent|surface|border|none,
           text on-accent|text|text-muted|danger,
           radius md, padding 8/12, font button (13/600)
a11y       Real click target; Enter/Space activate; loading state keeps label
           for screen readers ("Compiling…"); focus ring visible on Tab.
used on    S01 toolbar (New/Open/Save/Compile/Emulate), S04, S06 (Run/Step/Stop/Reset),
           S07 (OK/Cancel), S09 (Create/Delete)
```

## ToolButton (icon-only)

```
variants   default | toggled (pressed)
sizes      28px square (matches toolbar density from the original)
states     default, hover, active, focus-visible, disabled
tokens     bg surface-raised, fg text-muted (default) / accent (toggled),
           radius sm
a11y       aria-label required; tooltip on hover shows full verb
           ("Run (F5)", "Single Step (F8)").
used on    S06 toolbar: file / math / debug / view / external / virtual drives / help,
           S06 sub-toolbar: load / reload / step back / single step / run
```

## Editor (TextEdit + gutter + syntax highlight)

```
variants   com | exe | bin | boot template headers (preset lines, dimmed)
sizes      full-pane; monospace 13/18 (mono-base)
states     empty, dirty (title shows *), saved, readonly (after HLT/error),
           search-active (highlight matches), line-current (highlight line)
tokens     bg bg, fg text, line-number fg text-muted on bg-faint,
           current-line bg surface-raised,
           keywords syntax-keyword, directives syntax-directive,
           comments syntax-comment, strings syntax-string,
           labels syntax-label, numbers syntax-number
a11y       TextEdit is a real <input>; Tab inserts a tab (v1), no focus traps;
           ctrl+F focuses the find bar; ctrl+G focuses go-to-line.
used on    S01
```

## Line gutter

```
variants   none (no gutter) | line-numbers | line-numbers + breakpoints
sizes      28px (3-digit line) / 36px (4-digit + breakpoint hit)
states     normal, breakpoint-set (dot in danger color),
           breakpoint-hit (dot pulses in warning color),
           current-line (slight indent or arrow)
tokens     bg bg, fg text-muted, breakpoint-dot, breakpoint-hit
a11y       Click sets/toggles BP; right-click clears. Tooltip shows "BP @ CS:IP = 0100h".
used on    S01, S06
```

## Register cell

```
variants   16-bit (AX/BX/CX/DX/SP/BP/SI/DI),
           8-bit (AL/CL/DL/BL/AH/CH/DH/BH),
           segment (CS/DS/ES/SS)
sizes      two-letter name + 4-hex-digit value, ~64px wide, 18px tall
states     idle, focused (editable, accent border),
           changed-since-step (brief flash in warning color, 140ms),
           read-only (segment writes locked; show "RO" affordance on hover)
tokens     bg surface, fg text, focused bg surface-raised + accent border,
           changed flash warning
a11y       Enter or click enters edit mode; type hex digits; Enter commits.
           Esc cancels. Tab to next cell. Screen reader: "AX, 1234 hex".
used on    S06, S07 (Extended Viewer)
```

## Flag bit

```
variants   OF | DF | IF | TF | SF | ZF | AF | PF | CF
sizes      22px square cell, two-letter label below
states     clear (0, dim), set (1, accent dot)
tokens     bg surface, fg text-muted (clear), flag-set (set),
           focus-visible 2px accent ring
a11y       Click toggles. Space toggles when focused.
           Screen reader: "Zero flag, set" / "Carry flag, clear".
used on    S06, S07
```

## Memory cell

```
variants   segmented view (CS:/DS:/ES:/SS: prefix), flat-physical view
sizes     16 bytes per row, 24 rows visible, monospace 11/14 (mono-xs)
states     idle, hovered (subtle bg highlight),
           edited-on-click (cursor row pulses in memory-cursor color),
           breakpoint-set (red dot at row address)
tokens     bg memory-bg, fg memory-fg, current memory-cursor,
           address column fg text-muted, ASCII column fg memory-fg
a11y       Click selects row, double-click opens Extended Viewer.
           Arrow keys move cell; Enter opens viewer.
           Screen reader: "DS:0100, byte 41 hex, ASCII 'A'".
used on    S06
```

## Stack panel

```
variants   SS:SP view (top-down growth)
sizes     16 bytes per row, 12 rows visible, monospace 11/14
states     idle, underflow (red flash + warning on the pane header),
           edited-on-click (cursor pulses in memory-cursor color)
tokens     bg memory-bg, fg memory-fg, address column text-muted,
           top-of-stack pointer memory-cursor
a11y       Same as Memory cell.
used on    S06
```

## Code panel

```
variants   CS:IP | far-call target view
sizes     address column + hex bytes column + mnemonic column,
           ~120 char wide, monospace 11/14
states     idle, current-line (bg code-current-line),
           breakpoint-set (red dot in left margin),
           breakpoint-hit (warning pulse),
           hover (shows byte tooltip with offset)
tokens     bg bg, fg text-muted (address), fg text (mnemonic),
           current-line bg code-current-line,
           breakpoint-dot, breakpoint-hit
a11y       Read-only. Click in margin toggles BP. Current line is
           announced as "line 42, MOV AX, 5".
used on    S06, S13 (Disassembler view)
```

## Breakpoint dot

```
variants   none | set
sizes      8px circle in the gutter
states     set, hit (pulses warning color for 800ms after a hit)
tokens     color breakpoint-dot (set), breakpoint-hit (hit)
used on    S01 (editor gutter), S06 (code panel margin)
```

## Extended Viewer (modal)

```
variants   register view | memory view
sizes     ~320px wide, auto-height
states     read-only, editable (one field focused),
           invalid-input (red border + danger helper text below)
tokens     bg surface-raised, border border, fg text, focused border accent,
           helper text text-muted (normal) / danger (invalid)
a11y       Focus traps inside the modal; Esc closes; Enter commits edit.
           Screen reader announces value in all formats on open:
           "AX, hex 1234, decimal 4660, binary 0001 0010 0011 0100".
used on    S07
```

## Toolbar (top, dense)

```
variants   main toolbar | emulator sub-toolbar
sizes     24px-tall buttons, 4px gap
states    default, hovered, any-active-child
tokens    bg surface, fg text-muted (inactive) / text (active),
          border between buttons border
a11y      Tab order follows visual order; arrow keys move between buttons.
used on    S01, S06
```

## Pane-tab strip (bottom of emulator)

```
variants   screen | source | reset | aux | vars | debug | stack | flags
sizes     one tab per pane, 80px wide, 24px tall
states    default, active, focused
tokens    bg surface, fg text-muted (default) / text (active),
          active bottom-border accent
a11y      Number keys 1..8 select tabs; arrow keys cycle; Tab walks.
used on    S06
```

## Status bar

```
variants   editor status | emulator status
sizes     20px tall, full window width, three sections (left/center/right)
states    idle, busy (loading… spinner in right section),
          error (center shows last error in danger color)
tokens    bg surface, fg text-muted (idle) / danger (error)
a11y      Updates announced via aria-live="polite" on change.
used on    S01, S06
```

## Compile-errors panel

```
variants   modal (blocking) | inline (in status bar)
sizes     480px wide, max-height 60vh, scrollable
states    empty, single-error, multi-error (sorted by line number),
          jump-to-line (double-click on error)
tokens    bg surface-raised, fg text, line number text-muted,
          error text danger, border border
a11y      Errors are listed, not dialogued. Tab order matches visual.
          Double-click jumps focus to that line in the editor.
used on    S05
```

## Find bar

```
variants   find | find-and-replace
sizes     top of editor, ~360px wide
states    hidden, visible-empty, visible-with-query, no-match (danger helper)
tokens    bg surface-raised, border border, focused border accent,
          helper text text-muted (normal) / danger (no-match)
a11y      Esc closes; Enter finds next; Shift+Enter finds prev;
          screen reader: "Find, edit, 2 of 7".
used on    S01
```

## Virtual drive widget

```
variants   mounted | empty | boot-source
sizes     96px square floppy icon + label below
states    mounted, write-protected, boot-from-here (accent border),
          not-present (dim, "—" placeholder)
tokens    bg surface, fg text-muted (default) / text (mounted),
          boot indicator accent
a11y      Right-click menu: Mount / Unmount / Boot from here / Write 512 bytes.
used on    S06, S09
```

## Virtual screen (the 80x25 console)

```
variants   text-mode 03h (80x25 16-color) | text-mode 00h (40x25) |
           graphics 13h (320x200 256-color)
sizes     fixed pixel size per mode (scale via host DPI)
states    rendering, paused-on-step (cursor blinks at INT 10h position),
          blank (after HLT)
tokens    bg black (#000000), fg default 7 (light grey),
          attribute byte drives fg/bg colors from a 16-color palette
a11y      Pure visual; screen reader announces INT 21h/02h and INT 21h/09h
          output as it happens.
used on    S06
```

## Loading spinner

```
variants   none | circle (default) | dots
sizes     16px
tokens    color accent (active) / text-muted (idle)
a11y      aria-busy=true on the parent container; spinner is decorative.
used on    S04 (during Compile), S06 (during "loading example")
```

## Toast / snackbar (used only for genuine errors, per the unix covenant)

```
variants   error (danger) | warning (warning) | success (success)
sizes     full-width, 32px tall, slides up from status bar
states    hidden, shown, expiring (fades over 240ms)
duration  error 6s, warning 4s, success 2s; click to dismiss
tokens    bg surface-raised, fg text, border-left 4px in role color
a11y      aria-live="assertive" for error, "polite" for warning/success.
used on    global, anchored to bottom of any pane
```

---

## Composition rules (binding)

- **One widget, one job.** A `Button` does not know it is the "Run" button.
  The caller passes label + on-click + variant.
- **No `and`/`or` in names.** `ToolButton` is icon-only; `Button` has a label.
  They don't merge.
- **States are data, not booleans.** A register cell carries
  `RegCellState { idle, Focused, ChangedSinceStep }` — it doesn't carry three
  bools and let the caller decide.
- **Theme lives in `theme.rs`.** No widget file imports a `Color32`
  directly. They ask the `Tokens` struct.
- **Empty is a real state.** Every list-shaped widget renders its empty
  state explicitly. No silent blanks.

## Component inventory at a glance

v1 ships the ones marked **shipped**; the rest are documented so the
spec is complete, but their `widgets/<name>.rs` files do not exist yet.
The drift detector at the bottom of this doc is the source of truth for
the shipped set; running it is part of `cargo test`.

| Component | Files | Used on screens | Status |
| --- | --- | --- | --- |
| Button          | `widgets/button.rs`           | S01, S04, S05, S06, S07, S09 | shipped |
| ToolButton      | `widgets/tool_button.rs`      | S06, S09                     | shipped |
| Editor          | `widgets/editor.rs`           | S01                          | shipped (read-only view) |
| LineGutter      | `widgets/line_gutter.rs`      | S01, S06                     | shipped |
| RegisterCell    | `widgets/register_cell.rs`    | S06, S07                     | shipped |
| FlagBit         | `widgets/flag_bit.rs`         | S06, S07                     | shipped |
| MemoryCell      | `widgets/memory_cell.rs`      | S06                          | shipped |
| StackPanel      | `widgets/stack_panel.rs`      | S06                          | deferred (compose MemoryCell in screen) |
| CodePanel       | `widgets/code_panel.rs`       | S06, S13                     | shipped |
| BreakpointDot   | `widgets/breakpoint_dot.rs`   | S01, S06                     | shipped |
| ExtendedViewer  | `widgets/extended_viewer.rs`  | S07                          | deferred |
| Toolbar         | `widgets/toolbar.rs`          | S01, S06                     | shipped |
| PaneTabStrip    | `widgets/pane_tab_strip.rs`   | S06                          | shipped |
| StatusBar       | `widgets/status_bar.rs`       | S01, S06                     | shipped |
| ErrorsPanel     | `widgets/errors_panel.rs`     | S05                          | deferred |
| FindBar         | `widgets/find_bar.rs`         | S01                          | deferred |
| DriveWidget     | `widgets/drive_widget.rs`     | S06, S09                     | deferred |
| VirtualScreen   | `widgets/virtual_screen.rs`   | S06                          | deferred |
| Spinner         | `widgets/spinner.rs`          | S04, S06                     | deferred |
| Toast           | `widgets/toast.rs`            | global                       | deferred (unix covenant: no success modals) |

**v1 coverage: 11/19 widgets shipped.** Deferred items are *not* missing
on purpose — they wait for the v1 emulator + assembler milestones to land
so their call sites exist.

The `studio-preview` binary renders every shipped widget in every
documented state in a single window. This is the "screenshot the page"
step from the skill. Run it with:

```
cargo run --release --bin studio-preview
```
