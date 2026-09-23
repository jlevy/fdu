---
type: is
id: is-01m36ajmynyejms4zkynsyrvcz
title: "Full review of alpha correctness stack #99→#117 and 0.1.0 readiness (2026-09-22)"
kind: task
status: closed
priority: 0
version: 20
labels:
  - release
dependencies: []
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
child_order_hints:
  - is-01m36bkdy0gnqnn7z6jpd41ggv
  - is-01m36bkeecpc2ek6jcxdqnjqmx
  - is-01m36bkewjczea7hnthhkkv8wg
  - is-01m36bkfbxe11a31bv8s5zc9rr
  - is-01m36bkft17tw7vbat35hsw790
  - is-01m36bkg9fw21m72dpqvqbhpnk
  - is-01m36bkgrfzj433sbyr81dtbjs
  - is-01m36bkh85fc1z49e7fp2fcsz5
  - is-01m36bkhqcnfts27ggfc5pwtfd
  - is-01m36bm3zx2e7jv2pykpm6b0zb
  - is-01m36bm4k9grwp72hthb58bq0b
  - is-01m36bm560f11v69fyb6xqewz1
  - is-01m36bm5ndp5p73dbdeqwjas0m
  - is-01m36bm65pk7ftex7xvb3qcxn3
created_at: 2026-09-23T05:07:24.243Z
updated_at: 2026-09-23T09:42:56.171Z
closed_at: 2026-09-23T09:42:56.162Z
close_reason: "Done 2026-09-23. Every layer of stack #111 reviewed (published), findings fixed with independent delta reviews, dispositions posted; correctness stack merged atomically (9989c5ad), perf stack #94/#97 (d69c705b), #105 (4993b099), #109 (7e06e5a4), runbook fix #118 (0059ddd5); #96/#103 closed as superseded. Final main tree == the tree that passed an uninterrupted make check + cross-lint; main CI and the full 3-platform path-independence matrix green. Post-merge QA: correctness runbook (after fixing its report/7 scripts in #118) and integration runbook sections 5, 6, 9 passed; see fdu-tyvq notes. Remaining alpha work is on fdu-yv36, fdu-w76x, fdu-xgjx (conformance pointer), fdu-tyvq, fdu-d237, fdu-f7cf and the maintainer beads."
resolution: null
duplicate_of: null
---
Independent review of every layer of GitHub stack #111 (#99, #98, #110, #112-#117) through its head, audit of prior review dispositions on #94/#97/#105/#109/#96/#103, and an updated list of what remains before a publishable 0.1.0 alpha. Findings are returned to the coordinator and published only with the maintainer's approval.

Starting state (2026-09-22): all 15 open PRs CLEAN; #110-#117 have NO posted review comment (bodies cite private 'Astra' reviews); #99/#98 have review + disposition + alpha re-review; perf #94/#97/#105/#109 and #96/#103 have alpha re-reviews. No uninterrupted make check on the composed head is claimed. Host disk: ~400 MiB free, so no local build is possible this session; 34 GiB in /private/tmp, largely ~60 alpha worktrees.

## Notes

## Status (2026-09-22): review complete; nothing is merge-ready as a whole yet

Independent reviews done for #110, #112-#117 (Fable for engine layers, Opus for harness/plan), plus a disposition audit of #99/#98/#94/#97/#105/#109/#96/#103 and a commit-pair port-fidelity diff of #103→#117 (faithful; one deliberate strengthening: per-directory failure attribution).

Release-side findings so far (coordinator, verified at #117 head adc39d24):
- CHANGELOG has an [Unreleased] section above an unpublished [0.1.0] (placeholder date 2026-09-16). Fold Unreleased into 0.1.0 at tag time; nothing was ever published as 0.1.0.
- docs/project/release-notes/0.1.0.md does not mention directory filters or the list/paths/long presentations that #117 adds, or the machine default switching to list.
- README headline performance (dumac +11.3% etc.) was measured before this stack (fdu-y5xr closed). The plan states that the correctness stack makes no performance claim, and nobody has measured #117 against main, although #115 rewrites execution.rs/scan.rs routing. Re-measure after the perf layers compose.
- No uninterrupted `make check` on the composed head (scoped runs only). The host cannot run one now: ~400 MiB free, and 34 GiB in /private/tmp (~60 alpha worktrees).
- Maintainer-only blockers still open: fdu-atbv (repo security settings), fdu-znb0 (SSH signing key), fdu-o5st (registry accounts/2FA/release env).

## Layer review results (independent reviewers, read-only, verified at source)

Disk: freed 6.2 GiB with `cargo clean` in the main checkout (user-approved); the host briefly hit ENOSPC mid-review.

| PR | Verdict | Material findings |
| --- | --- | --- |
| #99 | ready | prior findings fixed; full make check recorded at head |
| #98 | ready after #99 | prior findings fixed; fdu-6act/fdu-ns3o can close on merge |
| #110 | fix in #117 | plan boxes checked on scoped-run evidence (R110-1); boxes checked while beads open (R110-2); core-models "Current Delivery" stale (R110-3) |
| #112 | ok in stack | R112-1 totals-only test is fixed by #113 (row-level, Types view only); R112-3 native dict None vs 0 is retired by #113 a7122fd2 |
| #113 | merge after fixes | R113-2 Medium CONFIRMED (ran gate parser): YAML 1.1 reads e3/E10/e+5 as numbers → fdu-ju6w; R113-3 Medium: text output never states errors_omitted → fdu-peil; R113-1 quadratic record_walk_errors is already fixed higher in the stack (normalize first) |
| #114 | merge after fix | R114-1 Medium: issue stamped with pass start epoch; a later-finishing concurrent pass drops the issue while the path stays Partial → partial with no error |
| #115 | merge after fixes | R115-1 Medium PLAUSIBLE: refresh persistence keyed to this pass's mutation, so a partial-then-clean refresh sequence can leave the snapshot behind the index; R115-2 Medium: Route::Opened accepts workers/order/accept_partial and ignores them; R115-3 Medium: admission stated in three places |
| #116 | merge after fixes (Low) | R116-1 must_serve passes when oracle and cache read fail identically; stale "registered known violation" wording in workflows |
| #117 | merge after doc fixes | port of #103/#96 faithful; R117-1 "pre-1.0 schema policy" cited but not written (release-process.md says field change bumps schema; /7 is itself new in this stack, so write the unreleased-schema rule); R117-2 CHANGELOG lost render()->Result API break; R117-3 two --modified-since examples lack --kind file |
| #105 | NOT ready | R2/R3/R4 open (fdu-03w2, fdu-27aj, fdu-yprs) |
| #94/#97/#109 | ready standalone | composition work fdu-qx0e, fdu-8fax not applied |
| #96/#103 | close as superseded after #117 | nothing lost in port |

Windows parallel_equivalence stall (fdu-ex5k): #115 reviewer finds no production cause in #115; likely `scan::reconcile` returning Err under churn on Windows, surfaced as a hang by thread::scope. Guard converts it to a visible failure; root cause open.

## Beads filed (children of fdu-1p8b)

Blocking fdu-xgjx (conformance): fdu-ju6w (YAML e3), fdu-peil (silent error truncation), fdu-goge (R114-1 epoch), fdu-yonh (opened route ignores delivery), fdu-ftsh (admission in three places), fdu-9vkc (schema rule + CHANGELOG), fdu-9mn0 (plan/bead reconciliation).
Blocking fdu-tyvq (release e2e): fdu-n2ok (one uninterrupted make check + cross-lint on the final composed candidate), fdu-w76x (CHANGELOG/release notes fold), fdu-yv36 (perf of stack vs main, then README re-measure).
Non-blocking: fdu-9kk8 (refresh persistence owed, plausible), fdu-gzd1 (harness must_serve + wording), fdu-39m3 (Windows churn stall root cause), fdu-d237 (Low findings).
Already tracked: #105 R2/R3/R4 fdu-03w2/fdu-27aj/fdu-yprs; composition fdu-qx0e, fdu-8fax; #98 S1-S4 fdu-1zrz/m3x5/hrjz/twry; fdu-2udc no-kind coverage decision; maintainer-only fdu-atbv, fdu-znb0, fdu-o5st.
Close on merge: fdu-6act, fdu-ns3o (#98), fdu-zjjt, fdu-f9fv, fdu-93e8, fdu-c22r (#117); close #96/#103 as superseded after #117 lands.

No reviews were posted to GitHub from this session (pending maintainer approval).

## Disk cleanup (2026-09-22, user-approved)
Removed 37 clean fdu worktrees under /private/tmp (branches kept in the repo) and stale build dirs; /private/tmp 34 GiB → 8.8 GiB. Kept: fdu-alpha-review-t5q4I6/integration (detached unpushed c5b759ee, cited by fdu-qx0e/fdu-8fax), fdu-correctness-state (uncommitted index.rs/scan.rs edits), and every audit/log/patch file in fdu-alpha-review-t5q4I6/ and fdu-alpha-stack-PnTm5M/ (reconciliation-audit.md, integration-*.patch, pi-* matrix recordings). Freed blocks are held by the Time Machine local snapshot 2026-09-22-233130 until macOS thins it.
