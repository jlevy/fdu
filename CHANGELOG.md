# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial scaffold: the architecture expressed in working, tested code.
  - **Observation/commit contract** (`types.rs`): producers submit verified, optionally
    conditional `Upsert` / `Remove` / `InvalidateSubtree` observations; the index emits
    clocked `Commit` batches containing exact effective mutations and state transitions.
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
  - **Python package** (`fdu`): typed immutable options, reports, roll-ups, provenance,
    cache values, and change feeds over a private `fdu._native` extension, with bulk
    calls and GIL release during native work.
    The same abi3 wheel includes watch support, a `py.typed` marker, exact extension
    stubs, the native CLI console script, license text, and a CycloneDX SBOM.
  - **Release rehearsal**: exact `0.1.0` identity checks, crate/sdist/wheel content
    inspection, installed-wheel and installed-sdist consumer tests, strict downstream
    type checking, and portable Linux, macOS, and Windows wheel build definitions.
  - **Opt-in content metrics** (`content`): compiled stable file-type rules; bounded
    parallel one-pass UTF-8, NUL, line, blank, and raw-word analysis; conditional
    fingerprint-checked commits; sparse type/family roll-ups; and independently
    checksummed atomic sidecars that preserve metadata snapshot v2. The `code-sloc-v1`
    dialect partitions code, comment, and blank lines for 15 common languages; logical
    prose and reader-visible Markdown add normalized words, paragraphs, and
    aggregate-derived pages.
    Bounded shebang, modeline, ambiguous-header, format-signature, and origin probes run
    only after path-only classification cannot decide.
  - **Metric summaries** (`fdu.report/2`): stable `types`, `families`, `languages`, and
    `documents` views with exact byte-share fractions, coverage, analyzer provenance,
    logical page denominators, detection source and confidence, origin flags, and
    matching Rust, CLI, and Python surfaces.
    The original extension view is retained as `extensions`, while metadata-only
    `fdu.report/1` output remains unchanged.

### Changed

- Corrected the fixed `.gitignore` matcher’s recursive, negation, and character-class
  semantics and bounded adversarial pattern work.
  Its semantic fingerprint is now version 2, so snapshots written with the earlier
  matcher are intentionally rejected and rebuilt from the filesystem.
- Runtime type-rule registries now fingerprint their validated semantic values instead
  of accepting an asserted identity.
  This intentionally invalidates earlier snapshots and content sidecars once; the next
  complete run rebuilds them under the verified registry identity.

### Known limitations

- Content performance evidence is currently local M1/APFS data rather than a controlled
  cross-platform release matrix; CI checks semantics and benchmark contracts, not timing
  thresholds.
- Content sidecars are profile-scoped.
  Repeating one profile reuses unchanged files, but switching profiles can reread
  content whose lower-level analyzer results were already computed under another
  profile.
- Content coverage is also profile-scoped rather than per analyzer.
  An unsupported deeper analyzer leaves the file uncovered for that profile instead of
  retaining a separate lower-level metric record.
- Content analysis is one-shot; watch mode remains metadata-only.
- Standard LOC covers 15 common languages rather than SCC or Tokei’s long tail.
  Unsupported code stays explicit coverage, and embedded-language and AST metrics are
  deferred rather than approximated.
- The snapshot format is a bounded flat bootstrap format; compressed lazy blocks remain
  Phase 1 work.
- Entry records are not yet packed to the ~25–32 bytes per file memory target.
- Roll-up metrics are a fixed set rather than a reducer registry.
- Watcher queue bounds and permanent backend-failure marking remain explicit Phase 1
  hardening work.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
