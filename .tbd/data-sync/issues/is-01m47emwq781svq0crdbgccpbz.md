---
type: is
id: is-01m47emwq781svq0crdbgccpbz
title: DOCUMENTS view percentages sum to more than 100%
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-10-06T01:53:28.294Z
updated_at: 2026-10-06T01:53:28.294Z
---
In the text documents view, the share column ('percentages are shares of document words') sums past 100%. On a shallow Linux clone (2026-10-05): rst 78.9%, xml 13.0%, text 9.2%, markdown 0.5%, latex <0.1%, html <0.1% (= 101.6%), while the rows' displayed word counts (4,390,955 / 723,951 / 512,005 / 27,102 / 1,334 / 149) give rst 77.6%. The README sample shows the same (74.7 + 24.5 + 8.8). Likely the share numerator or denominator uses a different word measure (document vs raw words) or a different population than the rows. Find which, fix so shares are exact and sum to <=100% (bounded rows), and add a test that a documents section's shares sum to 100% when unbounded.
