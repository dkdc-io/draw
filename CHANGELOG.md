# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Runnable hello-world examples in Rust and Python ([#38](https://github.com/dkdc-io/draw/pull/38))
- Integration tests for draw-core public API and arrow Binding round-trip ([#37](https://github.com/dkdc-io/draw/pull/37))
- CONTRIBUTING.md and README badges (Release, PyPI, crates.io, CI, License) ([#36](https://github.com/dkdc-io/draw/pull/36))
- Arrow snap-to-shape connection points (start_binding / end_binding) ([#35](https://github.com/dkdc-io/draw/pull/35))

### Changed
- Codebase simplification pass + UX improvements ([#34](https://github.com/dkdc-io/draw/pull/34))

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

[Unreleased]: https://github.com/dkdc-io/draw/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/dkdc-io/draw/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/dkdc-io/draw/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/dkdc-io/draw/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/dkdc-io/draw/releases/tag/v0.1.0
