---
type: is
id: is-01m0hg8h0t611hf16gbtnzacat
title: "Sidecar: bitmask encoding and containment-based record reuse"
kind: task
status: closed
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01m0hfrm2xbqzdx4avgegcvf0t
created_at: 2026-08-21T06:31:17.273Z
updated_at: 2026-08-21T07:15:53.301Z
closed_at: 2026-08-21T07:15:53.300Z
close_reason: Implemented on claude/fdu-content-axis; make check green (24 suites, 114 goldens).
---
Encode the analyzer set as a bitmask in the content sidecar, replacing the 0-4 ordinal.

Change record reuse from `record.profile == request.profile` to containment: a stored
record satisfies a request when its set is a superset of the requested one, projecting
down to the requested metrics.

Regression this replaces: an `all` sidecar forces a complete re-read for a later `code`
query even though every metric it needs is already stored.

Test: write a sidecar with `all`, then request `code`, and assert zero fresh reads.
