---
type: is
id: is-01m0vrk2scfs6rfsm2hfnwkz50
title: "Two-engine agreement oracle: conformance packet and recorded-observation replay"
kind: task
status: open
priority: 1
version: 7
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5017522830
    at: 2026-08-25T09:57:38.534Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5018663775
    at: 2026-08-25T12:07:06.491Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5019372007
    at: 2026-08-25T13:21:15.010Z
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-25T06:09:16.076Z
updated_at: 2026-08-25T13:21:15.011Z
---
Split out of fdu-vfyw, which shipped fdu's side of the contract: the reference embedder
example (crates/fdu-py/examples/browser_provider.py), the semantic_fingerprint recipe,
and the filesystem scenarios -- symlinks as leaves, the hidden allowlist, and gitignore
negations decided by control files pruning kept out of the index.

What remains needs the other repository and cannot be finished from fdu alone:

- The vendored File Rollup conformance packet, replayed into fdu and checked against the
  recorded expectations. metabrowser carries it at
  src/metabrowser/data/file-rollup-format/file-rollup-conformance.json.
- A recorded-observation replay driven into BOTH engines from one capture, so the two are
  compared over identical inputs rather than over two walks of one tree.
- The combined identity agreed byte for byte: fdu's adapter emits named components sorted
  by name, canonical JSON (ensure_ascii, compact separators, sorted keys), SHA-256 hex.
  metabrowser's EngineVersion carries scope_fingerprint and registry_fingerprint as
  separate values at the checkout in attic; whichever side moves, one recipe has to be
  written down and both have to reproduce it.

Explicitly NOT an oracle, and the reason this is a replay rather than a race: running two
live engines against one changing tree compares incomparable observation moments, and the
dual walk perturbs what is being measured.

Any difference the replay finds is documented or eliminated, not averaged.

## Notes

null

EXACT-HEAD AGREEMENT ADDITIONS from FDU d19b0ce / MetaBrowser 0577bb1 (2026-08-25): the fixture must now pin (1) the exact scope digest bytes for hidden_allowlist, max_depth, and max_files, including a non-ASCII allowed hidden component; (2) the observable max_files stop at a directory boundary case that distinguishes FDU whole-directory overshoot from the Python strict cap; and (3) FIFO/socket/special-object boot and live replacement, where the MetaBrowser provider view exposes only file, directory, and symlink. These cases decide design across both repositories; do not normalize the answers in an adapter-side mirror. Implementation owners are fdu-vfyw, reopened fdu-97dd, and fdu-bjhy.

FDU FIXTURE UPDATE at d0a6a6a (2026-08-25). The fdu half of the special-object scenario now exists: boot, refresh, replacement, watch, three-kind rows, and tally conservation under native exclusion. The bead remains open for the combined two-engine packet/replay and exact shared identity bytes; the reference adapter still includes fdu-only scope components, tracked on fdu-vfyw.

EXACT-HEAD CAP SEMANTICS DECISION at FDU PR #47 71772fc / MetaBrowser #74 0577bb1 (2026-08-25). FDU now keeps max_files as a global retained-index cap; MetaBrowser PythonInventoryHandle.rewalk_subtree gives each rewalk a fresh max_files budget and can grow beyond the original total. The FDU rule is the cleaner physical bound, but engines cannot claim agreement until the shared fixture records deletion/free-slot and subtree-rewalk cases and MetaBrowser deliberately adopts the same invariant. Do not hide this difference in the adapter. The fixture must also require Partial(Budget) and a typed ResourceStop issue at the same terminal data clock when a live or reconciliation upsert is refused.
