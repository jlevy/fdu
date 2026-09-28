---
type: is
id: is-01m3kjws5eyytmn0gxfe823rnw
title: "Python CLI entry: lazy package import so fdu starts in under 50 ms"
kind: task
status: open
priority: 1
version: 2
labels: []
dependencies:
  - type: blocks
    target: is-01m3kfjgq9x471eftadpm5z1v9
parent_id: is-01m3kfj7vjnh8patz58z2p2bh1
created_at: 2026-09-28T08:42:52.450Z
updated_at: 2026-09-28T08:42:53.598Z
---
Measured 2026-09-28 on flowmark (10.4k files, load 16-23, 20 runs): native stack binary 63 ms vs dust 106 ms, but the installed Python-entry fdu 0.1.0 took 164 ms. Python-entry 'fdu --version' is 88 ms vs 8 ms native: interpreter 22 ms + 'import fdu' ~54 ms, of which fdu._models self-time is ~55 ms (frozen dataclass definitions) plus dataclasses/inspect ~20 ms; _native 14 ms. The console script does 'from fdu import _main', so fdu/__init__.py eagerly imports _api/_models the CLI never uses. Fix: PEP 562 lazy __getattr__ in fdu/__init__.py (TYPE_CHECKING imports for static typing), a minimal CLI entry that imports only fdu._native. Keep 'import fdu' / 'from fdu import X' / dir(fdu) behavior identical. Gate: Python-entry 'fdu --version' <= 45 ms and 'fdu .' on flowmark within ~40 ms of the native binary; python-check (strict typing), parity, smoke, readme examples pass.
