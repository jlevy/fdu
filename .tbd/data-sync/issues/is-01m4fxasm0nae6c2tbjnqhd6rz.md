---
type: is
id: is-01m4fxasm0nae6c2tbjnqhd6rz
title: "Age: watch repaints measure ages from their own instant"
kind: task
status: open
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
created_at: 2026-10-09T08:44:01.534Z
updated_at: 2026-10-09T09:06:02.820Z
---
Each watch repaint and Session::report (Python Watch.report) read with a copy of the request whose now is that repaint's instant; windows are already absolute so they do not slide. The repaint check also compares each rendered row's exact mtime_ns and complete, with the age reference held fixed: an idle tree repaints nothing; any activity change repaints. Amend the Request.now doc, machine-output.md (fixed request instant), crates/fdu-py/README.md, SKILL.md, and the design principles' watch section. Tests: modified-after-start shows non-negative age; idle repaints nothing; touch within one age bucket repaints.
