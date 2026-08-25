---
type: is
id: is-01m0prgbradma67z3j1wfyh8r7
title: "Spec: fdu for interactive clients — the metabrowser contract"
kind: epic
status: open
priority: 1
version: 68
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
refs:
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/47#pullrequestreview-5017522830
    at: 2026-08-25T09:57:38.755Z
  - kind: pr
    url: https://github.com/jlevy/fdu/pull/44#issuecomment-5408704238
    at: 2026-08-25T09:57:38.756Z
labels: []
dependencies: []
child_order_hints:
  - is-01m0prgyer27hqzdm2pvjx44qg
  - is-01m0prgyv1eq0g0mzgntn1p4n6
  - is-01m0prgz8hvk0rsm1edqgwty5d
  - is-01m0prgznztcx92ypmk6aanszf
  - is-01m0prhbvmd38p7eqffrg08nr6
  - is-01m0prhc835eec71rccdfe50zb
  - is-01m0prhcmhj09n41s1zae35yhm
  - is-01m0prhd1jfqr4gvtaq96hbwd7
  - is-01m0prhpj01wa15ypxm0er2q6s
  - is-01m0prhpzxz5tx9t61nj95aegw
  - is-01m0prhqd27m471dn47yt973k0
  - is-01m0pt934wtpzs87mtmg2hxhsg
  - is-01m0pt93kk5pytsjrb0v5wrweq
  - is-01m0pt9he483bx4et2eykcdp1j
  - is-01m0pt9j1mbn3za7fyym7pyr9t
  - is-01m0ptezmtmkn04mh1f1rwgdxb
  - is-01m0ptvnmg9p0s1qh9174hpcv3
  - is-01m0qs0msk75k8r89b44vqqjnz
  - is-01m0qs0nnjz9z4mkw35ahwydvs
  - is-01m0qs197189wae43fmqxs82bs
  - is-01m0qs19pg77zfmd3s2kg7k905
  - is-01m0racc8tf20x27jjhh35vh5q
  - is-01m0raccjvpde63hx884rkmq5d
  - is-01m0raccwe6ywyac61ezhxk2ws
  - is-01m0racd5dxjfx1g5e0dsfay8q
  - is-01m0rahh7entj80k486sxs5k45
  - is-01m0rw7a5ref8h8b8b17kxccbs
  - is-01m0rw7bvxtw87tgde30emgs56
  - is-01m0rw7cddvwh9vetyxkmgrvsm
  - is-01m0rw7d4h3t49rwvk11cmk5xb
  - is-01m0t5szzjt8kr7yqkzg78cxhm
  - is-01m0t5t1ghzmetfs4qjbrzx44r
  - is-01m0t5t249szzrfqrng85e36me
  - is-01m0t5t2sa2rn3qm3m4dycv7hv
  - is-01m0tdy6k1kfkywsy4f8kga870
  - is-01m0tdy76e3ndzcsdwf8m6j8sq
  - is-01m0tdy7tfq528ftppqfpteypv
  - is-01m0tdy8b6h17fqk7mqge56svh
  - is-01m0tdy8swsdre8d15s96wx4km
  - is-01m0tdy9ceep2byvbtyvwc2vky
  - is-01m0tdy9tx76dachmfcgrq5r3a
  - is-01m0te8vfk0w5tp9337vkth4wy
  - is-01m0tf9pw281n7hsaenyp0aq3e
  - is-01m0tfbt0djry86tn8bd03ydr9
  - is-01m0tra5gw0ap6nbbzgt7egvr4
  - is-01m0tra6d42b6r8vsecjnh36e8
  - is-01m0vrk2scfs6rfsm2hfnwkz50
  - is-01m0vx6h8tfjyqjaxmt67nabrp
  - is-01m0vx6yw0f8bddcwggvk2ha0p
  - is-01kzqn502680awzhvddzntq32d
  - is-01m0w5fbs0n1xv9rxrmrp79mda
created_at: 2026-08-23T07:31:34.794Z
updated_at: 2026-08-25T09:57:38.756Z
---
Root epic for the interactive-client integration spec: partitioned tallies (tag planes), the embedder watch contract, the session integration shape, and the adoption proof. Each capability lands engine-first and clears the parity harness. The measured basis and the requirement-by-requirement contract map are in the spec.

## Notes

EXECUTION QUEUE (2026-08-24 design review; supersedes prior orderings; no counts by
policy -- `tbd list --parent fdu-u7vo --all` is the map).

The review verified all eight open findings against the code, found two new issues
(fdu-0778 gitignore bind-walk, fdu-g0n4 max_size), and settled a design on each bead's
notes. Cross-cutting decisions:
- `Cursor { session, clock }` is one type (fdu-325q) reused by reads (fdu-91ru) and
  watch batches (fdu-vfx7): land 325q before both.
- fdu-ycyy's load ordering and fdu-0778's bind-from-index are one ordering: header ->
  validate -> install types -> materialize -> bind Path-tier from index -> tag.
- fdu-kbir follows fdu-91ru (same function body; avoid double churn).
- fdu-xyvu is elevated from nice-to-have: MetaBrowser's contract REQUIRES hidden
  pruning with an exact-name allowlist, so it gates adoption.

Queue, one bead per commit, gate green each time:
1. fdu-ycyy  2. fdu-0778  3. fdu-37dv  4. fdu-325q       (P0 authority/data-loss)
5. fdu-91ru  6. fdu-vfx7                                  (observation boundary)
7. fdu-662n                                               (bounded results; bpp9 done)
8. fdu-kbir  9. fdu-hfdw                                  (binding evidence)
10. fdu-g0n4  11. fdu-xyvu                                (contract-required selection/scope)
then the mapped chain: fdu-jxs0 -> fdu-fltq, fdu-4o0m -> fdu-sgp7, fdu-pxfz (note:
`unignored` population = total minus plane, so conservation per dimension is the test),
fdu-7rwf, fdu-n7mv; then fdu-vfyw as the acceptance slice; fdu-m893/fdu-ey9q ride where
their deps clear; fdu-n4gn/fdu-2ig2 on the quiet host.

Consumer-side issues found in the same review (stale bd1dcf8 doc pins; mandatory-CPU
vs exact-or-absent; watcher-gap axis ambiguity; singular registry fingerprint; reset vs
gap recovery cost) are consolidated in a comment on metabrowser PR #74 rather than
tracked as fdu beads.

ALIGNMENT UPDATE at exact PR #47 head a3960fb / MetaBrowser 68eeaac (2026-08-24; all 19 FDU checks green). MetaBrowser resolved the five consumer-side ambiguities and hardened the as_of requirement. Existing beads now carry the implementation consequences: fdu-5yqb reopened to remove watcher_gap from coverage vocabulary; fdu-91ru remains open for the structured envelope, state-clock atomicity, and caller-pinned as_of; fdu-kbir has the final exact-or-absent CPU contract; fdu-fltq distinguishes consumer reset from provider gap recovery; fdu-vfyw pins canonical combination of type/tag/reducer fingerprints and cross-engine acceptance. Do not create parallel alignment beads; update these owners as #47 advances.

MONITOR UPDATE: PR #47 advanced to 558461a (2026-08-24; all 19 checks green). The lossless WatchBatch direction is correct and fixes removed-directory and filtered-dirty loss, but exact-head review reopened fdu-vfx7: terminal cursor is sampled after apply_next under a separate guard and can skip a concurrent commit; asyncio teardown blocks the event loop while the worker can be waiting on that same loop, then does not assert termination; state/work remain absent. fdu-jxs0 and fdu-fltq notes now record the retag-clock and reset-vs-gap interactions. Review the delta before treating the green carrier commit as adoption-ready.

MONITOR UPDATE: PR #47 advanced from 558461a to exact head 56dcf56 (2026-08-24; all 19 checks green). fdu-662n and fdu-g0n4 correctly landed positive child-page limits and inclusive max_size for adapter translation; fdu-hfdw now charges filtered report index visits. fdu-kbir was reopened after exact-head review: the GIL detach is correct and CPU absence aligns, but conversion_ns stops before projection conversion and omits the public Python Bundle/model conversion, materialized_bytes is an incomplete second O(output) estimate rather than a documented binding-payload count, and the public threading test would pass under GIL serialization. Existing fdu-vfx7, fdu-91ru, fdu-jxs0, fdu-fltq, and the remaining adoption gates stay open.

MONITOR UPDATE: PR #47 advanced from 56dcf56 to exact head 715f748 (2026-08-25). Removing WatcherGap from coverage is aligned and fdu-5yqb is correctly closed; py.detach on the actual read and exact-or-absent CPU are also aligned. Adoption is still blocked: fdu-vfx7 remains open because a direct producer commit can interleave between multiple watcher/reconcile deltas and be skipped by the returned terminal cursor, and long watch intervals let asyncio teardown return with its worker still alive. fdu-kbir was reopened again because model_ns double-counts extension conversion, one Work type now has conflicting total-versus-projection semantics, binding-payload accounting is non-exhaustive, and the timing-based GIL proof fails the macOS/Python 3.14 CI job. fdu-91ru, fdu-jxs0, and fdu-fltq remain open for the coherent envelope/data clock and recovery distinction.

MONITOR UPDATE: PR #47 advanced from 715f748 to exact head 278457a (2026-08-25; all 19 checks green). The 278457a performance follow-up fully resolves the reviewed fdu-kbir remainder: direct non-overlapping phase timings, distinct ProjectionCost, structural payload charging with an independent oracle, and a forced GIL-progress test; fdu-kbir stays closed. ac38584 correctly makes answer-affecting state a clocked journal commit, but its live delivery is incomplete, so fdu-jxs0 was reopened: reconcile begin/finish commits do not reach the sink used by WatchBatch, and PyWatch replaces actual state clocks with the terminal batch clock. fdu-vfx7 remains open because Retagged's unbounded directory vector bypasses all_dirty and journal retention, async teardown can still return with a live worker, and resulting provider state/work are absent. fdu-fltq remains open for provider-gap versus consumer-reset semantics; fdu-91ru remains open for typed issues/lifecycle and caller-pinned as_of. Remaining adoption gates are unchanged.

MONITOR UPDATE: PR #47 advanced from 278457a to exact head fad3d2f (2026-08-25; all 19 checks green). The complete journal slice fixes the prior concurrent-producer cursor omission and makes reconcile state visible through Session; the short internal async pull plus liveness check and typed teardown error close the teardown finding. Adoption remains blocked on mapped owners: fdu-vfx7 now records that a journal-only producer commit cannot wake an idle Session, plus the unbounded Retagged carrier and missing resulting state/work; fdu-jxs0 remains open for the public reconcile sink and exact per-transition clocks; fdu-fltq remains open because provider gaps still become reset and consumer truncation emits the MetaBrowser-invalid reset + all_dirty combination. fdu-91ru remains open for typed lifecycle/issues and caller-pinned as_of.

MONITOR UPDATE: PR #47 advanced from fad3d2f to exact reviewed head 7aaaf84 (2026-08-25; all 19 checks green and mergeability clean). The review accepts caller-pinned as_of, the typed coherent read envelope, exact state-transition clocks, journal-only delivery, bounded retagging, reset-versus-recovery semantics, dirty query kinds, read-side GIL release, async teardown, and the promoted-plane equivalence work; fdu-jxs0 and fdu-fltq stay closed. Exact-head formal review https://github.com/jlevy/fdu/pull/47#pullrequestreview-5015288678 reopened fdu-vfx7 because WatchBatch still lacks the complete resulting provider state at its cursor, the v1 invalidation path still materializes entry rows across PyO3, and public watch work omits binding/model conversion cost. It also reopened fdu-91ru because only child listings have native continuation: filtered-tree and catalog pages cannot satisfy MetaBrowser's mandatory bounds, advancing cursors, and lossless remainders without an unbounded FFI result or Python mirror. These are Phase 2 adoption blockers; no parallel beads were created.

MONITOR UPDATE: PR #47 advanced from 7aaaf84 through exact reviewed head b8ead94 (2026-08-25). Review https://github.com/jlevy/fdu/pull/47#pullrequestreview-5015624718 reopened fdu-xyvu because every macOS bulk scan/reconcile path bypasses hidden admission; current macOS engine and wheel CI fail on retained hidden paths. It reopened fdu-vfyw because the reference embedder uses an incompatible identity encoding and scope/semantic partition, discards directory continuation/remainder, labels child paging as recent-row paging without expected-version pinning, and materializes watch entry rows while dropping terminal state/work. fdu-91ru and fdu-vfx7 carry those underlying gates. Follow-up https://github.com/jlevy/fdu/pull/47#pullrequestreview-5015634122 accepts the unrelated b8ead94 pure-path vendored/documentation tag fold; fdu-n7mv stays closed. fdu-jxs0 and fdu-fltq remain correctly closed.

MONITOR UPDATE: PR #47 advanced from b8ead94 to exact reviewed head d19b0ce (2026-08-25; all 19 checks green and mergeability clean) against MetaBrowser #74 exact head 0577bb1 (all 5 checks green). Accepted: ff210d0 closes the macOS hidden-admission defect; eaae030 now carries terminal engine state coherently on delta ranges and WatchBatch; d19b0ce supplies a bounded native multi-path refresh whose binding samples the terminal clock after run-facts/analysis, which satisfies the revised completion-boundary contract even though one commit is only an optimization. Remaining A3 watch carrier work stays on fdu-vfx7, and native flat filtered-tree/catalog paging stays on fdu-91ru.

New exact-head adoption drift is deduplicated as follows. fdu-vfyw remains open because ff210d0 still hashes a different scope component set and ASCII encoding than MetaBrowser 0577bb1. fdu-97dd is reopened because whole-directory budget overshoot produces a different inventory for the same max_files than the strict Python stop. fdu-7sou is reopened and moved under this epic because FDU rejects watching with max_depth/max_files, while every normal MetaBrowser handle supplies both and requires one provider-owned observer. fdu-bjhy owns three-kind special-object exclusion across native projections/rollups/watch. fdu-kl7r carries the shared fixtures. These are design seams to reconcile on both owned sides, not compatibility shims.
