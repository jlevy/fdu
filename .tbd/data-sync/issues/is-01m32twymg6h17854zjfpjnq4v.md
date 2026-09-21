---
type: is
id: is-01m32twymg6h17854zjfpjnq4v
title: "PR #99 review R2: panic message offers an opt-out that path deliberately ignores"
kind: bug
status: closed
priority: 3
version: 3
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m32h6ea5cf5dd7rzf7ea0jdh
hold: null
hold_until: null
created_at: 2026-09-21T20:35:41.327Z
updated_at: 2026-09-21T20:57:27.128Z
started_at: 2026-09-21T20:36:00.263Z
closed_at: 2026-09-21T20:57:27.128Z
close_reason: "Fixed in bf8f6241: wait helpers split by whether the watch is established. establish_watch honours the opt-out; wait_established (unit) and wait_for (integration) return the ops/change directly and on silence panic that the watch was established so this is a lost event, with no opt-out offered. The dead else-return arms are gone. Verified by mutation: keeping the warm-up and writing the subject elsewhere makes created_files_arrive_as_verified_upserts panic at watch.rs:1157 with the new message after 60s."
resolution: null
duplicate_of: null
---
Low from https://github.com/jlevy/fdu/pull/99#issuecomment-5764971659. watch.rs:1136-1140 and tests/watch_session_integration.rs:145-150: on the allow_host_opt_out == false path the message still offers FDU_TEST_ALLOW_NO_NATIVE_WATCH=1, which is ignored there because the watch was already established. Fix: say the watch was established so silence is a real result. Also remove the dead else-return arms the suggestion lists.
