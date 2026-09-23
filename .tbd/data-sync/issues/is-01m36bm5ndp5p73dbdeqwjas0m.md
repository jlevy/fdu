---
type: is
id: is-01m36bm5ndp5p73dbdeqwjas0m
title: Diagnose the Windows parallel_equivalence churn stall (reconcile error under churn)
kind: bug
status: closed
priority: 2
version: 3
labels:
  - stack-followup
dependencies: []
parent_id: is-01m36ajmynyejms4zkynsyrvcz
created_at: 2026-09-23T05:25:42.701Z
updated_at: 2026-09-23T08:07:03.171Z
closed_at: 2026-09-23T08:07:03.169Z
close_reason: "Root cause: root device read through consistent observation; fixed in 95ad163b + fc352c83 on #98; native Windows Test and full matrix green on #98 and #117"
resolution: null
duplicate_of: null
---
fdu-ex5k added a test-only guard so a panic in reconciling_a_tree_that_is_changing_underneath_converges_once_it_settles surfaces instead of hanging; the cause of the Windows stall (job 107041656026) is unproven. The #115 reviewer found no production cause in #115 and suspects scan::reconcile returning a hard Err under churn on Windows (delete-pending directories or ACCESS_DENIED racing read_dir). If so, a vanished directory during reconcile should become a recorded issue, not Err. Next occurrence will now print the real error; reproduce on native Windows CI with repeats.

## Notes

2026-09-23: root cause found. With the fdu-ex5k guard, #117 Windows CI run 35828659282 failed visibly instead of hanging: reconcile returned Io { path: <walk root>, 'file changed while Windows metadata was observed' }. Every walk read the root's device through #98's consistent observation (two reads of FILE_BASIC_INFO/tag/BY_HANDLE_FILE_INFORMATION that must agree); a root whose children are being created changes its times constantly, so any walk of an active tree could fail outright on Windows. Fix 95ad163b on #98 (codex/release-windows-validity): root_device reads the volume serial once (used by the four walk entry points, the subtree boundary and opened discovery); query retries until two consecutive observations agree (max 4 reads). Merged up through #114; awaiting independent review and native Windows CI.
