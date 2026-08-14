---
type: is
id: is-01kzyetdsvgkx14ypgwe5qx9jg
title: "Re-land the content-metrics feature stranded when PR #10 was auto-closed"
kind: bug
status: open
priority: 1
version: 2
labels:
  - performance
dependencies:
  - type: blocks
    target: is-01kzyev98rtws5r6q3gswp7jt0
created_at: 2026-08-13T21:00:32.443Z
updated_at: 2026-08-13T21:01:00.568Z
---
PR #10 (feat: fast incremental code and document metrics — 10,848 additions across 112 files, 40 commits) was auto-closed by GitHub at 18:40:56 on 2026-08-13, four seconds behind the #8 merge, because it was based on codex/iterative-performance and that merge deleted the base branch. It was never merged: crates/fdu/src/content/ does not exist on main, and the spec is not in docs/project/specs/done/ on main. The work is intact on branch codex/file-content-metrics-plan at fbb36f892b1f31b4c51ade16403b846e22d080c6, with all four Bugbot review threads resolved and its checks green at that head. Tracking is currently inaccurate as a result: fdu-3n8c and fdu-eu80 were closed on 2026-08-13T12:03 with the reason 'All six content-metrics phases are implemented and validated by ... green cross-platform PR #10 checks', which is true of that branch but not of main. Action: open a fresh PR from codex/file-content-metrics-plan against main (GitHub cannot reopen or retarget a PR whose base branch is deleted), re-run the gate on the rebased head, then either confirm or reopen fdu-3n8c and fdu-eu80 to match what main actually contains.
