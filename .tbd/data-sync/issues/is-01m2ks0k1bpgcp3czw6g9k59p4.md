---
type: is
id: is-01m2ks0k1bpgcp3czw6g9k59p4
title: "PR #67 review PR67-5: JSON cache rows omit content_bytes where YAML and Python carry it"
kind: bug
status: closed
priority: 3
version: 2
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2krzt6endqrw5gq2phas5g6
created_at: 2026-09-16T00:14:06.890Z
updated_at: 2026-09-16T03:50:00.669Z
closed_at: 2026-09-16T03:50:00.668Z
close_reason: "f39b701: the JSON row writes content_bytes on every state, as YAML and the Python dict already did; goldens updated for the unrecognized and absent rows."
resolution: null
duplicate_of: null
---
crates/fdu-core/src/report_format.rs:1423 vs :1457; crates/fdu-py/src/lib.rs cache_status_dict. JSON writes content_bytes only for an fdu snapshot; YAML and the Python dict always carry it. DECISION (user): make JSON carry content_bytes on every row kind, or keep the rows consistent the other way.
