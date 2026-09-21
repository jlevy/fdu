---
type: is
id: is-01m2zpkypg9xvwkfy54mah4gxn
title: Restore timers must cover the whole sidecar load before H121 is judged; document load_content_cache rollback
kind: task
status: closed
priority: 2
version: 4
labels: []
dependencies: []
created_at: 2026-09-20T15:23:08.879Z
updated_at: 2026-09-21T08:20:31.397Z
closed_at: 2026-09-21T08:20:31.397Z
close_reason: "Shipped on #104 (https://github.com/jlevy/fdu/pull/104) at 743d2b9c / 4d78558e vs main a290aedc. CI run 35576573158 green on ubuntu/macos/windows including Performance evidence. exp-116 change_pct is the paired −2.158%; commit quoted as 984e4618; digest/regime/errata corrected; ledger+report regenerated. R3 name negatives, valid-rename load control, and H138 sharing guard restored; apply timer starts at candidates.remove. peak_rss_bytes prints as bytes (exp-117 377.5→339.4 MiB). Local rust-test passed except the known parallel_equivalence flake (not fixed)."
---
From the independent pre-merge verification of PR #91 (2026-09-20). Do this BEFORE judging H121, whose rule is "a named restore stage >= 50% of restore phase time".

- V91-3: in `content_cache.rs` (~:237-248) `candidates.remove` and the fingerprint compare now run between the decode timer and the apply timer, in neither bucket. At H112 (exp-109, commit 0ec489e7) the apply timer wrapped the whole loop. So exp-109's "apply 63.3%" is not comparable with today's `content_sidecar_apply_us`, the four restore rows cannot sum to ~100% of `load_content_cache`, and H121 would be judged on rows that under-count the load (~133k Path-hash removes per load; H116's -15.65% user CPU suggests it is not small). Start the apply timer before `candidates.remove`, or add a fifth bucket, and note the definition change in the H121 registry row. No product impact.
- V91-7: a failure detected late in the record stream now calls `clear_content()` (~:228, ~:264) while a failed header parse leaves the index untouched. No in-crate caller can observe it (every index reaching `load_content` starts with `content: None`), but an outside caller that runs `analyze_index` and then passes a corrupt sidecar to the public function sees a populated tier wiped. One sentence in the doc comment (~:150-156).
- V91-9 (note): exp-117 measured `bbfd7d1c`; `e667b739`/`c2487f38` later reworked the hot restore loop (two Option checks per record with counters off) without re-measurement. Acceptable; re-measure opportunistically on a quiet host.

## Notes

Context posted for Linux handoff on PR #94: https://github.com/jlevy/fdu/pull/94#issuecomment-5757234985
