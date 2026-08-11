---
type: is
id: is-01kzs52rb1a9gekzex8hc7a91x
title: fdu.stream/1 schema-bump test to match the report schema
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
created_at: 2026-08-11T19:34:07.456Z
updated_at: 2026-08-11T19:34:07.456Z
---
report_format.rs has the_schema_constant_is_the_versioning_promise pinning REPORT_SCHEMA to fdu.report/1, so an unversioned change fails a test. STREAM_SCHEMA has no equivalent: fdu.stream/1 can be changed today without anything going red. The spec's Testing Strategy asks for fixtures on both. Add the matching constant assertion and a record fixture, so the stream schema carries the same promise the report schema does.
