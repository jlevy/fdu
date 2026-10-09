---
type: is
id: is-01m4ftg7y0q3zkgcje4wdvpzfy
title: Stability pass reports 'nothing skipped' when Phase 4's subtree analysis did not run
kind: bug
status: open
priority: 2
version: 1
labels:
  - release
  - testing
dependencies: []
parent_id: is-01m47z5n33np3b15yena1k3hqt
created_at: 2026-10-09T07:54:34.290Z
updated_at: 2026-10-09T07:54:34.290Z
---
Found while recording 0.4.0 (#190): with FDU_QA_MEDIUM_ANALYZE unset and a medium tree that has Documentation/ but no docs/, the harness's Phase 4 subdirectory --analyze=code check and its reuse check never ran (47 checks against 0.3.0's 49), yet the driver reported 'Nothing failed, and nothing was skipped'. Record it as a skip (exit 3) or default the analyze subtree more robustly (e.g. Documentation/), with a test.
