---
type: is
id: is-01m2h6pb3g04vwcpsrgwammw7m
title: "PR #55 review PR55-DUR-2: format retirement per release, not per store; Library APIs N/A is wrong"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
parent_id: is-01m2h6nn916177k0wrc92zmkge
created_at: 2026-09-15T00:15:27.854Z
updated_at: 2026-09-15T00:32:14.806Z
closed_at: 2026-09-15T00:32:14.804Z
close_reason: "df47165: per-store format retirement with a refusal for newer and retired older formats; Library APIs DO NOT MAINTAIN (Attrs field, fdu-core unreleased); snapshot FORMAT_VERSION bump stated in Delta Accounting, slice 2, and file formats"
resolution: null
duplicate_of: null
---
PR #55, delta review 5204152578. Plan @4727de0 :314-318, :354-355 make retirement a property of a release and give a refusal message only for a newer format; :346 says Library APIs N/A although slice 2 adds a field to pub struct Attrs (engine_contract.rs:81-95, re-exported lib.rs:99 at dda7e6a); the snapshot FORMAT_VERSION bump from the fixed-width record (snapshot.rs:237-242) is stated only in open question 1. Fix: per-store retirement with an older-format refusal, DO NOT MAINTAIN for library APIs (fdu-core unreleased), state the bump in slice 2.
