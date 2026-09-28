---
type: is
id: is-01m3kjws5eyytmn0gxfe823rnw
title: "Python CLI entry: lazy package import so fdu starts in under 50 ms"
kind: task
status: in_progress
priority: 1
version: 4
labels: []
dependencies:
  - type: blocks
    target: is-01m3kfjgq9x471eftadpm5z1v9
parent_id: is-01m3kfj7vjnh8patz58z2p2bh1
created_at: 2026-09-28T08:42:52.450Z
updated_at: 2026-09-28T11:47:38.530Z
---
Measured 2026-09-28 on flowmark (10.4k files, load 16-23, 20 runs): native stack binary 63 ms vs dust 106 ms, but the installed Python-entry fdu 0.1.0 took 164 ms. Python-entry 'fdu --version' is 88 ms vs 8 ms native: interpreter 22 ms + 'import fdu' ~54 ms, of which fdu._models self-time is ~55 ms (frozen dataclass definitions) plus dataclasses/inspect ~20 ms; _native 14 ms. The console script does 'from fdu import _main', so fdu/__init__.py eagerly imports _api/_models the CLI never uses. Fix: PEP 562 lazy __getattr__ in fdu/__init__.py (TYPE_CHECKING imports for static typing), a minimal CLI entry that imports only fdu._native. Keep 'import fdu' / 'from fdu import X' / dir(fdu) behavior identical. Gate: Python-entry 'fdu --version' <= 45 ms and 'fdu .' on flowmark within ~40 ms of the native binary; python-check (strict typing), parity, smoke, readme examples pass.

## Notes

Implemented on branch claude/py-startup @ 38d849ab (from #144 head 3ae0e024); not yet linked into stack 141.
Design: fdu/__init__.py imports only fdu._native eagerly; public names resolve on first access via PEP 562 __getattr__/__dir__ in a runtime-only else-branch of 'if TYPE_CHECKING:' (local TYPE_CHECKING = False, deleted at module end), so pyright sees the same API and still flags misspelled fdu.X. _main moved from _api into __init__ with a local signal import; the fdu:_main entry point is unchanged. New tests: crates/fdu-py/tests/test_startup.py.
Measured (hyperfine medians, 20 runs, 3 warmups, load 7.3, same native .so both sides, flowmark): fdu --version 65.4 -> 21.6 ms (python -c pass 18.7, native 4.7); fdu . 111.0 -> 72.9 ms (native 57.6, dust 80.8). Gate <= 45 ms met; wheel fdu . is 15 ms over native.
Gates: make python-check pass (74 pytest incl. 4 new; basedpyright 0 errors), make python-smoke pass, make docs-format-check pass. make parity-check FAILS identically before and after this change: tests/golden/cli-content.tryscript.md (and cli-surface) still record '(N non-gitignored, 0 gitignored)', which 41b3ae75 changed to '(0 gitignored)' without regenerating goldens. Pre-existing in #144; not fixed here.
Only visible change: help(fdu) now also lists __getattr__ and __dir__. Remaining Python-side cost is the signal module import (enum), ~1.5-5 ms by load.
