---
type: is
id: is-01m2yh8kc79nw7bn6k6xw8g3bp
title: Address remaining 0.1 correctness blockers after performance review
kind: task
status: in_progress
priority: 1
version: 26
spec_path: docs/project/specs/active/plan-2026-09-22-fdu-alpha-correctness-stack.md
delegate: codex-alpha-coordinator
labels: []
dependencies: []
child_order_hints:
  - is-01m2yhjb6r67n23jera0t03ebv
  - is-01m2yhjbkazg1zds9x9janfd5m
  - is-01m2yhjbz0hfj930q39b01rmap
  - is-01m2yhjcawjwehkfdxrejrhe64
  - is-01m2yhtwjg6j0vjyjf1v9sehd5
  - is-01m2ymchsj7j6ayg8j46kc1v00
  - is-01m2ymmc4c03yvnc10myb2jmk6
  - is-01m2yrppkm6hp1gbsytzmcmnme
  - is-01m2ysreqwf3xczbbevbv4ve06
  - is-01m2ysrer2fjkw6qa1z57epf46
  - is-01m2yt3kzcakn6nka88x1z48ja
  - is-01m35xrf4gyhcj9q42reapfeen
  - is-01m35yv020z6cayddk13mnhmpt
  - is-01m3637s9hez8e47f6tw7svrhp
  - is-01m363y726frrgtqtp2z8xanpm
hold: null
hold_until: null
created_at: 2026-09-20T04:30:19.526Z
updated_at: 2026-09-23T04:12:13.915Z
started_at: 2026-09-20T04:31:31.377Z
---
Audit remaining release correctness against the current #92 stack, reconcile stale or already-fixed beads, implement confirmed defects in coherent slices with subagents, review changes, validate and publish PRs. Preserve unrelated directory-rollup work and existing core-model ownership. Final candidate verification remains distinct from publishing.

## Notes

Reviewed sub-slices are now draft PR #98 (Windows validity, fe06b11b; native CI exposed avoidable allocation regressions under repair) and #99 (ignore matcher cfe38b8e plus explicit permission/native-watch preconditions a86dbfb3). Both are based on exact PR92 head 937f9445. Report/7/Python and provenance/watch integration continue in separate owned worktrees. Root independently compared iterative deep JSON decoding against standard JSON across 2012 cases. Known memory limitations remain tracked under b2qz (Markdown) and 1zb6 (long code lines). No merge or release authorization.

Integration now includes report7/stream2 serializers and typed Python models, per-analyzer outcomes with encoding coverage, stale/error/provenance fixes, controls-off snapshot projection across routes, Windows ChangeTime validity, and Git pattern corrections. Root CLI golden validation passed165cases before the final projection merge. Original PR91 and PR92 exact-head full make check both passed. PR99 all19CI checks green. PR98 all3 full platform matrices passed at96f16922; final Windows performance harness found an independent-oracle digest mismatch under investigation. Root review of state follow-up found concurrent root passes can undercount newer omitted errors and falsely recover Complete, plus fatal multipath abort can clear unvisited issues; state agent assigned fixes and regressions. Integrated full gate and final Astra route review pending. b2qz/1zb6 remain documented analyzer memory limits; no release/merge performed.

2026-09-22 resumed alpha correctness delivery: plan PR110 in native stack111 follows reviewed PR99 and corrected PR98. PR98 head fb9fef49 now passes all23CI checks including all3fullplatformmatrices; independent Astra oracle review has no blockers. Metric layer PR112 ba3e9564 published and under independent review. Answer fixes7edd44b0/2e5425b0 pass36format tests,56Python model tests, native/public smoke and57exact parser cases. State followup7af768c8 fixes old-pass error resurrection with bounded scope arbitration, focused tests/clippy passing; independent review pending. PR103 flat diagnostics red regressions established; fix under validation. Full Phase2.3 execution Plan remains required and assigned next. Old worktrees/root checkout preserved. No merge or release.

2026-09-22 stabilization checkpoint: native stack111 has nine reviewed dependent PRs99→98→110→112→113→114→115→116→117. Final production at4464308e (test-only followups throughff2b07da) fixes typed answers, per-analyzer values, shared writers, scope/state reconciliation, watch handoff, Plan/Delivery persistence, controls-off projection, and directory-query composition. New fdu-bwo2 canonical own-listing withdrawal and explicit DirectoryIncomplete publication reviewed independently; Linux full matrix16272cases zero deviations at446, release rehearsal35814796698 all9jobs passed. FinalLinux parity artifact recorded by CI and independently reviewed: same24 existing deviations,10additional passing sessions. Final local broad gate has passed core828 tests/integrations/CLI175/format64; remaining gate and finalall-platform CI still running. Currentheadff2b fullPI/release35815753312 pending; intermediate layer fixture/prerequisite cleanup being batched. User requested switching remaining subagent work toSol to conserve budget; model handoff completed. AllPRs remain unmerged; implementation beads remain inprogress for final validation/publication acceptance. No performance acceptance or release publication claim.

2026-09-22 publication checkpoint: after exact ff2b07da path-independence run35815707617 passed Linux16,272, macOS16,272, Windows14,382 cases with zero false verdicts and zero exception lists on each, the reviewed test-fixture/parity batch fast-forwarded the native stack: PR113 876bd39c, PR114 0266b95c, PR115 7a38900b, PR116 ee8cff4c, PR117 d0fbbe93. PR117 includes reviewed plan evidence and merge order. Production behavior did not change from ff2b07da; corrections are golden portability/guide text, exact Linux parity artifact lines, and pedantic test literals. New exact-head CI and final acceptance remain pending; no PR merge, release publication, or performance integration occurred.
