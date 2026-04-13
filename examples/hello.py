"""Minimal hello-world for the dkdc_draw Python bindings.

Requires: `uv run maturin develop` (or `pip install dkdc-draw`).

Run: `uv run python examples/hello.py`
"""

import json
from pathlib import Path

import dkdc_draw

doc_json = dkdc_draw.new_document("hello")
doc = json.loads(doc_json)

doc["elements"] = [
    {
        "type": "Rectangle",
        "id": "box",
        "x": 20.0,
        "y": 20.0,
        "width": 200.0,
        "height": 80.0,
    },
    {
        "type": "Text",
        "id": "label",
        "x": 40.0,
        "y": 50.0,
        "text": "hello, draw",
    },
    {
        "type": "Arrow",
        "id": "arrow",
        "x": 240.0,
        "y": 60.0,
        "points": [{"x": 0.0, "y": 0.0}, {"x": 80.0, "y": 0.0}],
    },
]

doc_json = json.dumps(doc)

Path("hello.svg").write_text(dkdc_draw.export_svg(doc_json))
print("wrote hello.svg")

Path("hello.png").write_bytes(dkdc_draw.export_png(doc_json))
print("wrote hello.png")

out = Path("hello.draw.json")
dkdc_draw.save_document(doc_json, str(out))
print(f"wrote {out}")
