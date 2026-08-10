"""Real-tree performance loop: measure fdu against a large tree we do not own.

This package is deliberately separate from the generated-corpus harness in
``benchmarks``. A generated corpus is ours: we can mutate it, restore it, and prove
an exact oracle for it. A real tree is somebody else's working directory. It is
immutable to us, it has no recipe, and the only honest guarantee we can offer is
that we noticed if it changed underneath a measurement.

The package is a development workflow, not a CI gate. Nothing here runs in
``make check``; see ``docs/project/guides/performance-loop.md``.
"""

__all__ = ["ledger", "measure", "profile", "tree"]
