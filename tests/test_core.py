"""Unit tests for the `dkdc_draw.core` PyO3 surface.

Covers the canonical flow — build / save / load / export — plus error paths.
Skips `run_cli` since it's an argv-driven CLI harness; structural correctness
is already covered by the Rust integration tests.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

import dkdc_draw
from dkdc_draw import core


def _parse(doc_json: str) -> dict:
    return json.loads(doc_json)


def test_top_level_reexports_wrapper_api() -> None:
    # `dkdc_draw.__init__` re-exports the PyO3 functions so `from dkdc_draw
    # import new_document` works. Smoke-test that the wrapper layer is intact.
    for name in (
        "new_document",
        "load_document",
        "save_document",
        "export_svg",
        "export_png",
        "run_cli",
        "run",
        "main",
    ):
        assert hasattr(dkdc_draw, name), f"dkdc_draw missing {name}"


def test_new_document_returns_valid_json() -> None:
    doc = _parse(core.new_document("my drawing"))
    assert doc["name"] == "my drawing"
    assert doc["elements"] == []
    assert doc["version"] == 1
    assert isinstance(doc["id"], str) and doc["id"]


def test_save_load_roundtrip_empty(tmp_path: Path) -> None:
    original = core.new_document("roundtrip")
    path = tmp_path / "empty.draw.json"

    core.save_document(original, str(path))
    loaded = core.load_document(str(path))

    assert _parse(original) == _parse(loaded)


def test_save_load_roundtrip_with_element(tmp_path: Path) -> None:
    doc = _parse(core.new_document("with element"))
    doc["elements"].append(
        {
            "type": "Rectangle",
            "id": "r1",
            "x": 10.0,
            "y": 20.0,
            "width": 100.0,
            "height": 60.0,
        }
    )
    path = tmp_path / "rect.draw.json"

    core.save_document(json.dumps(doc), str(path))
    loaded = _parse(core.load_document(str(path)))

    assert loaded["elements"][0]["id"] == "r1"
    assert loaded["elements"][0]["type"] == "Rectangle"
    assert loaded["elements"][0]["width"] == 100.0


def test_export_svg_well_formed() -> None:
    doc = core.new_document("svg")
    svg = core.export_svg(doc)
    assert svg.startswith("<svg")
    assert svg.rstrip().endswith("</svg>")


def test_export_png_has_magic_bytes() -> None:
    doc = core.new_document("png")
    png = core.export_png(doc)
    assert png[:4] == b"\x89PNG"


def test_export_png_default_scale_is_2() -> None:
    doc = core.new_document("scale")
    # Default (scale=2.0) should be larger than scale=1.0 for the same doc.
    default_png = core.export_png(doc)
    small_png = core.export_png(doc, 1.0)
    # An empty doc renders to a small image either way, but 2x should still
    # produce a file at least as large as 1x in practice.
    assert len(default_png) >= len(small_png)


def test_export_png_custom_scale() -> None:
    doc = core.new_document("scale custom")
    big = core.export_png(doc, 3.0)
    assert big[:4] == b"\x89PNG"


# ── Error paths ──────────────────────────────────────────────────────


def test_load_nonexistent_raises(tmp_path: Path) -> None:
    with pytest.raises(RuntimeError):
        core.load_document(str(tmp_path / "does-not-exist.draw.json"))


def test_save_invalid_json_raises(tmp_path: Path) -> None:
    with pytest.raises(RuntimeError):
        core.save_document("not valid json", str(tmp_path / "x.draw.json"))


def test_export_svg_invalid_json_raises() -> None:
    with pytest.raises(RuntimeError):
        core.export_svg("")
    with pytest.raises(RuntimeError):
        core.export_svg("not valid json")
    with pytest.raises(RuntimeError):
        core.export_svg("{}")  # valid json, missing Document fields


def test_export_png_invalid_json_raises() -> None:
    with pytest.raises(RuntimeError):
        core.export_png("{ not json")
