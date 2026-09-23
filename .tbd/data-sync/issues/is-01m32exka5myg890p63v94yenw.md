---
type: is
id: is-01m32exka5myg890p63v94yenw
title: "Harness gaps: subset detects content re-widening only via --cache only, and the registry can absorb regressions"
kind: bug
status: in_progress
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
delegate: codex-alpha-coordinator
labels: []
dependencies: []
parent_id: is-01m31hvhfvefh5ka5z4fsymdta
created_at: 2026-09-21T17:06:19.589Z
updated_at: 2026-09-23T02:54:07.005Z
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

## Notes

Implemented in codex/alpha-cache-contract at 47a86ed7, pending composition above the execution layer. Adds 18 exact refresh-to-cache-only positive controls across CLI and Python cache-reading routes, requiring cache_only source and stale freshness. A dead cache, a cold fallback, a failed seed, and false freshness all fail retained tests. The subset now includes W_code before mutations. The production conformance judge rejects any registered exception, so recording/classifying a regression cannot produce a passing gate. Historical registry analysis remains available without waiving production acceptance.

All 35 harness unit tests, Ruff checks/formatting, and diff checks pass. Independent Astra review found no remaining issue. Runtime validation against the final built engine and regeneration of the empty registry from all three CI platform recordings remain pending. The delivery specification and harness README state these obligations; no waiver was removed by guesswork.

Published as PR #116. Exact-head runtime evidence: https://github.com/jlevy/fdu/actions/runs/35810763705 at b2b8dd23 judged all 16,272 Linux, 16,272 macOS, and 14,382 Windows cases allowed, including all 18 positive exact-cache serving controls on each platform. Merged the three judged JSON artifacts using registry.py, then retired both empty classes in 0bb6dd83; no semantic waivers remain. All 36 current harness unit tests pass. The composed PR #117 still requires final make check, cross-lint, exact-head CI and release rehearsal; bead remains in progress pending acceptance and merge.
