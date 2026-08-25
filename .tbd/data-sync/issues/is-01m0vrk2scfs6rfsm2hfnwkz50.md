---
type: is
id: is-01m0vrk2scfs6rfsm2hfnwkz50
title: "Two-engine agreement oracle: conformance packet and recorded-observation replay"
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-25T06:09:16.076Z
updated_at: 2026-08-25T06:09:16.076Z
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
