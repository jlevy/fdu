---
type: is
id: is-01m2kfrnx7bjpgyef1e89eg16m
title: "PR #63 review PR63-SCOPE-1: a hand-built Index's control limits are not cross-checked against its scope at save/load"
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01m2kfr5a9x4sksa32yq00fh8k
created_at: 2026-09-15T21:32:30.502Z
updated_at: 2026-09-16T06:08:24.554Z
closed_at: 2026-09-16T06:08:24.553Z
close_reason: "826ed93: save and install_controls, which every load goes through, compare the table's limits with the scope's ignore-rules fingerprint and refuse a mismatch with Error::ControlLimitsOutsideScope; Index::new_with_config builds both halves from one ScanConfig. Verified fixed by PR #63 delta review 5218970886 at 9105768, which traced every production constructor."
resolution: null
duplicate_of: null
---
PR #63 review PR63-SCOPE-1 at 1fd71a9: index.rs:1588, control.rs:190, snapshot.rs:211,718, lib.rs:382. Index::new_with_scope installs default limits under any scope fingerprint; snapshot::save and load never compare the table's limits with the scope. Fix: compare observed_ignore_rules_fingerprint(limits) with the scope at save and load, refuse a mismatch with a typed error, and give a public constructor the limits.
