---
type: is
id: is-01m2pprs3ns0br59gv4xpaqfjt
title: Code comments contradict corrected architecture docs (snapshot bucket, 'requested' profile)
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - review
dependencies: []
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
created_at: 2026-09-17T03:32:37.108Z
updated_at: 2026-09-17T03:32:37.108Z
---
From PR #78 review suggestions: snapshot.rs:70-72 says the snapshot stores the bucket an entry was assigned (records hold parent, kind, name, attrs only, :657-666); query_report.rs:684 and content_index.rs:185 call the profile 'requested' when it is the held set. Fix with the request and stored-state model work.
