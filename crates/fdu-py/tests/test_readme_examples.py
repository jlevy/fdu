"""The README Python examples run against the installed package.

The package README is the PyPI page and the repository README is the crates.io page, so
their examples are the first code a reader copies. Nothing ran them, and both kept calling
an ``AnalysisProfile`` that was gone once the content axis became an analyzer set.
Running each block as written is cheaper than keeping a second copy of it in a test.
"""

from __future__ import annotations

import re
from pathlib import Path

import pytest

REPOSITORY = Path(__file__).resolve().parents[3]
READMES = (REPOSITORY / "README.md", REPOSITORY / "crates" / "fdu-py" / "README.md")
PYTHON_BLOCK = re.compile(r"^```python\n(.*?)^```$", re.DOTALL | re.MULTILINE)
#: The one placeholder the examples use for a tree; the test supplies a real one.
PLACEHOLDER_ROOT = 'Path("/path/to/tree")'


def test_every_readme_python_example_runs(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    tree = tmp_path / "tree"
    (tree / "src").mkdir(parents=True)
    (tree / "src" / "main.py").write_text("print('hello')\n", encoding="utf-8")
    (tree / "notes.md").write_text("# Notes\n\nA few words to count.\n", encoding="utf-8")
    # Examples that name `Path(".")` open the tree through the working directory, and any
    # snapshot they write lands here instead of in the user's cache.
    monkeypatch.chdir(tree)
    monkeypatch.setenv("XDG_CACHE_HOME", str(tmp_path / "cache"))

    ran = 0
    for readme in READMES:
        text = readme.read_text(encoding="utf-8")
        for block in PYTHON_BLOCK.finditer(text):
            line = text.count("\n", 0, block.start()) + 2
            where = f"{readme.relative_to(REPOSITORY)}:{line}"
            source = block.group(1).replace(PLACEHOLDER_ROOT, 'Path(".")')
            try:
                exec(compile(source, where, "exec"), {"__name__": "readme_example"})
            except Exception as error:
                error.add_note(f"the Python example starting at {where} no longer runs")
                raise
            ran += 1
    # Three blocks today; an empty match would pass while testing nothing.
    assert ran >= 3, f"expected the README Python examples, found {ran}"
