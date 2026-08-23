---
type: is
id: is-01m0prhcmhj09n41s1zae35yhm
title: "Session integration shape: mid-walk progress, async form, session-to-watch clock handoff"
kind: task
status: open
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0qs0msk75k8r89b44vqqjnz
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:32:08.465Z
updated_at: 2026-08-23T17:01:33.827Z
---
Three requirements that land with the progressive-results session, not after it: progress readable mid-walk (entries applied, clock, completeness) for crawl-status UIs; the async shape shipping with the sync one (same adapter policy as watch); and the walk-complete clock being the clock a watch resumes from, tested for the no-gap property.

## Notes

The Python and CLI shapes land together because the CLI shape is what makes the Python shape testable. Cli::run_watch (cli.rs:661) is refactored so its repaint loop takes either producer; prepare_report (execution.rs:188) grows a progressive sibling that retains the index and yields frames. Progress readable mid-walk — entries applied, clock, completeness — because a crawl-status UI renders exactly that. The CLI half is fdu-m893; the goldens are fdu-ey9q.
