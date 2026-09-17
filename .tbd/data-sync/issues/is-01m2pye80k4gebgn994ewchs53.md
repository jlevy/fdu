---
type: is
id: is-01m2pye80k4gebgn994ewchs53
title: "P2.1.2: Per-analyzer records and sidecar layout, read back into today's wire shape"
kind: task
status: open
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - core-models
dependencies:
  - type: blocks
    target: is-01m2pye8b6mrzcdsq27ffct9d4
  - type: blocks
    target: is-01kzyp8vpx1852y9sjnb7k6w2g
parent_id: is-01m2phzn814exmf4ty5vw6zha0
created_at: 2026-09-17T05:46:40.531Z
updated_at: 2026-09-17T05:47:44.448Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 1: Measured Values", commit 2. Locators were verified at `5f2d36d`; they drift as earlier commits land, so re-find code by function name.

**Change**

- `content/content_model.rs`: `MetricValues` (`:313-410`) and `FileAnalysis` (`:432-449`) become `BasicMetrics`, `CodeMetrics`, `WordMetrics` and `FileAnalysis { fingerprint, bytes, detection: ContentDetection, lines: AnalyzerOutcome<BasicMetrics>, code: Option<AnalyzerOutcome<CodeMetrics>>, words: Option<AnalyzerOutcome<WordMetrics>>, error }`; drop `classification`, `profile`, `provenance`; add `document_word_stats`.
- `content/content_analysis.rs`: `analyze_open_file` (`:161-341`) removes the `Unsupported` early return (`:302-313`), so unsupported code becomes a code outcome while lines and words continue; paragraphs come only from `words` (today `collect_logical`, `content/content_basic_metrics.rs:170-178`, enabled only by `words` at `content/content_analysis.rs:189`; code files' paragraphs zeroed at `:289`; Markdown overwritten at `:328`); probe results go into `detection`; file-level reasons apply to every requested unit. Also `record` (`:382-400`), `analyzed_record` (`:343-354`), `io_record` (`:356-372`), `count_coverage` (`:402-412`), `AnalysisReport` (`:24-47`).
- `content/content_index.rs`: `MetricTally` (`:16-45`) and `ContentRollUp` (`:49-94`) tally per unit; delete `by_type` and `by_family`, which nothing reads (a public removal: `content.rs:20` re-exports `ContentRollUp`); `commit`, `prepare`.
- `content/content_cache.rs`: `put_record` (`:204-216`), `parse` (`:218-282`), `put_metrics`/`read_metrics` (`:332-376`), classification codecs (`:428-509`) write detection and one block per requested unit. Either share the sidecar format bump with P1.2.3 or take the next one; both land before 0.1.0.
- `examples/perf_probe.rs`: `attach_content_summary` (`:1400-1438`) rebuilds the digest per unit as `fdu-content-summary-v2`.
- Call sites: `lib.rs:541-551` and tests at `lib.rs:1286-1395` and `index.rs:7934-7984`; content tests in `content/content_index.rs:304-325`, `content/content_cache.rs:625-818`, `content/content_analysis.rs:453-849`; `query.rs:18` and `content.rs:20-26` re-exports; `scripts/content-selfcheck.mjs:85-114`.

**Tests**

- Update `code_profile_partitions_supported_languages_and_marks_others_unsupported` (`content/content_analysis.rs:653`) and the paragraph and document tests (`:453`, `:710`, `:787`).
- Goldens in `cli-content.tryscript.md` change where values were erased (the Haskell lines and words, the 666-to-656 logical words).

**Done when**

- `make check` passes; from P1.1.3 on it includes the path-independence subset, which reports no unregistered difference.
- Every golden diff is read and attributed to this commit; none is regenerated blind.

Risks: `paragraphs` is owned by the `words` unit, which covers two analyzers; the content digest in the performance ledger changes at this commit.
