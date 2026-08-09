---
type: is
id: is-01kzm3nms9b78zn0nqqyy9sq26
title: Reconcile all remaining work into an urgent-first plan and bead graph
kind: task
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels:
  - planning
dependencies: []
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-09T20:33:17.096Z
updated_at: 2026-08-09T20:48:34.715Z
---
Inventory every active and future issue against the three governing plans; backfill missing issues and spec paths; eliminate orphan work; make merge blockers, pre-release hardening, evidence infrastructure, engine work, product work, and later backlog explicit; wire only real blocker dependencies; list every owned bead in its plan; validate links and tbd integrity; commit, push, and wait for CI.

## Notes

Inventory, graph, and plan reconciliation complete. All 42 non-closed beads have a governing spec and every spec names each owned bead. Only independent P0 bugs fdu-ad45 and fdu-nlh8 are implementation-ready; both block final approval fdu-sn43, and all downstream work is gated behind that approval or a real measured/design prerequisite. Added future epic fdu-x746 and a self-contained future roadmap for four deferred extensions. Precommit review found one graph defect (parent blockers do not propagate); fixed it by adding fdu-9cf0 directly to every future child. Flowmark, document hygiene, git diff checks, tbd integrity, and make check all pass. tbd doctor reports only the known managed AGENTS.md and Codex-hook drift owned by fdu-ad45. Pending commit, push, PR update, and CI.
