---
type: is
id: is-01m3kfjj22b63eyenyc9ys3b28
title: Hand off the 0.2.0 release to the maintainer
kind: task
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m3mcwtkbvr9kj5j2qwpyyd3j
created_at: 2026-09-28T07:44:51.774Z
updated_at: 2026-09-28T17:57:07.921Z
closed_at: 2026-09-28T17:57:07.920Z
close_reason: "Superseded: the user directed the agent to cut 0.2.0 end to end (fdu-ryof); published 2026-09-28."
resolution: null
duplicate_of: null
---
Maintainer-only steps after the stack merges (never done by an agent without explicit go-ahead): release.yml rehearsal dispatch on main, verify artifacts, signed tag v0.2.0, publish through the release environment, announce. Prepare a short checklist in the final report with the exact commands from docs/project/guides/release-process.md.
