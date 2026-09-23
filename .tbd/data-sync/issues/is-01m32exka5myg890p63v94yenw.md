---
type: is
id: is-01m32exka5myg890p63v94yenw
title: "Harness gaps: subset detects content re-widening only via --cache only, and the registry can absorb regressions"
kind: bug
status: in_progress
priority: 1
version: 2
delegate: codex-alpha-coordinator
labels: []
dependencies: []
parent_id: is-01m31hvhfvefh5ka5z4fsymdta
created_at: 2026-09-21T17:06:19.589Z
updated_at: 2026-09-23T01:51:14.487Z
---
Verified in an adversarial review, 2026-09-21.

## Subset coverage (by execution)

A re-widening mutant (any stored sidecar serves any enabled analyzer set) makes the subset flag 35 unregistered keys — but every one is a `--cache only` request: `warm/cli-report/only/W_all/-/{a_lines,a_code,a_words,a_lines_v_documents,a_code_langs_name_lim1}` and the same five across six mutations.

Under `auto` and `read-only` the analysis pass re-prepares the tier (`index.rs:3454`) and the answer comes out cold-correct, so `make check`'s subset detects a content re-widening ONLY on cache-only requests. Adequate for the command line, but `matrix.py:301-322` sets the subset's `selfwarm=False` and `mutation_warmers=(W_default, W_all)`, so W_code then a_lines after a mutation — the Unsupported-Haskell case that was fdu-gija's first symptom — runs only in the full matrix.

Baseline subset: 884 cases, 6.2s command-line only.

## Registry gaming (by reading)

- `registry.py:203-206` compares only the generalized path SET, so a registered key whose values regress further is absorbed silently. The `unverified-subtree` entries on `mutation/.../unreadable/...` list essentially every metric path, so any content regression on those 17 keys is invisible.
- Class membership for a new key is free-text at record time. `merge` assigns `unclassified`, but nothing checks a hand-typed class actually fits.
- "Class has no entries" and "registered case not executed" only run under `full` (`registry.py:208-215`), so the subset cannot catch a stale waiver.
- `verify` does not check that `known-violations.toml` matches CI's recording, so a local `--record` plus hand classification passes.
