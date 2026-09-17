---
type: is
id: is-01m2q1amctg37zcrd417xcmjzs
title: make python-smoke's uv tool run step still picks a free-threaded Python
kind: bug
status: open
priority: 2
version: 1
labels:
  - release
dependencies: []
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-09-17T06:37:07.865Z
updated_at: 2026-09-17T06:37:07.865Z
---
make python-smoke's last step runs `uv tool run --isolated --no-index --from <wheel> fdu --version` with no --python. On a host whose default uv interpreter is free-threaded CPython 3.14 (cp314t), uv refuses the cp312-abi3 wheel: 'A path dependency is incompatible with the current platform ... free-threaded CPython 3.14 ... stable ABI (abi3) requires a GIL-enabled interpreter'. fdu-pd1b pinned WHEEL_PYTHON for the uv venv steps but missed this uv tool run line, so make check fails locally on such a host (CI is unaffected). Fix: pass --python $(WHEEL_PYTHON) to that uv tool run. Workaround: UV_PYTHON=3.12 make check. Found 2026-09-17 while gating the path-independence layer.
