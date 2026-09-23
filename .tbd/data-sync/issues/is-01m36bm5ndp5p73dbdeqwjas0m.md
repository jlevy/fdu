---
type: is
id: is-01m36bm5ndp5p73dbdeqwjas0m
title: Diagnose the Windows parallel_equivalence churn stall (reconcile error under churn)
kind: bug
status: open
priority: 2
version: 1
labels:
  - stack-followup
dependencies: []
parent_id: is-01m36ajmynyejms4zkynsyrvcz
created_at: 2026-09-23T05:25:42.701Z
updated_at: 2026-09-23T05:25:42.701Z
---
fdu-ex5k added a test-only guard so a panic in reconciling_a_tree_that_is_changing_underneath_converges_once_it_settles surfaces instead of hanging; the cause of the Windows stall (job 107041656026) is unproven. The #115 reviewer found no production cause in #115 and suspects scan::reconcile returning a hard Err under churn on Windows (delete-pending directories or ACCESS_DENIED racing read_dir). If so, a vanished directory during reconcile should become a recorded issue, not Err. Next occurrence will now print the real error; reproduce on native Windows CI with repeats.
