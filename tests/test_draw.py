import json
import os
import tempfile

import pytest

try:
    from dkdc_draw import export_svg, load_document, new_document, save_document

    HAS_NATIVE = True
except ImportError:
    HAS_NATIVE = False


def test_import():
    """Verify the package can be imported."""
    import dkdc_draw

    assert hasattr(dkdc_draw, "run_cli")
    assert hasattr(dkdc_draw, "new_document")
    assert hasattr(dkdc_draw, "export_svg")


@pytest.mark.skipif(not HAS_NATIVE, reason="native module not built")
def test_new_document_returns_valid_json():
    """new_document returns valid JSON with expected structure."""
    result = new_document("test")
    doc = json.loads(result)
    assert doc["name"] == "test"
    assert "elements" in doc
    assert isinstance(doc["elements"], list)


@pytest.mark.skipif(not HAS_NATIVE, reason="native module not built")
def test_export_svg_returns_svg_string():
    """export_svg returns an SVG string containing expected tags."""
    doc_json = new_document("svg-test")
    svg = export_svg(doc_json)
    assert "<svg" in svg
    assert "</svg>" in svg


@pytest.mark.skipif(not HAS_NATIVE, reason="native module not built")
def test_save_and_load_roundtrip():
    """save_document + load_document round-trips without data loss."""
    doc_json = new_document("roundtrip")
    with tempfile.TemporaryDirectory() as tmp:
        path = os.path.join(tmp, "test.draw.json")
        save_document(doc_json, path)
        loaded_json = load_document(path)
    assert json.loads(doc_json) == json.loads(loaded_json)


@pytest.mark.skipif(not HAS_NATIVE, reason="native module not built")
def test_export_svg_invalid_json_raises():
    """export_svg raises RuntimeError on invalid JSON input."""
    with pytest.raises(RuntimeError):
        export_svg("")

    with pytest.raises(RuntimeError):
        export_svg("not valid json")
