---
type: is
id: is-01m0hg9f26pmh0c9vbfswb624n
title: Note when requested analysis is displayed by no selected view
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01m0hfrm2xbqzdx4avgegcvf0t
created_at: 2026-08-21T06:31:48.037Z
updated_at: 2026-08-21T07:15:54.272Z
closed_at: 2026-08-21T07:15:54.271Z
close_reason: Implemented on claude/fdu-content-axis; make check green (24 suites, 114 goldens).
---
The complement of the `--view all` omission note. When an explicit `--view` selects only
views that ignore content analysis, the run bought I/O it cannot display:

  note: --analyze all read 1.2 GiB; no selected view displays content metrics
        - try --view families, languages, or all

A note, never an error, for one reason: warming the content sidecar so a later run is
warm is a legitimate use and `--cache`-aware callers depend on it. An error would break
that; silence would hide a potentially enormous cost.

Test both directions of Principle 13: the note appears and the run still exits 0, and no
view under any content setting causes a file body to be opened that `--analyze` did not
authorize.
