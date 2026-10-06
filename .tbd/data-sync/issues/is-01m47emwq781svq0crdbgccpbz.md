---
type: is
id: is-01m47emwq781svq0crdbgccpbz
title: DOCUMENTS view percentages sum to more than 100%
kind: bug
status: closed
priority: 2
version: 4
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m47z5n33np3b15yena1k3hqt
hold: null
hold_until: null
created_at: 2026-10-06T01:53:28.294Z
updated_at: 2026-10-06T10:08:57.459Z
started_at: 2026-10-06T06:42:16.371Z
closed_at: 2026-10-06T10:08:57.458Z
close_reason: "Fixed: grouped-section share denominators are the sum of every row's numerator before the share filter and row bound (query_report.rs share_denominator). Document words are derived from pooled LogicalWordStats, so a mixed section's pooled total was below the rows' sum and shares summed past 100%. Engine test + golden session (71.4% + 28.6%, was 88.0% + 35.2%), docs, CHANGELOG Fixed, README sample regenerated. make check exit 0 at 6bbed3ce on claude/document-shares-fix."
resolution: null
duplicate_of: null
---
In the text documents view, the share column ('percentages are shares of document words') sums past 100%. On a shallow Linux clone (2026-10-05): rst 78.9%, xml 13.0%, text 9.2%, markdown 0.5%, latex <0.1%, html <0.1% (= 101.6%), while the rows' displayed word counts (4,390,955 / 723,951 / 512,005 / 27,102 / 1,334 / 149) give rst 77.6%. The README sample shows the same (74.7 + 24.5 + 8.8). Likely the share numerator or denominator uses a different word measure (document vs raw words) or a different population than the rows. Find which, fix so shares are exact and sum to <=100% (bounded rows), and add a test that a documents section's shares sum to 100% when unbounded.
