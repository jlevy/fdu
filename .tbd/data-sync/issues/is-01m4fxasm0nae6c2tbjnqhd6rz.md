---
type: is
id: is-01m4fxasm0nae6c2tbjnqhd6rz
title: "Age: watch repaints measure ages from their own instant"
kind: task
status: closed
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
hold: null
hold_until: null
created_at: 2026-10-09T08:44:01.534Z
updated_at: 2026-10-09T11:16:03.072Z
started_at: 2026-10-09T09:50:07.616Z
closed_at: 2026-10-09T11:16:03.071Z
close_reason: "Landed in 000be640 (+docs ff609e61): Session::report and every repaint read with a request copy whose now is the repaint instant (windows already absolute); repaint identity measures ages from the session's fixed reference and mixes exact mtime_ns/complete of every age-showing row. Tests: new file gets non-negative age, idle repaints nothing while ages roll over, touch within one unit repaints. make check green at 66652033."
resolution: null
duplicate_of: null
---
Each watch repaint and Session::report (Python Watch.report) read with a copy of the request whose now is that repaint's instant; windows are already absolute so they do not slide. The repaint check also compares each rendered row's exact mtime_ns and complete, with the age reference held fixed: an idle tree repaints nothing; any activity change repaints. Amend the Request.now doc, machine-output.md (fixed request instant), crates/fdu-py/README.md, SKILL.md, and the design principles' watch section. Tests: modified-after-start shows non-negative age; idle repaints nothing; touch within one age bucket repaints.
