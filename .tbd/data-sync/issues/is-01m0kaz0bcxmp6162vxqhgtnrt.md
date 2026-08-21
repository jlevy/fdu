---
type: is
id: is-01m0kaz0bcxmp6162vxqhgtnrt
title: "tryscript: requires: declares and reports which binary a run resolved"
kind: feature
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-python-cli-parity.md
labels: []
dependencies: []
parent_id: is-01m0k965p7hx4dy6t0cj29rsae
created_at: 2026-08-21T23:37:11.272Z
updated_at: 2026-08-21T23:37:11.272Z
---
tryscript resolves a command through PATH and says nothing about where it landed. For a
parity run that is the difference between a proven result and a green check.

Add `requires:` to the front matter: named commands must resolve before the first session
runs, and the run reports the resolved path.

    requires:
      - fdu

    resolved fdu -> /.../tests/parity/py/fdu   (12 files, 129 sessions)

Abort rather than run when a required command is missing. Reporting the path is the half
that matters -- it makes the harness legible rather than merely correct, the same reason
every fdu report carries its own source and freshness.
