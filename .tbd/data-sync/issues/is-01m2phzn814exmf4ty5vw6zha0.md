---
type: is
id: is-01m2phzn814exmf4ty5vw6zha0
title: "Phase 2 item 1: measured values: per-analyzer records, one definition per metric"
kind: epic
status: open
priority: 0
version: 16
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - content
  - design
  - release
dependencies:
  - type: blocks
    target: is-01m2pmrn6ka9kt7f4kcjcmvn8c
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
child_order_hints:
  - is-01m2pye7p7e3rn725762gtyaak
  - is-01m2pye80k4gebgn994ewchs53
  - is-01m2pye8b6mrzcdsq27ffct9d4
  - is-01m2pye8ptnr6019rcb15g4w69
  - is-01m2pye91dbqqaf1qy70cem3mt
created_at: 2026-09-17T02:08:59.648Z
updated_at: 2026-09-17T05:47:39.610Z
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 1: Measured Values", moved here when the item became beads (PR #78 at `e52383d4`; locators verified at `5f2d36d`). The commits are this bead's children, P2.1.1 to P2.1.5; their blockers carry the ordering, so this bead only groups them and closes when they do.

The engine currently loses measured work in two ways, both explained by the evidence:
- The `Unsupported` gate (`content/content_analysis.rs:302-313`) returns
  `MetricValues::default()` for a Haskell file under `code` (no arm in
  `content/content_code_metrics.rs:105-124`), which accounts for every loss in the
  matrix: 5 physical lines, 14 raw words, and the 666-to-656 logical-word difference,
  because logical words are derived after summing.
- Paragraphs are counted only when `collect_logical` is on
  (`content/content_basic_metrics.rs:170-178`), which only `words` enables
  (`content/content_analysis.rs:189`), so `lines` and `code` report a false zero; code
  files’ paragraphs are zeroed (`content/content_analysis.rs:289`), and Markdown
  paragraphs are overwritten by the prose analyzer (`content/content_analysis.rs:328`).

| File | Function or type | Change |
| --- | --- | --- |
| `content/content_model.rs` | `MetricSlotId` (`:18`, unused internally but re-exported at `content.rs:24`, so its removal is public) | Replace with `MetricDef { name, owner: AnalysisSet, analyzer: AnalyzerId, doc }` and `METRICS`: `lines` owns `physical_lines`, `blank_lines`, `nonblank_lines`, `raw_words`; `code` owns `code_lines`, `comment_lines`, `code_blank_lines`; `words` owns `logical_words`, `paragraphs`, `visible_words`, `visible_logical_words`, `document_words` |
| `content/content_model.rs` | `MetricValues` (`:313-410`), `FileAnalysis` (`:432-449`) | `BasicMetrics`, `CodeMetrics`, `WordMetrics`; `FileAnalysis { fingerprint, bytes, detection: ContentDetection, lines: AnalyzerOutcome<BasicMetrics>, code: Option<AnalyzerOutcome<CodeMetrics>>, words: Option<AnalyzerOutcome<WordMetrics>>, error }`; drop `classification`, `profile`, `provenance`; add `document_word_stats` |
| `content/content_analysis.rs` | `analyze_open_file` (`:161-341`), `record` (`:382-400`), `analyzed_record` (`:343-354`), `io_record` (`:356-372`), `count_coverage` (`:402-412`), `AnalysisReport` (`:24-47`) | Remove the early return; unsupported code becomes a code outcome while lines and words continue; paragraphs come only from `words`; probe results go into `detection`; file-level reasons apply to every requested unit |
| `content/content_index.rs` | `MetricTally` (`:16-45`), `ContentRollUp` (`:49-94`), `commit`, `prepare` | Tally per unit; delete `by_type` and `by_family`, which nothing reads; a public API removal, since `content.rs:20` re-exports `ContentRollUp` |
| `content/content_cache.rs` | `put_record` (`:204-216`), `parse` (`:218-282`), `put_metrics`/`read_metrics` (`:332-376`), classification codecs (`:428-509`) | Write detection and one block per requested unit |
| `index.rs` | `pending_analysis_candidates`, `apply_analysis` | Drop `record.profile`; compare against the name-only classification |
| `query/query_report.rs` | `metric_summary` (`:1380-1565`), `MetricRow` (`:627-662`), `document_words` (`:1577-1583`), `share_value` (`:1567-1574`), `ShareMetric` (`:602-623`) | Group by `index.classify` only (delete `:1390-1392`); aggregate per unit; the share metric comes from the request (`CodeLines` only with `code`; `DocumentWords` only with `words`, otherwise the new `RawWords`); `document_words` and `pages` return `Option` and name their source |
| `examples/perf_probe.rs` | `attach_content_summary` (`:1400-1438`) | Rebuild the digest per unit and version it as `fdu-content-summary-v2` |

**Call sites:** `lib.rs:541-551` and tests at `lib.rs:1286-1395` and
`index.rs:7934-7984`; content tests in `content/content_index.rs:304-325`,
`content/content_cache.rs:625-818`, and `content/content_analysis.rs:453-849`; the text,
JSON, and YAML writers; `query.rs:18` and `content.rs:20-26` re-exports;
`crates/fdu-py/python/fdu/_models.py` `MetricValues` and `_metric_row` (`:573-616`,
`:924-948`); `scripts/content-selfcheck.mjs:85-114`.

**Tests:** a new `crates/fdu-core/tests/metric_independence.rs` over Rust, Python,
Haskell, Markdown, `.txt`, a C++ `.h`, an extensionless shebang script, an extensionless
`%PDF` file, invalid UTF-8, a NUL file, and a generated marker: for every `METRICS`
entry on every row and total, the cold value is identical under every analyzer set
including its owner and absent under every set that does not, and grouping is identical
under every set. Update
`code_profile_partitions_supported_languages_and_marks_others_unsupported`
(`content/content_analysis.rs:653`),
`deep_detection_drives_named_consumers_and_report_evidence` (`:576`), and the paragraph
and document tests (`:453`, `:710`, `:787`). Goldens in `cli-content.tryscript.md`
change where values were erased or files regroup.

**Commits:**
1. The `METRICS` table and a test that every emitted metric key appears in it once.
2. Per-analyzer records and sidecar layout, read back into today’s wire shape.
3. Name-based grouping and content detection counts.
4. Presence follows the request under `fdu.report/7`, after the answer model’s walks.
5. The metric-independence test.

**Risks:** `paragraphs` is owned by the `words` unit, which covers two analyzers; if
`words` is ever split, the metric splits too.
Per-unit coverage maps break readers of a flat `coverage.analyzed`. Regrouping is
user-visible: extensionless scripts leave `languages`, and C++ headers named `.h` group
as C. The content digest in the performance ledger changes at this commit.

## Original scope (before the implementation detail)

The design problem behind fdu-gija. The analysis request is index state, set when a tree is opened,
while every other report dimension (views, selection, size, ignored) is a Query parameter projected
over retained state. Reports therefore present whatever the content tier holds, and anything that lets
the tier hold more than was requested (containment reuse, a long-lived index, cache-only reads) leaks
cache history into answers. Coverage is one value per file for a whole analyzer set, so one analyzer's
Unsupported erases another's results, and "a wider record holds every metric a narrower request would
recompute" is not true of the data model.

Goal: carry the requested analyzer set on the report request; record coverage and metrics per analyzer
(fdu-ky5m, fdu-7dj6); project metrics, derived words and analysis metadata to the request; restore
containment reuse only on that basis; prove warm-equals-cold for every analyzer-set pair on every surface.
The spec needs a design revision before implementation (the content-metrics plan is in done/).

## Notes

2026-09-17 (PR #78 review): Decisions: the analyzer registry owns metric definitions and coverage semantics; per-analyzer results and coverage; unrequested metrics absent (no key in JSON/YAML, None in Python); classification groups by name and registry only, content probes reported under detection and used only to pick the analyzer; document_words exists only with the words analyzer, and pages names raw_words otherwise.
