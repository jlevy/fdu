---
type: is
id: is-01m47831pcsbtvygctgzqfvzc8
title: "Spec: views imply the analysis they need (--view/--analyze redesign)"
kind: epic
status: open
priority: 2
version: 8
spec_path: docs/project/specs/active/plan-2026-10-05-fdu-view-analyze-redesign.md
labels: []
dependencies: []
child_order_hints:
  - is-01m4783dw80sfhd591g6k5ypt4
  - is-01m47845hn01rj4v9gf50dh4dp
  - is-01m47846395xp250085ae705pe
  - is-01m47846n4749gjqdf0hy5gt6w
  - is-01m47850add0aewxrewn0d5hc7
  - is-01m47850vtykg7fwyg5zx1mf39
  - is-01m47851e1mnk9xpfh9jyee6rx
created_at: 2026-10-05T23:58:52.099Z
updated_at: 2026-10-05T23:59:57.376Z
---
A content view (code, documents) is a request for the analysis it shows: a fresh basis (CLI, fdu.report, Request::build) enables the analyzers the views imply in union with --analyze; a held basis is never widened and refuses with the analyzer named; analyzers alone still choose their default view; --analyze remains the control axis. Demo command becomes fdu linux --view code,documents --limit 6. Full design, options rejected, before/after, and validation plan are in the spec.
