---
type: is
id: is-01kzg4ak7v8z2a7s41rsms8jcb
title: "Revalidation: directory-mtime shortcut and parallel sweep streamed as deltas"
kind: feature
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzg4c6h9v2dzand7t090p278
  - type: blocks
    target: is-01kzg4d2fb96erw3h1b5k0c6xy
parent_id: is-01kzg48ekn4sm0azybr010qgmn
child_order_hints:
  - is-01kzn1jf9f4k7amp6z8t5g07zx
created_at: 2026-08-08T07:27:45.915Z
updated_at: 2026-08-10T05:15:50.446Z
---
Optimize revalidation after the measured 10k-1M cost curve and syscall walker. An unchanged cached directory fingerprint may skip only read_dir name-set discovery; it must never skip statting every known child or recursing into known directories, because an in-place file edit does not change the parent directory mtime. A changed directory fingerprint requires re-listing to discover additions, removals, and renames. Matching file fingerprints retain derived data with zero content reads. Run the truth-check as a bounded parallel sweep and stream conditional observations as deltas; stale-while-revalidating must remain explicitly labeled. Prove in-place edits, membership changes, timestamp-edge cases, partial errors, races, cancellation, and sequential-oracle equivalence before accepting speed evidence.
