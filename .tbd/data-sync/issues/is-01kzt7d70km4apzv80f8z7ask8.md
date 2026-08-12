---
type: is
id: is-01kzt7d70km4apzv80f8z7ask8
title: "PR#6 R8: stale-temp test plants a name the writer can never generate"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-12T05:34:01.745Z
updated_at: 2026-08-12T05:37:16.443Z
closed_at: 2026-08-12T05:37:16.443Z
close_reason: "Fixed and mutation-checked in faeb7df; disposition posted to PR #6"
---
crates/fdu/src/snapshot.rs. a_stale_temporary_does_not_block_a_later_write plants the pre-entropy name shape .tmp.{pid}.0 while create_temp_file now emits .tmp.{pid}.{entropy}.{seq}, so the paths cannot match, AlreadyExists never fires and the retry loop the test narrates is never exercised. My own comment claiming the collision is not reachable from one process is also wrong: planting the current entropy and the next NEXT_TEMP_FILE value forces a real collision. Medium.
