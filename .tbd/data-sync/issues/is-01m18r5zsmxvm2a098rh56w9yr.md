---
type: is
id: is-01m18r5zsmxvm2a098rh56w9yr
title: Control-table charge model inflates ~6.5x and pays repeatedly for identical sources
kind: task
status: closed
priority: 1
version: 8
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - scale
  - control-state
  - stack-followup
  - release
dependencies: []
parent_id: is-01m18r51dyvcp3bzw8yca45ph7
created_at: 2026-08-30T07:12:14.643Z
updated_at: 2026-09-15T20:00:41.304Z
closed_at: 2026-09-15T20:00:41.303Z
close_reason: "eed4f62: identical .gitignore contents are stored and charged once, keyed by FNV-1a identity with a byte comparison so a collision never shares a matcher; removal releases a content with its last holder, pinned by a fixed-seed property test. ~/wrk (4,830 files, 980 distinct) charges 4.10 MiB, from 13.84. PR #63."
resolution: null
duplicate_of: null
---
retained_source_cost (crates/fdu-core/src/control.rs:337) charges:
  64 + path_bytes + source.len()*2 + (newlines+1)*64 + slash_count*24

Measured over all 3256 .gitignore files in ~/wrk: 1.53 MiB of real source charges 9.93 MiB - 6.47x inflation. The dominant term is (newlines+1)*64, an estimate of compiled-matcher memory, worth ~5.4 MiB of the 9.93.

Identical .gitignore files compile to identical matchers, and ControlIdentity already carries an FNV-1a fingerprint of the source. ~/wrk has 3256 files but only 946 distinct contents (3.44x redundancy; one content appears 235 times). Storing distinct content once and keying directories to it drops the charge from 9.93 MiB to 3.81 MiB - a 61.7% cut - with no policy change and no loss of the exact-bytes contract.

Note dedup alone is NOT sufficient: 3.81 MiB against a 4 MiB cap is no margin, and ~ is larger than ~/wrk. Pair with a larger, liftable budget.

Cost to weigh: refcounted shared content makes removal harder to reason about in a module whose stated virtue is that deletion is an ordinary state transition.

Acceptance: retention is deduplicated by fingerprint; removal semantics stay exact and tested; measured retention on ~/wrk drops by the predicted order.

## Notes

2026-09-14 (triage at c0511e9): unchanged; formula at `control.rs:355-366@c0511e9`, per-directory ownership at `control.rs:73-94`, fingerprint present but unused for sharing (`control.rs:61-67`). Follow-on with fdu-okne once fdu-1onj makes the bound non-fatal.

2026-09-14 DECISION (user, supersedes the default-off decision recorded earlier the same day): .gitignore information is built into the tool and the library, and is rolled up by default on every surface: CLI reports, --watch, library open, fdu.open/fdu.scan/fdu.report, and opened roots. Each request can turn it off (--no-gitignore on the CLI, read_controls=False in the library and Python). The typed 'not observed' answer from #57 stays, for requests that opt out. The CLI shows split totals, for example '1.2 GB (340 MB ignored)', plus --exclude-ignored and --only-ignored filters. Prerequisites before the default flips: fdu-1onj (the control budget degrades to partial instead of aborting), fdu-okne (a liftable bound named in the error), fdu-szkg (charges deduplicated by fingerprint), and a speed check against main with controls on. Prerequisite for the default flip.

2026-09-15 DECISIONS (user), PR A design (plan: scratchpad/reviews/plan-gitignore-default-on.md, section 7):
Q1: a crossed control budget is a coverage note with exit 0. Sizes stay exact; only ignore classification is partial. Text output names the directories whose rules were not loaded and the flag that raises the budget. JSON carries an ignore_rules coverage field.
Q2: the budget is part of snapshot scope, mixed into ignore_rules_fingerprint. Raising it cold-scans once.
Q3: the 16 KiB per-line guard can be lifted by the same budget flag (the recommendation was fixed; the user chose liftable, per 'every bound is liftable'). Over the guard it degrades and names the guard and the flag.
Q11: bump snapshot FORMAT_VERSION in PR A, so snapshots carry refused rules and the budget.
