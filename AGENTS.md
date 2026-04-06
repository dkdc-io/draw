# draw

Local-first drawing tool.

## Commands

```bash
bin/build          # Build all (Rust + Python)
bin/build-rs       # Build Rust crate
bin/build-py       # Build Python bindings (maturin develop)
bin/check          # Run all checks (format, lint, test)
bin/check-rs       # Rust checks (fmt, clippy, test)
bin/check-py       # Python checks (ruff, ty)
bin/test           # Run all tests
bin/test-rs        # Rust tests
bin/format         # Format all code
bin/install        # Install CLI (Rust + Python)
bin/build-wasm     # Build WASM bindings (wasm-pack)
bin/bump-version   # Bump version (--patch, --minor (default), --major)
```

## Architecture

```
crates/draw-core/       # Core library (document model, serialization, export, renderer)
  src/lib.rs            # Library root
  src/document.rs       # Document struct (top-level container)
  src/element.rs        # Element enum + shape types
  src/style.rs          # StrokeStyle, FillStyle, FontStyle
  src/point.rs          # Point, Bounds, ViewState
  src/history.rs        # Undo/redo stack with batch support
  src/render.rs         # Unified tiny-skia renderer (native + WASM)
  src/export_svg.rs     # SVG export (with hachure fill support)
  src/export_png.rs     # PNG export via resvg
  src/storage.rs        # File I/O (.draw.json)
crates/draw-app/        # Desktop app (wry webview)
  src/lib.rs            # Native window wrapping the webapp
crates/draw-webapp/     # Axum webapp (port 1213)
  src/lib.rs            # Web server, static file embedding
  src/api.rs            # REST API handlers (drawings CRUD, SVG/PNG export)
  frontend/             # Vanilla JS frontend (embedded at compile time)
    index.html          # SPA shell
    style.css           # Dark theme
    theme.js            # Centralized constants (colors, sizes, timing)
    app.js              # Tool state, keyboard shortcuts, document management, undo/redo
    canvas.js           # Canvas2D rendering engine, pan/zoom, shape drawing
    interactions.js     # Pointer events, hit testing, selection, move/resize
    api.js              # REST API client
crates/draw-cli/        # CLI binary (dkdc-draw on crates.io)
  src/main.rs           # Binary entry point
  src/lib.rs            # Re-exports core + run_cli
  src/cli.rs            # CLI (clap) with --app/--webapp flags, subcommands
crates/draw-wasm/       # WASM bindings for the renderer
  src/lib.rs            # DrawEngine: render, hit-test, interactions via wasm-bindgen
crates/draw-py/         # PyO3 bindings (cdylib)
py/dkdc_draw/           # Python wrapper + type stubs
```

Feature flags on `draw-cli`: `app` (pulls in draw-app), `webapp` (pulls in draw-webapp).

## Renderer

The unified tiny-skia renderer (`render.rs`) is the single rendering engine:
- Renders all element types: rect, ellipse, diamond, line, arrow, freedraw, text (placeholder)
- Fill patterns: solid, hachure, crosshatch, none
- Viewport transform with pan/zoom
- Hit testing and selection visuals
- Compiles to both native (desktop) and WASM (browser)
- Text rendering via cosmic-text is planned (currently uses placeholder rectangles)

## File Format

Drawings are saved as `.draw.json` files in `~/.config/draw/drawings/`. The format is a direct JSON serialization of the `Document` struct.

## Keyboard Shortcuts

| Key | Tool |
|-----|------|
| V | Select |
| R | Rectangle |
| O | Ellipse |
| D | Diamond |
| L | Line |
| A | Arrow |
| P | Pen (freedraw) |
| T | Text |
| E | Eraser |
| Ctrl+Z | Undo |
| Ctrl+Shift+Z | Redo |
| Ctrl+S | Save |
| Ctrl+A | Select all |
| Ctrl+D | Duplicate |
| Delete | Delete selected |
| Space+drag | Pan |
| Ctrl+scroll | Zoom |
