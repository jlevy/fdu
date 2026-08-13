---
type: is
id: is-01kzkzms7gmpjb0smwfc0c74wr
title: Pin dut and gdu performance adapters with capability proofs
kind: task
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzkzmsegmx4sfswka2084se6
  - type: blocks
    target: is-01kzg4c6h9v2dzand7t090p278
parent_id: is-01kzkzm62q1vwxbv9hbp39bxxm
created_at: 2026-08-09T19:22:54.575Z
updated_at: 2026-08-13T14:56:26.095Z
---
Acquire dut and gdu through the read-only third-party checkout workflow; inspect repository instructions, hooks, submodules, build scripts, licenses, and dependency inputs before execution; then pin exact revisions/releases, build profiles, binaries, checksums, direct argument vectors, minimal environments, traversal/size/cache semantics, parsers, and non-skipping postconditions. Complete the capability matrix and label traversal baseline, normal product job, and fdu full-stat target separately. Unavailable or unverifiable tools fail release runs visibly; comparator binaries are never linked into or distributed with fdu.

## Notes

2026-08-13 dut audit: pin current upstream 68d4ba2 and the built binary checksum; source identifies as 1.1 although newest tag is 1.0. GPL means executable benchmark input and ideas only. Capability contract: retains directories plus bounded top-N files, not a full inventory; allocated/apparent/count modes are separate; no newest mtime or machine output. Reject stderr warnings even with exit 0 and require independent root-total checks. Fixture gate: multi-chunk >1 MiB directory, high-cardinality cross-directory hard links, sparse and preallocated files with inverse apparent/allocated ranking, unreadable/stat failure, symlinks, one-filesystem boundary, and supported non-UTF-8 names. Publish verified-warm, dut-compatible pagecache-drop-only, and controlled-cold as distinct regimes.
