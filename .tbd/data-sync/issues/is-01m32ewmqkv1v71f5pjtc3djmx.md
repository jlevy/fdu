---
type: is
id: is-01m32ewmqkv1v71f5pjtc3djmx
title: "Path-independence is one-sided: a cache that never serves passes every case"
kind: bug
status: in_progress
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
delegate: codex-alpha-coordinator
labels: []
dependencies: []
parent_id: is-01m31hvhfvefh5ka5z4fsymdta
created_at: 2026-09-21T17:05:48.275Z
updated_at: 2026-09-23T02:54:06.369Z
---
Verified by execution, 2026-09-21. The harness compares warm answers against cold ones, so a cache that simply stops serving is indistinguishable from a correct one.

`runner.py:265-267`: under `only`, any failure whose stderr starts with `snapshot is not usable` is recorded `refused` and always allowed; under `auto`/`read-only` a miss just scans cold and compares `same`.

Mutant (`serves_snapshot` always `Refuse`): the subset ran 884 cases with ZERO failures — `refused 248, same 615, stale 21` against a baseline of `refused 158, same 668, stale 58`. Only the cross-route outcome check (`runner.py:590-600`, Python surface only) notices over-refusal, and only when routes disagree with each other.

What actually catches a dead cache is the ~12 unit tests in `lib.rs`, `execution.rs` and `stored_state.rs` asserting `OpenPath::CacheOnly`, hit counts, and `a_snapshot_serves_exactly_the_identity...` — that is, the "mechanism" tests.

Consequence for policy: mechanism tests and answer tests guard opposite directions and both are necessary. An earlier framing in this session disparaged mechanism tests; taken seriously that would have deleted the only guard against a cache that never serves. Correct the framing wherever it was written down.

Fix direction: give the harness a serve-rate expectation per policy, so a run where cache-only refuses far more than the recording fails rather than passing quietly.

## Notes

Implemented in codex/alpha-cache-contract at 47a86ed7, pending composition above the execution layer. Adds 18 exact refresh-to-cache-only positive controls across CLI and Python cache-reading routes, requiring cache_only source and stale freshness. A dead cache, a cold fallback, a failed seed, and false freshness all fail retained tests. The subset now includes W_code before mutations. The production conformance judge rejects any registered exception, so recording/classifying a regression cannot produce a passing gate. Historical registry analysis remains available without waiving production acceptance.

All 35 harness unit tests, Ruff checks/formatting, and diff checks pass. Independent Astra review found no remaining issue. Runtime validation against the final built engine and regeneration of the empty registry from all three CI platform recordings remain pending. The delivery specification and harness README state these obligations; no waiver was removed by guesswork.

Published as PR #116. Exact-head runtime evidence: https://github.com/jlevy/fdu/actions/runs/35810763705 at b2b8dd23 judged all 16,272 Linux, 16,272 macOS, and 14,382 Windows cases allowed, including all 18 positive exact-cache serving controls on each platform. Merged the three judged JSON artifacts using registry.py, then retired both empty classes in 0bb6dd83; no semantic waivers remain. All 36 current harness unit tests pass. The composed PR #117 still requires final make check, cross-lint, exact-head CI and release rehearsal; bead remains in progress pending acceptance and merge.
