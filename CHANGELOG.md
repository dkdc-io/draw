# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-04-13

**Breaking**: the `dkdc-draw-core` public API surface is tightened. Unused `pub` items in the `geometry` module and `Bounds::from_points` are now `pub(crate)`. See migration notes at the end of this entry.

### Added
- Sample gallery: 5 canonical drawings exercising every element type and fill pattern, committed as `.draw.json` + `.svg` + `.png` under `examples/gallery/` ([#41](https://github.com/dkdc-io/draw/pull/41))
- Python test coverage grown from 5 to 27 tests across `tests/test_core.py` and `tests/test_gallery_roundtrip.py` ([#43](https://github.com/dkdc-io/draw/pull/43))
- Runnable hello-world examples in Rust and Python ([#38](https://github.com/dkdc-io/draw/pull/38))
- Integration tests for draw-core public API and arrow Binding round-trip ([#37](https://github.com/dkdc-io/draw/pull/37))
- CONTRIBUTING.md and README badges (Release, PyPI, crates.io, CI, License) ([#36](https://github.com/dkdc-io/draw/pull/36))
- Arrow snap-to-shape connection points (start_binding / end_binding) ([#35](https://github.com/dkdc-io/draw/pull/35))
- CHANGELOG.md following Keep a Changelog ([#39](https://github.com/dkdc-io/draw/pull/39))

### Changed
- **Breaking**: demoted unused items in `dkdc-draw-core` from `pub` to `pub(crate)` ([#42](https://github.com/dkdc-io/draw/pull/42)):
  - `geometry::ARROWHEAD_LENGTH`, `ARROWHEAD_ANGLE`, `HACHURE_LINE_WIDTH`
  - `geometry::normalize_bounds`, `compute_arrowhead`, `generate_hachure_lines`
  - `geometry::ArrowheadPoints`, `HachureLine` (structs + fields)
  - `point::Bounds::from_points`
- Adopted surgical clippy pedantic opt-ins workspace-wide; no behavior change ([#40](https://github.com/dkdc-io/draw/pull/40))
- Codebase simplification pass + UX improvements ([#34](https://github.com/dkdc-io/draw/pull/34))

### Migration notes for 0.2.1 → 0.3.0

`Point::distance_to` and `Bounds::intersects` remain `pub` and are the recommended geometry primitives for external callers. If you were reaching into the demoted helpers directly, the intended replacements are:

- `geometry::normalize_bounds` → inline: `let (x, y, w, h) = (min(x1, x2), min(y1, y2), (x1 - x2).abs(), (y1 - y2).abs())`
- `geometry::compute_arrowhead` / `ArrowheadPoints` → no public alternative yet; open an issue if you need this — happy to carve out a stable shape for it.
- `geometry::generate_hachure_lines` / `HachureLine` → no public alternative yet; same as above.
- `Bounds::from_points` → construct via `Bounds::new(x, y, w, h)` from your own min/max reduction.

## [0.2.1] - 2026-04-07

### Added
- Python package now ships with both webapp and desktop app included ([#33](https://github.com/dkdc-io/draw/pull/33))

## [0.2.0] - 2026-04-07

### Added
- Webapp included in the Python package ([#31](https://github.com/dkdc-io/draw/pull/31))

### Fixed
- `bin/setup` now installs `wasm-pack` ([#30](https://github.com/dkdc-io/draw/pull/30))
- `build.rs` auto-builds WASM artifacts for the webapp ([#29](https://github.com/dkdc-io/draw/pull/29))

## [0.1.1] - 2026-04-06

First tagged release published to crates.io and PyPI.

### Added
- `export_png` Python binding ([#19](https://github.com/dkdc-io/draw/pull/19))
- Release workflows for crates.io, PyPI, and GitHub ([#17](https://github.com/dkdc-io/draw/pull/17))

### Fixed
- Removed webapp feature from PyO3 build dependency ([#27](https://github.com/dkdc-io/draw/pull/27))
- Reverted WASM artifacts; use workspace `default-members` instead ([#26](https://github.com/dkdc-io/draw/pull/26))
- Committed WASM artifacts for fresh-clone builds ([#25](https://github.com/dkdc-io/draw/pull/25))
- Added `uuid` js feature for WASM; include frontend in webapp package ([#24](https://github.com/dkdc-io/draw/pull/24))
- Publish all crates needed for dkdc-draw ([#23](https://github.com/dkdc-io/draw/pull/23))
- Reconciled crate publish list ([#22](https://github.com/dkdc-io/draw/pull/22))
- Added `draw-wasm` to `bump-version` script ([#20](https://github.com/dkdc-io/draw/pull/20))
- Added version specifiers for internal dependencies ([#16](https://github.com/dkdc-io/draw/pull/16))

### Changed
- Improved Python type stubs and added functional tests ([#18](https://github.com/dkdc-io/draw/pull/18))
- Documented undo history bug; removed dead code ([#21](https://github.com/dkdc-io/draw/pull/21))

## [0.1.0] - 2026-04-06

Initial release. Core drawing tool with CLI, webapp, and desktop app.

### Added
- Document model with 7 element types: Rectangle, Ellipse, Diamond, Line, Arrow, FreeDraw, Text
- Sketch-style fills: hachure, crosshatch, solid, none
- Full interaction set: drag, resize, multi-select, rubber band, pan, zoom
- Undo/redo with batched multi-element changes
- Copy/paste/duplicate with ID remapping; z-ordering
- SVG and PNG export
- Local `.draw.json` file format under `~/.config/draw/drawings/`
- Desktop app via webview; webapp on port 1213; vanilla JS frontend
- WASM renderer bindings
- Python bindings via PyO3
- Round-trip, bounds, SVG export, and arrowhead tests ([#13](https://github.com/dkdc-io/draw/pull/13))
- Arrowhead constants unified between preview and SVG export ([#1](https://github.com/dkdc-io/draw/pull/1))

### Fixed
- Start arrowheads now render in SVG export ([#4](https://github.com/dkdc-io/draw/pull/4))
- Excluded `draw-app` and `draw-webapp` from workspace CI checks ([#6](https://github.com/dkdc-io/draw/pull/6), [#10](https://github.com/dkdc-io/draw/pull/10))
- Skipped maturin build in CI ([#12](https://github.com/dkdc-io/draw/pull/12))
- Missing `chrono` dependency on WASM crate ([#9](https://github.com/dkdc-io/draw/pull/9))

[Unreleased]: https://github.com/dkdc-io/draw/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/dkdc-io/draw/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/dkdc-io/draw/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/dkdc-io/draw/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/dkdc-io/draw/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/dkdc-io/draw/releases/tag/v0.1.0
