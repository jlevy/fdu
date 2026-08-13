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
    freshness, revision-safe conditional arbitration (including structural and ABA
    races), an operation-bounded change feed, and correct pre-Unix-epoch timestamp
    reduction.
  - **Scan layer** (`scan.rs`): portable cold scan plus applying full/subtree/shared
    reconciliation with stale-observation arbitration, exact scope matching, retryable
    invalidations with missing/non-directory ancestor widening, root-only depth-zero
    semantics, bounded producer batches, and enforcement of depth, symlink, and
    filesystem subtree boundaries.
  - **Snapshot** (`snapshot.rs`): semantic-scope invalidation, bounded streaming load,
    payload integrity checks, complete-only concurrency-safe atomic replacement, and
    corrupt-equals-empty semantics.
    Unix snapshots are created owner-only (`0600`).
  - **Watch layer** (`watch.rs`, feature `watch`): notify-backed, coalescing then
    verifying by stat, with `Flag::Rescan` escalated to `InvalidateSubtree` rather than
    dropped, plus an apply/reconcile driver that closes invalidations.
    The applying driver re-verifies queued samples at a clock-stable commit boundary,
    rejects a watcher for another root, and rejects depth- and filesystem-restricted
    scopes until events can be filtered against those boundaries.
  - **CLI** (feature `cli`): composable scope, selection, view, format, and mode axes;
    compact human tree output; schema-versioned text/JSON/JSONL/YAML reports; cache
    lifecycle controls; and a `tail -f`-style watch stream.
    Reports require an explicit path, while bare `fdu` prints help without scanning the
    current directory.
  - **Python bindings** (`fdu-py`): bulk API mirroring the query, provenance, cache, and
    watch surfaces while releasing the GIL during native work.
    The wheel does not compile the optional watch dependency.
  - **Opt-in content metrics** (`content`): compiled stable file-type rules; bounded
    parallel one-pass UTF-8, NUL, line, blank, and raw-word analysis; conditional
    fingerprint-checked commits; sparse type/family roll-ups; and independently
    checksummed atomic sidecars that preserve metadata snapshot v2.
  - **Metric summaries** (`fdu.report/2`): stable `types`, `families`, `languages`, and
    `documents` views with exact byte-share fractions, coverage, analyzer provenance,
    logical page denominators, and matching Rust, CLI, and Python surfaces.
    The original extension view is retained as `extensions`, while metadata-only
    `fdu.report/1` output remains unchanged.

### Known limitations

- **No performance claims.** The walker is a portable `read_dir` + `symlink_metadata`
  implementation. The `getdents64`/`statx` layer that Goal 1 requires is not built, and
  no benchmark gate has been run.
- The snapshot format is a bounded flat bootstrap format; compressed lazy blocks remain
  Phase 1 work.
- Entry records are not yet packed to the ~25–32 bytes per file memory target.
- Roll-up metrics are a fixed set rather than a reducer registry.
- Watcher queue bounds and permanent backend-failure marking remain explicit Phase 1
  hardening work.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
