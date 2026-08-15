---
type: is
id: is-01m01mrdmjsmjq1ykp1d01zspf
title: "H92: persist roll-ups and the extension interner in the snapshot"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/research/research-2026-08-15-consumer-structural-headroom.md
labels:
  - perf
dependencies: []
created_at: 2026-08-15T02:42:01.490Z
updated_at: 2026-08-15T02:42:01.490Z
---
Load re-derives what save discarded: merge_upward per record, re-interned extensions. Both are deterministic functions of the tree the snapshot already pins. Composes with fdu-pdra (H78 usable-layout format) and H35 per-block checksums; fail-closed preserved (checksum-before-trust, per block). Pre-registered signal: warm-snapshot-load component_ns down several-fold on the 450k subject.
