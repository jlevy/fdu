---
type: is
id: is-01kzs927a7dfykfw74pvt3x9aj
title: Two public Provenance types after the composable-CLI and progressive-results merge
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-11-fdu-progressive-results.md
labels: []
dependencies: []
created_at: 2026-08-11T20:43:44.326Z
updated_at: 2026-08-11T20:43:44.326Z
---
Merging the progressive-results work into the composable-CLI branch leaves the crate exporting two distinct public types named Provenance. fdu::Provenance (crates/fdu/src/types.rs) is the per-value provenance introduced by the progressive-results design: where a value came from, when it was observed, whether it is final. fdu::query::Provenance (crates/fdu/src/query/report.rs) is report-level: scan_started_at, generated_at, source, complete, errors. Both are legitimate in their own module and the compiler is satisfied, so nothing fails - which is exactly why it needs a decision rather than discovery. A consumer writing 'use fdu::Provenance' gets the value-level one and will not notice until the field names disagree. Options: rename the report one to ReportProvenance, rename the value one to ValueProvenance or Origin, or nest them so the paths carry the distinction. The crate is unpublished (0.0.1, publishing gated by fdu-9cf0) so this is cheap now and a semver event later. Decide before publish.
