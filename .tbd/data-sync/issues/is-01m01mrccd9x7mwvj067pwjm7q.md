---
type: is
id: is-01m01mrccd9x7mwvj067pwjm7q
title: "H89: one-slot extension memo; stop allocating and UTF-8-validating per file"
kind: task
status: closed
priority: 2
version: 2
spec_path: docs/project/research/research-2026-08-15-consumer-structural-headroom.md
labels:
  - perf
dependencies: []
created_at: 2026-08-15T02:42:00.205Z
updated_at: 2026-08-15T03:11:52.665Z
closed_at: 2026-08-15T03:11:52.665Z
close_reason: "Refuted as exp-056: wall +1.59% [-1.62%, +6.08%], user CPU regressed; the memo costs what derive-and-intern saved (H51/H62 shape). Headroom accrues to fdu-xde5 (H86)"
---
derive_ext builds a Vec per file (dhat ~0.9 alloc/file), String::from_utf8 validates it (~3% engine Ir), intern_ext resolves by String compare in a BTreeMap. Directory listings carry runs of the same extension, so a one-slot (bytes,ExtId) memo beside the parent memo removes allocation, validation, and most lookups. Pre-registered signal: cold-scan-index user_cpu_ns down; wall down >=3% alone or folded into H86.
