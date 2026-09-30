"""Suite-wide isolation that no single test should have to remember."""

from __future__ import annotations

from collections.abc import Iterator

import pytest


@pytest.fixture(autouse=True)
def isolated_cache(
    tmp_path_factory: pytest.TempPathFactory, monkeypatch: pytest.MonkeyPatch
) -> Iterator[None]:
    """
    Keep every test out of the invoking user's cache.

    A test that writes a snapshot through the default location would otherwise leave it
    in the real `~/.cache/fdu`, and read whatever an earlier run left there (fdu-n57h).
    `FDU_CACHE_DIR` outranks `XDG_CACHE_HOME`, so an exported one is removed too. The
    directory is not the test's own `tmp_path`, which tests scan as a tree; a test that
    sets `XDG_CACHE_HOME` itself still wins, because its fixture runs after this one.
    """
    monkeypatch.delenv("FDU_CACHE_DIR", raising=False)
    monkeypatch.setenv("XDG_CACHE_HOME", str(tmp_path_factory.mktemp("xdg-cache")))
    yield
