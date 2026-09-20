---
type: is
id: is-01m2yhjb6r67n23jera0t03ebv
title: Retained content failures are reused and can become falsely complete
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-20T04:35:38.839Z
updated_at: 2026-09-20T04:35:38.839Z
---
At 937f9445, index::pending_analysis_candidates reuses matching IoError/ChangedDuringRead records, so the next analyze_index returns a default complete report and Python refresh clears earlier errors. Disk persistence already refuses these records. Add shared per-outcome reuse eligibility and retained-index recovery/repeated-failure tests; coordinate with per-analyzer model.
