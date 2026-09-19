---
type: is
id: is-01m2xrgf9vg6pexb6cjvf92v3f
title: "H129: cache-only restore omits classify"
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
delegate: unknown@spud10
labels: []
dependencies: []
parent_id: is-01m2x7xbt1c4we0wffeth7jrd6
hold: null
hold_until: null
created_at: 2026-09-19T21:17:43.098Z
updated_at: 2026-09-19T21:27:13.223Z
started_at: 2026-09-19T21:17:53.095Z
closed_at: 2026-09-19T21:27:13.222Z
close_reason: Accepted exp-128. Restore-without-classify kept at 6887a864. Wall -13.11% on frozen metabrowser-clone. Quiet this tick 31.53%.
resolution: null
duplicate_of: null
---
Cache-only restore classifies every file twice (analysis_candidates, then apply_analysis_record staleness guard) and then commits the sidecar classification. exp-125 sample: first classify 5.94% of content_open (909/15296), apply guard 7.36% (1125/15296). Combined ~13.3% of content_open.

Mechanism: restore-only candidate walk keeps the HashMap (not H116) and path_of; omits classify; restore apply skips the self-check. Completeness stays H125 restore-count. Digest identical. Incomplete sidecar still refused.

Quiet gate this tick: 31.53% (fail once, skip). Uncontrolled allowed. Do not retry file-count. Do not mint H86/EntryId.
