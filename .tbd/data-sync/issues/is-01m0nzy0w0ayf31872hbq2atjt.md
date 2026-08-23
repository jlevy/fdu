---
type: is
id: is-01m0nzy0w0ayf31872hbq2atjt
title: "PR #42 review R19: lib-only now carries two overlapping comments and a positional reference"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m0nzwr6pcg3fnj1445zpc81z
created_at: 2026-08-23T00:22:08.000Z
updated_at: 2026-08-23T00:39:59.073Z
closed_at: 2026-08-23T00:39:59.073Z
close_reason: Fixed. One comment on lib-only, describing the guard by what it does rather than by line position.
---
Makefile:231-235. The pre-existing comment was kept and a new one added; the new one says 'The third line is the check', which is already a two-line continuation.
