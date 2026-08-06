from pathlib import Path


def read_app_source(root: Path) -> str:
    """Return the complete WASM app module after its physical split."""
    sources = [root / "src/app.rs", *sorted((root / "src/app").glob("*.rs"))]
    return "\n".join(path.read_text(encoding="utf-8") for path in sources)
