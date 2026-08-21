---
type: is
id: is-01m0kazqkb46cjw4byw8x1ae7k
title: make test-parity, with guards that have each been seen to fail
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-python-cli-parity.md
labels: []
dependencies: []
parent_id: is-01m0k965p7hx4dy6t0cj29rsae
created_at: 2026-08-21T23:37:35.082Z
updated_at: 2026-08-21T23:37:35.082Z
---
`make test-parity` replays the corpus against each surface and compares the diff with the
committed deviation file, reporting passes, failures, and skips per surface.

Guards, each of which must have been seen to fail before it is worth having:

- an empty diff fails -- the shim did not run
- removing the shim gives `command not found` on the first session, not a passing run
- a deliberate divergence produces extra hunks and fails
- the skip count is asserted, so a shim cannot skip its way to green

Outside `make check` until the shim is complete and the deviation file is argued, then
inside.
