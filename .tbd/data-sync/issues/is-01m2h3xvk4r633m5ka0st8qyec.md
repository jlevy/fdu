---
type: is
id: is-01m2h3xvk4r633m5ka0st8qyec
title: "PR #60 review PR60-DOCS-1: no CHANGELOG entry for the removed gitignore build feature"
kind: bug
status: in_progress
priority: 2
version: 2
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2h3xgn1f3azaqbcvh9mzxas
created_at: 2026-09-14T23:27:08.387Z
updated_at: 2026-09-14T23:27:30.879Z
---
PR #60 at df43bdd: CHANGELOG.md is unchanged. Missing a breaking entry: gitignore build feature removed from fdu-core and fdu, fdu default now ["watch"], a dependent's features = ["gitignore"] fails to resolve. Missing the behaviour change for consumers who built without it: control input applied, or refused with ControlStateNotObserved on a non-observing scope, instead of UnsupportedScanConfig; ignore_rules_fingerprint 0 -> 2 so controls-on snapshots miss once. Base CHANGELOG.md:81-82 (#57, 2b52f87) says 'every index in a build without the gitignore feature'; correct it.
