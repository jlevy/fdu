---
type: is
id: is-01m2yhjb6r67n23jera0t03ebv
title: Retained content failures are reused and can become falsely complete
kind: bug
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
delegate: codex@spud10
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
hold: null
hold_until: null
created_at: 2026-09-20T04:35:38.839Z
updated_at: 2026-09-23T08:14:06.761Z
started_at: 2026-09-20T04:39:46.910Z
closed_at: 2026-09-23T08:14:06.761Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
At 937f9445, index::pending_analysis_candidates reuses matching IoError/ChangedDuringRead records, so the next analyze_index returns a default complete report and Python refresh clears earlier errors. Disk persistence already refuses these records. Add shared per-outcome reuse eligibility and retained-index recovery/repeated-failure tests; coordinate with per-analyzer model.
