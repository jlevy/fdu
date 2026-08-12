---
type: is
id: is-01kzvcfz2m5y717jr4b0z0kh39
title: Fix stale dirty marker in checkout build versions
kind: bug
status: closed
priority: 2
version: 2
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzq1vhvfdyrrhmz3343qh5nr
created_at: 2026-08-12T16:22:09.235Z
updated_at: 2026-08-12T16:48:51.558Z
closed_at: 2026-08-12T16:48:51.556Z
close_reason: "Restored Cargo's recursive package tracking with cargo:rerun-if-changed=. while retaining HEAD/ref tracking. Verified on clean commit 8525e2b: version was g8525e2bd9, adding a package-source edit rebuilt to g8525e2bd9.dirty, and removing the edit rebuilt back to g8525e2bd9 without moving HEAD."
---
PR #5 review found that build.rs narrows Cargo rerun tracking to HEAD/ref, so source edits and cleanups can leave the embedded .dirty suffix stale. Add a deterministic regression test and preserve accurate dev-build version identity without harming published-crate builds.
