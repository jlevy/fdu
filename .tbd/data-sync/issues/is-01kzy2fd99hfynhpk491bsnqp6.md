---
type: is
id: is-01kzy2fd99hfynhpk491bsnqp6
title: "Snapshot load: insert records via known parent id and defer roll-ups to one bottom-up pass"
kind: task
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - perf
  - linux
dependencies: []
parent_id: is-01kzy554jjg27mz97mryenftym
created_at: 2026-08-13T17:24:48.552Z
updated_at: 2026-08-13T18:11:54.018Z
---
snapshot.rs parse_stream pushes every record through the full apply path: PathBuf join + normalize + ensure_dir_chain + per-record merge_upward, i.e. O(N*D) BTreeMap ancestor work and ~N PathBuf allocations to rebuild state the record already encodes (parent slot is in hand). A loader-private insert_child_of(parent_id, name, kind, attrs) plus a single deferred bottom-up roll-up pass makes load O(N) with near-zero path work. This extends accepted exp-005/exp-009 reasoning (preserve identity known at the boundary) and is the largest lever on the Linux warm-open inversion measured in the PR #8 review: verified warm open was +72% vs cold scan at 450k entries on ext4. Pre-register warm-snapshot-load component as the signal.
