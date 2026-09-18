---
type: is
id: is-01m2phzbwx3qzvf77e6xajdsj9
title: Wheel console command ignores Ctrl-C during --watch (uvx, uv tool install, pip)
kind: bug
status: closed
priority: 0
version: 3
labels:
  - release
  - python
dependencies:
  - type: blocks
    target: is-01m2phzegm4b3scda7d1xq3gnm
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-09-17T02:08:50.077Z
updated_at: 2026-09-18T03:07:28.138Z
closed_at: 2026-09-18T03:07:28.138Z
close_reason: "Implemented on PR #87: SIGINT, registry READMEs, caret pins, version stamp/LF, 0.2 API note, 200ms interval, transient-summary --no-gitignore, python-smoke --python, JSON 2^53."
---
The wheel's `fdu` is a Python console script (`fdu=fdu:_main`) that runs the native CLI inside
`_native.main()`. Python's SIGINT handler only sets a flag nothing checks until the native call
returns, and `--watch` never returns, so Ctrl-C does nothing. A one-shot run finishes its work
first and only then dies. The native `cargo install` binary exits immediately.

Found by end-to-end testing of the 0.1.0 release-candidate wheel on macOS (control: `sleep` and a
one-shot scan die on SIGINT; `fdu --watch` survives 10 s with SIGINT at default disposition).

Fix on claude/release-e2e-fixes: `_api._main` restores `SIGINT` to `SIG_DFL` before calling the native CLI.
Regression test in crates/fdu-py/tests/smoke.py (POSIX) waits for the first watch report, sends
SIGINT, and requires exit by signal; verified to fail on the shipped `_api.py` and pass with the fix.
The Python library's own `Index.watch()` loop already raises KeyboardInterrupt within about 1 s.
