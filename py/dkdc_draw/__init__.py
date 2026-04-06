"""Local-first drawing tool — Python bindings for dkdc-draw."""

import sys

from dkdc_draw.core import (
    export_png,
    export_svg,
    load_document,
    new_document,
    run_cli,
    save_document,
)

__all__ = [
    "run",
    "run_cli",
    "main",
    "new_document",
    "load_document",
    "save_document",
    "export_svg",
    "export_png",
]


def run(argv: list[str] | None = None) -> None:
    """Run the draw CLI with the given arguments."""
    if argv is None:
        argv = sys.argv
    try:
        run_cli(argv)
    except KeyboardInterrupt:
        sys.exit(130)


def main() -> None:
    """CLI entry point."""
    run()
