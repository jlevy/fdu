---
type: is
id: is-01m2kfrnx7bjpgyef1e89eg16m
title: "PR #63 review PR63-SCOPE-1: a hand-built Index's control limits are not cross-checked against its scope at save/load"
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m2kfr5a9x4sksa32yq00fh8k
created_at: 2026-09-15T21:32:30.502Z
updated_at: 2026-09-15T21:32:30.502Z
---
PR #63 review PR63-SCOPE-1 at 1fd71a9: index.rs:1588, control.rs:190, snapshot.rs:211,718, lib.rs:382. Index::new_with_scope installs default limits under any scope fingerprint; snapshot::save and load never compare the table's limits with the scope. Fix: compare observed_ignore_rules_fingerprint(limits) with the scope at save and load, refuse a mismatch with a typed error, and give a public constructor the limits.
