---
type: is
id: is-01m6yn4418zpvm5mpxnfdb3cds
title: "Selection coverage: selected and scanned tallies and the note"
kind: task
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
labels: []
dependencies:
  - type: blocks
    target: is-01m6yn44yyd0z9gz709ag6hyks
parent_id: is-01m6ycagweqhppqznjzwagp365
created_at: 2026-11-08T22:40:34.344Z
updated_at: 2026-11-08T23:30:13.022Z
closed_at: 2026-11-08T23:30:13.022Z
close_reason: Done in b805920d
resolution: null
duplicate_of: null
---
Spec: Selection coverage. When selection predicates (kind, patterns, size, time, ignored state) narrow the scanned set, the report carries selected and scanned tallies (files, bytes, allocated) on the envelope and ends with one note 'selected N of M files, X of Y'; omitted when everything was admitted; suppressed by --quiet; scanned is the unfiltered root roll-up (several roots: their union). Python model fields.
