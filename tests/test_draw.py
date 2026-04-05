def test_import():
    """Verify the package can be imported."""
    import dkdc_draw

    assert hasattr(dkdc_draw, "run_cli")
    assert hasattr(dkdc_draw, "new_document")
    assert hasattr(dkdc_draw, "export_svg")
