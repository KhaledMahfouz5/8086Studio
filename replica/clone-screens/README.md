Clone screenshots go here (one per screen, same viewport as `replica/screens/`).

**Pending a desktop session.** This sandbox has no X server / virtual
framebuffer, so the egui binaries (`studio`, `studio-preview`) can't be run
headless to produce PNGs. On any desktop:

```
cargo run --release --bin studio-preview   # design-system page
cargo run --release --bin studio           # the emulator app
# screenshot each pane at 1440x900 into this directory as SNN.png
python3 imgdiff.py ../screens/emu8086-main.webp S06.png --json > ../diffs/S06.json
python3 parity.py ../features.csv --visual ../diffs/*.json --markdown >> ../parity.md
```

Tracked in `replica/parity.md` under "Layout parity".
