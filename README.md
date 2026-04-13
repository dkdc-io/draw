# draw

[![GitHub Release](https://img.shields.io/github/v/release/dkdc-io/draw?color=blue)](https://github.com/dkdc-io/draw/releases)
[![PyPI](https://img.shields.io/pypi/v/dkdc-draw?color=blue)](https://pypi.org/project/dkdc-draw/)
[![crates.io](https://img.shields.io/crates/v/dkdc-draw?color=blue)](https://crates.io/crates/dkdc-draw)
[![CI](https://img.shields.io/github/actions/workflow/status/dkdc-io/draw/ci.yml?branch=main&label=CI)](https://github.com/dkdc-io/draw/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-8A2BE2.svg)](https://github.com/dkdc-io/draw/blob/main/LICENSE)

Local-first, sketch-style drawing tool. Excalidraw-inspired, built on Rust with a vanilla JS frontend.

Single binary. No cloud. Your drawings live in `~/.config/draw/drawings/`.

## Features

- **9 tools**: Select, Rectangle, Ellipse, Diamond, Line, Arrow, Pen (freedraw), Text, Eraser
- **Sketch-style fills**: hachure (diagonal lines), cross-hatch, solid, none
- **Full interactions**: drag, resize, multi-select, rubber band, pan (scroll/space+drag), zoom (ctrl+scroll)
- **Undo/redo**: all operations, including batched multi-element changes
- **Copy/paste/duplicate** with proper ID remapping
- **Z-ordering**: bring to front/back/forward/backward
- **Styles**: stroke color/width/dash, fill color/pattern/density, opacity, font
- **Export**: SVG and PNG
- **Document management**: auto-save, dirty indicator, rename, list
- **Desktop app**: native window via webview (same UI, no browser needed)
- **Python bindings**: `import dkdc_draw`
- **Keyboard-driven**: full shortcut set, `?` for help overlay

## Install

### From source (Rust)

```bash
cargo install dkdc-draw --features webapp
```

### From source (Python)

```bash
uv tool install dkdc-draw
```

### Development

```bash
bin/setup    # install dependencies
bin/install  # build and install CLI
```

## Usage

```bash
draw --webapp       # launch web UI
draw --app          # launch desktop app
draw new [name]     # create new drawing
draw open <file>    # open .draw.json file
draw list           # list saved drawings
draw export-svg <file> [-o output.svg]
draw export-png <file> [-o output.png] [--scale 2.0]
```

### Python

```python
import dkdc_draw

doc = dkdc_draw.new_document("sketch")
dkdc_draw.save_document(doc, "sketch.draw.json")
svg = dkdc_draw.export_svg(doc)
```

### Keyboard shortcuts

| Key | Action |
|-----|--------|
| V | Select |
| R | Rectangle |
| O | Ellipse |
| D | Diamond |
| L | Line |
| A | Arrow |
| P | Pen (freedraw) |
| T | Text |
| E | Eraser |
| G | Toggle grid snap |
| ? | Help overlay |
| Ctrl+Z | Undo |
| Ctrl+Shift+Z | Redo |
| Ctrl+S | Save |
| Ctrl+A | Select all |
| Ctrl+D | Duplicate |
| Delete | Delete selected |
| ] / [ | Bring to front / Send to back |

## Architecture

```
crates/
  draw-core/     Document model, serialization, SVG/PNG export, tiny-skia renderer
  draw-cli/      CLI binary (clap)
  draw-webapp/   Axum web server (port 1213) + embedded vanilla JS frontend
  draw-app/      Desktop app (wry webview)
  draw-wasm/     WASM bindings for the renderer
  draw-py/       PyO3 Python bindings
```

## Development

```bash
bin/build    # build all (Rust + Python)
bin/check    # run all checks (format, lint, test)
bin/format   # format all code
bin/test     # run all tests
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Issues and PRs welcome.

## License

MIT
