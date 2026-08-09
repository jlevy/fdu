# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial scaffold: the architecture expressed in working, tested code.
  - **Delta contract** (`types.rs`): `Upsert` / `Remove` / `InvalidateSubtree`, clocked
    and idempotent — the only way the index or cache is modified.
  - **In-memory index** (`index.rs`): parent-pointer arena with per-directory
    pre-computed roll-ups (counts, apparent and allocated bytes, newest mtime,
    per-extension tallies), O(depth) delta application, and free-slot reuse.
  - **Scan layer** (`scan.rs`): portable walk and revalidation, both producing deltas.
  - **Snapshot** (`snapshot.rs`): engine-fingerprint invalidation, atomic temp-file plus
    rename, and corrupt-equals-empty semantics.
  - **Watch layer** (`watch.rs`, feature `watch`): notify-backed, coalescing then
    verifying by stat, with `Flag::Rescan` escalated to `InvalidateSubtree` rather than
    dropped.
  - **CLI** (feature `cli`): human tree output with percentage bars, `--by-type`,
    versioned JSON (`fdu.tree/1`), `NO_COLOR` and pipe detection.
  - **Python bindings** (`fdu-py`): bulk API releasing the GIL during native work.

### Known limitations

- **No performance claims.** The walker is a portable `read_dir` + `symlink_metadata`
  implementation. The `getdents64`/`statx` layer that Goal 1 requires is not built, and
  no benchmark gate has been run.
- The snapshot format is a flat uncompressed placeholder; only its lifecycle invariants
  are final.
- Entry records are not yet packed to the ~25–32 bytes per file memory target.
- Roll-up metrics are a fixed set rather than a reducer registry.
