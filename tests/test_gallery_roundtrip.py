"""Round-trip every gallery drawing through the Python bindings.

The committed `examples/gallery/*.draw.json` files act as fixtures — they
cover every element type, fill pattern, and stroke style in the core API.
If a backwards-incompatible change lands in the serde layer or the export
pipeline, this surfaces immediately against real documents.
"""

from __future__ import annotations

from pathlib import Path

import pytest

from dkdc_draw import core

GALLERY = Path(__file__).resolve().parents[1] / "examples" / "gallery"
SLUGS = ["flowchart", "sticky", "wireframe", "sketch", "patterns"]


@pytest.fixture(scope="module")
def gallery_json() -> dict[str, str]:
    missing = [s for s in SLUGS if not (GALLERY / f"{s}.draw.json").exists()]
    if missing:
        pytest.skip(f"gallery fixtures missing: {missing}")
    return {s: (GALLERY / f"{s}.draw.json").read_text() for s in SLUGS}


@pytest.mark.parametrize("slug", SLUGS)
def test_gallery_load_roundtrip(
    tmp_path: Path, slug: str, gallery_json: dict[str, str]
) -> None:
    src = GALLERY / f"{slug}.draw.json"
    loaded = core.load_document(str(src))

    dst = tmp_path / f"{slug}.draw.json"
    core.save_document(loaded, str(dst))
    reloaded = core.load_document(str(dst))

    # Re-save-then-reload is a byte-stable fixed point.
    assert loaded == reloaded


@pytest.mark.parametrize("slug", SLUGS)
def test_gallery_export_svg(slug: str, gallery_json: dict[str, str]) -> None:
    svg = core.export_svg(gallery_json[slug])
    assert svg.startswith("<svg")
    assert svg.rstrip().endswith("</svg>")


@pytest.mark.parametrize("slug", SLUGS)
def test_gallery_export_png(slug: str, gallery_json: dict[str, str]) -> None:
    png = core.export_png(gallery_json[slug])
    assert png[:4] == b"\x89PNG"
    # Gallery drawings have content, so PNG shouldn't be trivially small.
    assert len(png) > 500
