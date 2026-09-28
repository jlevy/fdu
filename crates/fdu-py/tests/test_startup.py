"""The `fdu` command must not pay for the typed API it never uses (fdu-03yn).

The console script runs `from fdu import _main`, and importing the whole API there cost
about 55 ms of an 88 ms `fdu --version`. The package now imports its public names on
first access, so each check here runs in a fresh interpreter: in this one, other tests
have long since imported everything.
"""

from __future__ import annotations

import ast
import importlib
import subprocess
import sys
from pathlib import Path

import fdu


def _fresh(code: str) -> list[str]:
    """Run `code` in a new interpreter and return its stdout lines."""

    completed = subprocess.run(
        [sys.executable, "-c", code], check=False, capture_output=True, encoding="utf-8"
    )
    assert completed.returncode == 0, completed.stderr
    return completed.stdout.splitlines()


def test_the_console_entry_imports_only_the_native_module() -> None:
    # Charged against the interpreter's own start, so a module that site or a `.pth` file
    # loads is not counted as fdu's.
    [added] = _fresh(
        "import sys\n"
        "before = set(sys.modules)\n"
        "from fdu import _main\n"
        "print(' '.join(sorted(set(sys.modules) - before)))\n"
    )
    assert set(added.split()) == {"fdu", "fdu._native"}, added


def test_the_console_entry_still_runs_the_native_cli() -> None:
    lines = _fresh(
        "import signal, sys\n"
        "sys.argv = ['fdu', '--version']\n"
        "from fdu import _main\n"
        "status = _main()\n"
        "print(status, signal.getsignal(signal.SIGINT) is signal.SIG_DFL,"
        " 'fdu._models' in sys.modules)\n"
    )
    assert lines[0].startswith(f"fdu {fdu.__version__}"), lines
    # Exit status 0, Ctrl-C restored to its default (fdu-18vk), and still no models.
    assert lines[-1] == "0 True False", lines


_FRESH_IMPORT = r"""
import pickle, pydoc, re, sys
import fdu

print(sorted(name for name in sys.modules if name.split(".")[0] == "fdu"))
public = {name for name in dir(fdu) if not name.startswith("_") or name == "__version__"}
print(public == set(fdu.__all__), "fdu._models" in sys.modules)
from fdu import Index, Query
print(Index.__module__, Query.__module__)
query = fdu.Query(views=(fdu.View.SUMMARY,))
print(pickle.loads(pickle.dumps(query)) == query)
try:
    fdu.not_a_name
except AttributeError as error:
    print(error)
from fdu import opened
print(opened.__name__)
star = {}
exec("from fdu import *", star)
print(sorted(set(star) - {"__builtins__"}) == sorted(fdu.__all__))
text = pydoc.render_doc(fdu, renderer=pydoc.plaintext)
listed = set(re.findall(r"^    (?:class )?(\w+)\(", text, re.MULTILINE))
print([name for name in fdu.__all__ if callable(getattr(fdu, name)) and name not in listed])
"""


def test_a_fresh_import_is_lazy_and_complete() -> None:
    lines = _fresh(_FRESH_IMPORT)
    assert lines == [
        # Importing the package loads the native module and nothing else.
        "['fdu', 'fdu._native']",
        # `dir` lists every public name without importing them.
        "True False",
        "fdu._api fdu._models",
        "True",
        "module 'fdu' has no attribute 'not_a_name'",
        # A submodule is still reachable through the package.
        "fdu.opened",
        "True",
        # `help(fdu)` still documents every public class and function.
        "[]",
    ], lines


def _declared_sources() -> dict[str, str]:
    """Map each name the `TYPE_CHECKING` imports declare to the module it names."""

    assert fdu.__file__ is not None
    tree = ast.parse(Path(fdu.__file__).read_text(encoding="utf-8"))
    [guard] = [
        node
        for node in tree.body
        if isinstance(node, ast.If)
        and isinstance(node.test, ast.Name)
        and node.test.id == "TYPE_CHECKING"
    ]
    return {
        alias.asname or alias.name: node.module or ""
        for node in guard.body
        if isinstance(node, ast.ImportFrom)
        for alias in node.names
    }


def test_type_checkers_and_the_runtime_see_the_same_names() -> None:
    declared = _declared_sources()
    assert set(declared) | {"__version__"} == set(fdu.__all__)
    assert set(declared.values()) == {"_api", "_models"}, declared
    # The one list the lazy lookup keeps beside `__all__` must name exactly the `_api` ones.
    assert {name for name, module in declared.items() if module == "_api"} == fdu._API_NAMES
    for name, module in declared.items():
        source = importlib.import_module(f"fdu.{module}")
        assert getattr(fdu, name) is getattr(source, name), name
