"""The suite resolves its cache under a directory of its own, never the user's."""

from __future__ import annotations

import os
from pathlib import Path

import fdu


def test_the_default_cache_is_the_suites_own() -> None:
    # conftest.py's autouse fixture; without it a snapshot written through the default
    # location lands in the invoking user's cache (fdu-n57h).
    assert "FDU_CACHE_DIR" not in os.environ
    home = Path(os.environ["XDG_CACHE_HOME"])
    directory = fdu.cache_directory()
    assert directory == (home / "fdu").absolute(), directory
