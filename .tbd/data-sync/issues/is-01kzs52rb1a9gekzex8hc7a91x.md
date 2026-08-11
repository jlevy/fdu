---
type: is
id: is-01kzs52rb1a9gekzex8hc7a91x
title: fdu.stream/1 schema-bump test to match the report schema
kind: task
status: closed
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzq1vqasq5aqd5g07357h76t
created_at: 2026-08-11T19:34:07.456Z
updated_at: 2026-08-11T20:49:17.433Z
closed_at: 2026-08-11T20:49:17.432Z
close_reason: Whole rendered records pinned for upsert, remove, and invalidate in both json and text, including the absent-not-null contract for optional fields. A constant-only assertion would have left the record shape free to move.
---
report_format.rs has the_schema_constant_is_the_versioning_promise pinning REPORT_SCHEMA to fdu.report/1, so an unversioned change fails a test. STREAM_SCHEMA has no equivalent: fdu.stream/1 can be changed today without anything going red. The spec's Testing Strategy asks for fixtures on both. Add the matching constant assertion and a record fixture, so the stream schema carries the same promise the report schema does.
