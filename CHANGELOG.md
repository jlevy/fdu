# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial scaffold: the architecture expressed in working, tested code.
  - **Observation/commit contract** (`types.rs`): producers submit verified, optionally
    conditional `Upsert` / `Remove` / `InvalidateSubtree` observations; the index emits
    clocked `AppliedDelta` batches containing only effective mutations.
  - **In-memory index** (`index.rs`): parent-pointer arena with per-directory
    pre-computed roll-ups (counts, apparent and allocated bytes, newest mtime,
    per-extension tallies), O(depth) apply, generation-safe free-slot reuse, explicit
    freshness, and an operation-bounded change feed.
  - **Scan layer** (`scan.rs`): portable cold scan plus applying full/subtree/shared
    reconciliation with stale-observation arbitration.
  - **Snapshot** (`snapshot.rs`): semantic-scope invalidation, bounded streaming load,
    complete-only atomic replacement, and corrupt-equals-empty semantics.
  - **Watch layer** (`watch.rs`, feature `watch`): notify-backed, coalescing then
    verifying by stat, with `Flag::Rescan` escalated to `InvalidateSubtree` rather than
    dropped, plus an apply/reconcile driver that closes invalidations.
  - **CLI** (feature `cli`): human tree output with percentage bars, `--by-type`,
    versioned JSON (`fdu.tree/2`), exact entry kinds, error details, partial exit status,
    `NO_COLOR`, and pipe detection.
  - **Python bindings** (`fdu-py`): bulk API retaining scan scope and exposing freshness
    and errors while releasing the GIL during native work.

### Known limitations

- **No performance claims.** The walker is a portable `read_dir` + `symlink_metadata`
  implementation. The `getdents64`/`statx` layer that Goal 1 requires is not built, and
  no benchmark gate has been run.
- The snapshot format is a bounded flat bootstrap format; compressed lazy blocks remain
  Phase 1 work.
- Entry records are not yet packed to the ~25–32 bytes per file memory target.
- Roll-up metrics are a fixed set rather than a reducer registry.
