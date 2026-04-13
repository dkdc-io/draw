# Contributing to draw

Thanks for your interest. draw is a small project; contributions that keep it small are welcome.

## Dev setup

```bash
bin/setup    # install toolchains + wasm-pack + Python deps
bin/build    # build everything (Rust + Python + WASM)
bin/install  # install the CLI
```

macOS or Linux. The desktop app depends on system GTK/WebKit on Linux; on macOS it uses the built-in WebView.

## Workflow

```bash
bin/check    # fmt, clippy, tests — Rust and Python. Run before you push.
bin/format   # apply formatting
bin/test     # tests only
```

CI runs `bin/check` on every PR. Narrower scripts (`bin/check-rs`, `bin/check-py`) exist for iteration but the full `bin/check` is the gate.

## Running it

```bash
draw --webapp    # web UI at http://localhost:1213
draw --app       # desktop window
```

## Layout

See `AGENTS.md` for the crate map. Short version:

- `crates/draw-core`: document model, serialization, SVG/PNG export, renderer
- `crates/draw-cli`: the `draw` binary
- `crates/draw-webapp`: Axum server + embedded vanilla-JS frontend
- `crates/draw-app`: desktop webview wrapper
- `crates/draw-wasm`: renderer compiled to WASM for the browser
- `crates/draw-py`: PyO3 Python bindings
- `py/dkdc_draw`: Python package + type stubs

## PRs

- One logical change per PR. Small and focused beats big and sweeping.
- Commit style: [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `refactor:`, `docs:`, `chore:`, `ci:`).
- Keep `bin/check` green.
- New features should come with tests. Bug fixes should come with a regression test when practical.
- User-visible changes warrant a line in `README.md` (feature list or keyboard table) or `CHANGELOG.md`.

## Reporting bugs

Open an issue at https://github.com/dkdc-io/draw/issues. Include:

- What you did
- What you expected
- What actually happened
- `draw --version`, OS, and whether you hit it in `--webapp` or `--app`

A minimal `.draw.json` reproducing the bug is gold.

## License

By contributing you agree your work is licensed under MIT, the same as the rest of the project.
