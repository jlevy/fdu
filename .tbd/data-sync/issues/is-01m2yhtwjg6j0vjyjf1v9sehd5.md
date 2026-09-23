---
type: is
id: is-01m2yhtwjg6j0vjyjf1v9sehd5
title: Preserve native path identity in cache status output
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
created_at: 2026-09-20T04:40:18.759Z
updated_at: 2026-09-23T08:14:06.871Z
started_at: 2026-09-20T04:40:49.993Z
closed_at: 2026-09-23T08:14:06.871Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
Cache status currently renders cache paths and roots through lossy strings without raw companions. Add raw path identity to fdu.cache/2 rows through the shared emission policy and test non-Unicode cache/root paths.
