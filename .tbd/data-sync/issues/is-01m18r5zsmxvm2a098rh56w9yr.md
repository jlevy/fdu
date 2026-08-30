---
type: is
id: is-01m18r5zsmxvm2a098rh56w9yr
title: Control-table charge model inflates ~6.5x and pays repeatedly for identical sources
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - scale
  - control-state
dependencies: []
parent_id: is-01m18r51dyvcp3bzw8yca45ph7
created_at: 2026-08-30T07:12:14.643Z
updated_at: 2026-08-30T07:12:14.643Z
---
retained_source_cost (crates/fdu-core/src/control.rs:337) charges:
  64 + path_bytes + source.len()*2 + (newlines+1)*64 + slash_count*24

Measured over all 3256 .gitignore files in ~/wrk: 1.53 MiB of real source charges 9.93 MiB - 6.47x inflation. The dominant term is (newlines+1)*64, an estimate of compiled-matcher memory, worth ~5.4 MiB of the 9.93.

Identical .gitignore files compile to identical matchers, and ControlIdentity already carries an FNV-1a fingerprint of the source. ~/wrk has 3256 files but only 946 distinct contents (3.44x redundancy; one content appears 235 times). Storing distinct content once and keying directories to it drops the charge from 9.93 MiB to 3.81 MiB - a 61.7% cut - with no policy change and no loss of the exact-bytes contract.

Note dedup alone is NOT sufficient: 3.81 MiB against a 4 MiB cap is no margin, and ~ is larger than ~/wrk. Pair with a larger, liftable budget.

Cost to weigh: refcounted shared content makes removal harder to reason about in a module whose stated virtue is that deletion is an ordinary state transition.

Acceptance: retention is deduplicated by fingerprint; removal semantics stay exact and tested; measured retention on ~/wrk drops by the predicted order.
