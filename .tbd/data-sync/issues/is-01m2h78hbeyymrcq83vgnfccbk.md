---
type: is
id: is-01m2h78hbeyymrcq83vgnfccbk
title: "PR #57 review PR57-L89E-1: an evicted continuation fails the whole read"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m2h77qemaay1jkhbhfzh35me
created_at: 2026-09-15T00:25:24.077Z
updated_at: 2026-09-15T00:28:47.274Z
closed_at: 2026-09-15T00:28:47.273Z
close_reason: "88801b0: per the fdu-l89e rule, a consumed or evicted continuation refuses only its projection as ProjectionRefusal::ContinuationUnavailable (Python RefusalReason.CONTINUATION_UNAVAILABLE); a foreign or never-issued token still fails the read; 7ef0d16 re-recorded the golden. Holds at c141285; an_evicted_or_consumed_token_refuses_and_a_token_never_issued_fails and the opened-root goldens pass."
resolution: null
duplicate_of: null
---
PR #57 review https://github.com/jlevy/fdu/pull/57#pullrequestreview-5200240760, P2. At 7b804df, crates/fdu-core/src/opened/continuation.rs:125-130, :153-155; opened/read.rs:121: a continuation this root issued and later evicted (MAX_CONTINUATIONS) or consumed failed the whole ReadRequest as ContinuationUnavailable, a state-dependent failure the fdu-l89e decision keeps out of whole-read failures.
