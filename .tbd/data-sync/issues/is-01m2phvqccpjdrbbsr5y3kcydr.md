---
type: is
id: is-01m2phvqccpjdrbbsr5y3kcydr
title: "Content analysis answers depend on cache history: narrower --analyze served from a wider sidecar"
kind: bug
status: open
priority: 0
version: 4
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - release
  - content
dependencies:
  - type: blocks
    target: is-01m2phzegm4b3scda7d1xq3gnm
parent_id: is-01m2phzn814exmf4ty5vw6zha0
created_at: 2026-09-17T02:06:50.763Z
updated_at: 2026-09-17T02:57:32.502Z
---
The same `--analyze` request gives different answers depending on which request warmed the content sidecar. Reproduced on the 0.1.0 release candidate (5f2d36d) through the CLI and the Python API.

1. Unsupported record. `--analyze code` records a code file with no line-of-code counter (e.g. Haskell) as `Unsupported` with zero metrics, discarding the line and word counts. A later `--analyze lines` is served that record: 12 lines cold vs 4 lines warm, coverage `unsupported: 1`.
2. Unprojected wider records. After `--analyze all`, `--analyze lines` reports metrics nobody requested (code/comment lines, logical and visible words, paragraphs), `document_words` changes (13 cold vs 12 warm, so documents-view shares and ranking can change), and `analysis.analyze` reports `["lines","code","words"]` although the field and Python's `AnalysisMetadata.analyze` are documented as what the report requested.

Root cause: #37 (2aa7da1, 2026-08-21) changed sidecar reuse from equality to containment and kept wider tiers in memory (`ContentIndex::prepare`, `save_content_cache`), but the consumers still read the content tier as if it were the request. The report has no access to the requested analyzer set: `Query` carries none, `query::report(index, query, provenance)` reads `index.content()`, `ContentReportMetadata.profile` is filled from the stored set, and `metric_summary` adds `record.metrics` and derives document words from `record.profile`. Coverage is one value per file for the whole analyzer set, so containment's premise ("a wider record holds every metric a narrower request would recompute") is false for `Unsupported`. Tests asserted the optimization (no file opened, hits) but never that a warm answer equals a cold one; 21 of 27 content goldens run `--cache off`.

Fix options for 0.1.0 (user decision pending):
- Exact-match reuse: a sidecar answers only the identical analyzer set. Restores the invariant the report was built on; narrowing re-reads files.
- Request-scoped projection now: carry the requested set to the report and project metrics, derived words, and metadata to it; Unsupported records never answer a set without `code`.

Either way: a warm-versus-cold equivalence test over every analyzer-set pair (CLI and Python), and docs that state the chosen reuse rule (design principles, cache design, engine architecture, usage, README, CHANGELOG). Branch claude/release-e2e-fixes has the Unsupported slice and a test that exposed the wider case.

## Notes

2026-09-17: Not to be point-patched (maintainer). Resolved by the explicit core models plan: per-analyzer
records and one definition per metric (this epic), the request model giving the reader the requested analyzers,
and the path-independence test. The Unsupported-record rule on branch claude/release-e2e-fixes is superseded.
