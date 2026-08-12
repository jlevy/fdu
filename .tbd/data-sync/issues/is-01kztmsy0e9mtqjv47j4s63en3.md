---
type: is
id: is-01kztmsy0e9mtqjv47j4s63en3
title: Keep performance ledger conditions tied to its cumulative comparison
kind: bug
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies: []
created_at: 2026-08-12T09:28:09.997Z
updated_at: 2026-08-12T11:29:18.553Z
closed_at: 2026-08-12T11:29:18.552Z
close_reason: "Fixed in d1e730e: headline and reproduction sections now share the latest cumulative anchor, with mixed-subject regression coverage."
---
The generated ledger selects the latest cumulative experiment for Where it stands but the latest experiment overall for Reproducing this. Once exp-015 added a second subject, the headline's 60k comparison was silently paired with 720k reproduction facts. Use one anchor experiment for both sections and add mixed-subject regression coverage.
