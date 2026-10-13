---
type: is
id: is-01m4v2r2sx4pgcwxjfse8kre8n
title: Describe current fdu behavior in README without release history
kind: task
status: in_progress
priority: 2
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
hold: null
hold_until: null
created_at: 2026-10-13T16:50:18.556Z
updated_at: 2026-10-13T16:54:49.121Z
started_at: 2026-10-13T16:50:35.156Z
---
User requested README-only cleanup: describe current behavior and remove historical version comparisons and performance charts, linking to the existing measurements report instead. Preserve useful installation compatibility, evidence provenance, author voice, and other agents work. Scope: README.md only; coordinator owns review, gates, PR, and release.

## Notes

README-only patch prepared on main e6b8850b. Replaced historical benchmark table, release comparisons, optimization chronology, and old timing claims with current native-reader, bounded-report, and cache behavior plus performance report links. Removed obsolete clone revision and benchmark footnote; retained demo provenance, install compatibility/pinning, peer feature-source versions, and standard footer. Changed README Fastest headline to Fast; other package surfaces still say Fastest and were reported to coordinator, not edited. Validation: git diff --check passed; 29 local link targets exist; exactly one guideline footer; no historical fdu 0.0-0.4 release references remain. Coordinator reviewed scope and is running pinned make docs-format. Full make check remains unrun because external scratch has insufficient safe capacity. Leave open pending gate, review, commit, PR, and CI. Coordinator owns final sync and release gates.
