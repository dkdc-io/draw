def run_cli(argv: list[str]) -> None:
    """Run the draw CLI with the given arguments.

    Args:
        argv: Command-line arguments (including the program name as argv[0]).

    Raises:
        RuntimeError: If the CLI command fails.
    """
    ...

def new_document(name: str) -> str:
    """Create a new empty drawing document and return it as a JSON string.

    Args:
        name: Display name for the new document.

    Returns:
        JSON string representing the document.

    Raises:
        RuntimeError: If serialization fails.
    """
    ...

def load_document(path: str) -> str:
    """Load a drawing document from a .draw.json file and return it as a JSON string.

    Args:
        path: Filesystem path to the .draw.json file.

    Returns:
        JSON string representing the loaded document.

    Raises:
        RuntimeError: If the file cannot be read or parsed.
    """
    ...

def save_document(json: str, path: str) -> None:
    """Save a drawing document (given as a JSON string) to a .draw.json file.

    Args:
        json: JSON string representing the document.
        path: Filesystem path to write to.

    Raises:
        RuntimeError: If the JSON is invalid or the file cannot be written.
    """
    ...

def export_svg(json: str) -> str:
    """Export a drawing document to an SVG string.

    Args:
        json: JSON string representing the document.

    Returns:
        SVG markup string.

    Raises:
        RuntimeError: If the JSON is invalid.
    """
    ...
