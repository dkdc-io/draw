# gallery

Canonical sample drawings showing off the full `draw-core` API surface. Each
drawing is committed as three artifacts:

- `<slug>.draw.json` — the document, in the native `.draw.json` format (this
  is what you'd open in the app or CLI).
- `<slug>.svg` — exported via `export_svg`.
- `<slug>.png` — exported via `export_png_with_scale(&doc, 2.0)` (retina 2x).

## index

| drawing     | features exercised                                      |
| ----------- | ------------------------------------------------------- |
| flowchart   | rectangles, diamond, ellipse, arrows with arrowheads, text labels |
| sticky      | solid-fill rectangles with rotation, multiline text labels |
| wireframe   | ellipses, dashed strokes, nested frames, diagonal lines |
| sketch      | freedraw curves (sine wave + parametric heart), arrow annotation |
| patterns    | every `FillType` side by side (Solid, Hachure, CrossHatch, None) |

## regenerating

All artifacts are produced by a single Rust example:

```bash
cargo run --example gallery -p dkdc-draw-core
```

It writes into `examples/gallery/` relative to the workspace root. Override
the output directory with `DRAW_GALLERY_OUT=<path>`.

The Python bindings can round-trip any `.draw.json` in this directory:

```bash
uv run python -c '
import json, pathlib, dkdc_draw
doc = pathlib.Path("examples/gallery/flowchart.draw.json").read_text()
pathlib.Path("/tmp/flowchart.svg").write_text(dkdc_draw.export_svg(doc))
'
```

## convention

If you change any drawing in `crates/draw-core/examples/gallery.rs`,
regenerate the artifacts and commit them together. The committed assets are
the gallery — reviewers use the diff on the `.svg` / `.png` to see exactly
how rendering changed.
