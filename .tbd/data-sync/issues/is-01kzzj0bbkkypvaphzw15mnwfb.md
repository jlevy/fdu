---
type: is
id: is-01kzzj0bbkkypvaphzw15mnwfb
title: "S1: emit parent-relative observations from the cold scan"
kind: task
status: open
priority: 0
version: 1
labels: []
dependencies: []
created_at: 2026-08-14T07:15:26.707Z
updated_at: 2026-08-14T07:15:26.707Z
---
The cold scan has the same defect the snapshot loader had, and the fix for the loader measured -51.9 percent. scan.rs emits Op::Upsert with a cloned relative PathBuf; Index::apply_upsert then calls normalize() and ensure_dir_chain() to descend from the root, one map lookup per level, to reach the directory the worker was standing in when it produced the record. A single-threaded callgrind of scan-index over 17,100 entries, with the probe's oracle backed out, puts the allocator at about 35 percent of engine work and path-component iteration at about 13, and the caller tree attributes 426,818 component comparisons to apply_validated - about 25 per entry. Add a parent-relative observation form, Op::UpsertUnder { parent: EntryId, name, kind, attrs }, emitted for entries below a directory the walk has already established. This does not bypass the delta contract: the index still arbitrates, conditional generation and revision guards still apply, and reconciliation already carries parent-relative expectations. Predict cold-scan-index wall down at least 15 percent with user CPU and minor faults down; cold-scan-producer unchanged since the work is consumer-side. Index tier only. Re-screen S2, S3 and S4 after this lands.
