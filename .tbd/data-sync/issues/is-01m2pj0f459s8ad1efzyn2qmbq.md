---
type: is
id: is-01m2pj0f459s8ad1efzyn2qmbq
title: "Phase 2 item 2: answer model: one typed value per document, serialized by every writer"
kind: epic
status: closed
priority: 0
version: 24
spec_path: docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md
labels:
  - output
  - design
  - release
dependencies:
  - type: blocks
    target: is-01m2pmrn6ka9kt7f4kcjcmvn8c
  - type: blocks
    target: is-01m2yh8kc79nw7bn6k6xw8g3bp
parent_id: is-01m2pmr9aftb3vzfyjyanndny3
child_order_hints:
  - is-01m2pye9bnamf5vezd6tg2bwz2
  - is-01m2pye9p8df0h0rzch4fq37wy
  - is-01m2pyea2jz71fks8v1zep7c21
  - is-01m2pyeaeaj7mqe04gk45j9xjd
  - is-01m2pyeas8avwnpbxs4tnr9rq0
  - is-01m2pyeb3yz78d3d94dv78dv4g
  - is-01m2pyebe93530evdeaw8tcxh6
  - is-01m2pyebrt6j2ghh4c4vcdvyxg
created_at: 2026-09-17T02:09:26.148Z
updated_at: 2026-09-23T08:14:06.952Z
closed_at: 2026-09-23T08:14:06.952Z
close_reason: "Implemented in the alpha correctness stack (#99, #98, #110, #112-#117), independently reviewed per layer with published reviews, dispositions and delta reviews; merged to main in 9989c5ad on 2026-09-23 after an uninterrupted make check and cross-lint on the gated tree and green CI on every layer. Acceptance of the conformance gate remains on fdu-xgjx; the analyzer-set containment deferral remains on fdu-7dj6."
resolution: null
duplicate_of: null
---
Plan: `docs/project/specs/active/plan-2026-09-17-fdu-explicit-core-models.md`, section "Phase 2, Item 2: The Answer Model and Writers", moved here when the item became beads (PR #78 at `e52383d4`; locators verified at `5f2d36d`). The commits are this bead's children, P2.2.1 to P2.2.8; their blockers carry the ordering, so this bead only groups them and closes when they do.

| File | Function or type | Change |
| --- | --- | --- |
| `emit.rs`, `emit/emit_json.rs`, `emit/emit_yaml.rs`, `emit/emit_scalar.rs` (new) | `trait Sink { begin_map(Shape), end_map, begin_seq(Shape), end_seq, key(&'static str), str, u64, i64, bool, null }` with `enum Shape { Block, Inline }`; `JsonSink::{pretty, line}`; `YamlSink`; `is_plain_safe`, `write_json_string`, `write_yaml_scalar` | Add; zero dependencies; generic, not `dyn`, so scalar calls monomorphize |
| `emit/emit_scalar.rs` | replaces `quote` (`report_format.rs:1277-1295`) and `yaml_scalar` (`:1240-1252`) | The `fdu-4xy9` policy: plain only for non-empty ASCII `[A-Za-z0-9._/+-]` that does not start with a digit or sign, is not dot-numeric or `.inf`/`.nan`, and is not a YAML 1.1 boolean or null spelling; otherwise double-quoted, escaping `"`, `\`, controls below 0x20, 0x7F-0x9F, U+2028, U+2029, U+FEFF, U+FFFE, and U+FFFF |
| `report_format.rs` | new `emit_report`, `emit_change`, `emit_cache_status`; `Field { name, presence: Presence }` with `Presence::{Always, Nullable, WhenLossy, WhenAnalyzer(AnalysisSet), WhenSet}` | The only declarations of document structure; the tree walk uses an explicit stack |
| `report_format.rs` | `render` (`:106-113`) | Keep, and add `write(report, format, color, out: &mut dyn io::Write)` for streaming |
| `report_format.rs` | JSON writers (`:466-471`, `:538-976`), YAML writers (`:450`, `:642`, `:880`, `:976-1234`), `indent` (`:1257-1264`), `collapse` (`:1271-1274`), `json_count` (`:1717-1719`) | Delete |
| `report_format.rs` | `render_change` (`:1507-1549`), `render_cache_status` (`:1610-1714`), `report_schema` and schema constants (`:61`, `:63`, `:70`, `:1324-1332`, `:1481`), `render_text_metrics` (`:231-311`), `share_metric_note` (`:319-325`) | Machine formats through the walks, with YAML change records as `---` documents; `fdu.report/7`, `fdu.stream/2`, `fdu.cache/2`; text decides what to show from unit presence and `pages()` |
| `query/query_report.rs` | `Report` (`:779-829`) | Carries `status`, `provenance`, and the request echo; `notes` and `ignored_entries` stay text-only and the schema marks them off the wire |
| `crates/fdu-py/src/lib.rs` | `PyIndex::report` (`:191-249`), `report_dict` through `tree_dict` (`:718-975`); `cache_status_dict` (`:1484-1523`); `Index.since` change dicts (`:500-530`) | Delete the native dict after migrating its test; cache status and change sets read wire keys (`invalidate`, a labelled reason) |
| Documentation | `fdu.report/7` and `fdu.stream/2` in `docs/project/guides/cache-design.md`, `docs/project/architecture/fdu-surface-architecture.md`, `docs/project/architecture/fdu-engine-architecture.md`, `docs/project/release-notes/0.1.0.md`, `docs/project/guides/release-process.md`, `crates/fdu/src/skills/SKILL.md`, and `README.md` | Update |
| `crates/fdu-py/python/fdu/_models.py`, `_api.py` | `MetricValues`, `MetricRow`, `Detection`, `_metric_row`, `_tree`, `report_from_dict` (`:573-1155`); `_cache_status` (`:151-172`), `_change` (`:344-356`) | Optional metric fields, `Pages`, per-unit coverage, paths preferring `path_raw`, an iterative `_tree`; read only wire keys |

**Call sites:** `crates/fdu/src/cli.rs:692`, `:707`, `:729`, `:746-751`, `:812`, `:847`,
`:849`, `:956-978`, `:1095-1106`, `:1159-1166`, and tests `:2514-2517` and `:2796-2799`;
`execution.rs:624-625`; `examples/perf_probe.rs:785`;
`crates/fdu-py/src/opened_binding.rs:935-941` and `opened.py:1089-1098`;
`crates/fdu-py/src/lib.rs:981-992`, `:1102`, `:1260`, `:1418-1449`, `:1467-1481`;
`_api.py:198-207`, `:258-272`, `:464-473`; `_models.py:778-800`; documentation in
`crates/fdu/src/skills/SKILL.md:226-229` and `:272` and `README.md:266`, `:550`, `:561`,
`:604`.

**Tests:**
- `emit` unit tests: the scalar policy over the 121-string corpus from
  [`explorations/yaml-conformance`](../../../../explorations/yaml-conformance/README.md),
  copied to `crates/fdu-core/src/testdata/`; the JSON escape set; sink nesting and
  errors; a `SchemaCheck` adapter checking key order and presence against `Field`
  tables.
- `report_format.rs`: rewrite the YAML quoting, JSON escaping, JSON Lines,
  schema-version, stream-record, and cache-schema tests (`:2902`, `:2915`, `:2429`,
  `:2685`, `:2708`, `:2018`); extend stack-safety and non-Unicode path tests (`:2957`,
  `:3175`, `:3192`, `:2465`, `:2543`) to YAML; add a JSON Lines test with `{ ` in a
  name.
- `scripts/check-yaml.mjs`, reusing the unmerged YAML fixes recorded on `fdu-c2ml`:
  awkward and non-UTF-8 names, `documents`, strict YAML 1.2 (`strict`, `uniqueKeys`,
  `intAsBigInt`) and YAML 1.1 parsing of every document kind deep-equal to exactly
  parsed JSON (the `JSON.parse` reviver’s `context.source`) and to reassembled JSON
  Lines, across views and analyzer sets, cache status, and a watch stream; no raw C1,
  U+2028, or U+FFFE.
- Python: move `crates/fdu-py/tests/smoke.py:324-412` to parsed `render("json")`; add a
  writer-equality test in `test_models.py`. YAML parser parity stays in Node unless a
  reviewed Python YAML dependency is added.
- Goldens: `cli-json`, `cli-content`, `cli-axes`, `cli-lifecycle`, `cli-cache`,
  `cli-surface`, and `cli-watch`; `crates/fdu/tests/cli_color.rs:45` and `:118`.

**Commits:**
1. The emit module and scalar policy with unit tests.
2. The policy inside the existing writers, with strict YAML parsing in `check-yaml.mjs`.
3. JSON Lines through the walk; delete `collapse`.
4. Pretty JSON through the walk with a declared layout; golden diffs are whitespace
   only, proven by comparing parsed values before and after.
5. YAML through the walk, matching JSON’s shape.
6. Change records and cache status through walks.
7. Text over the model.
8. Python models, the `smoke.py` migration, and deleting the native dict.

**Risks:** byte-identical pretty JSON would need layout hints for today’s accidents, so
the plan accepts whitespace-only golden diffs verified by parsed equality; the sink
removes today’s repeated copying (`indent` and `collapse`), but `render` still
materializes a string for Python, and YAML indentation still grows with tree depth, so
record JSON, JSON Lines, and YAML render modes in `perf_probe` with `make perf-record`;
`Index.since` change sets are folded into the change-record model rather than kept as a
fourth shape.

## Original scope (before the implementation detail)

fdu's JSON and YAML are hand-written field by field in report_format.rs, so the two formats diverged
under one schema id and YAML scalar quoting failed real parsers (fdu-c2ml). Python already wraps the
Rust renderer, which is the right boundary. Goal (maintainer direction 2026-09-17): rely on highly
conformant YAML in Rust, reusing a standard approach or, if none is adequate, a reusable YAML utility
used consistently by every YAML site and usable in future projects; a single ordered value model so JSON
and YAML cannot diverge structurally; conformance proven against real YAML 1.1 and 1.2 parsers.

## Notes

2026-09-17 (PR #78 review): Answer model includes tree status and provenance as separate parts; four formats plus Python models; the unused native dict (fdu-py lib.rs:719-803) is deleted rather than migrated.

2026-09-22 typed-answer fixture followup staged, not yet published: CLI guide/golden now names report/7 and stream/2, the document-page test requests words before checking words_per_page, and the one native JSON file path uses the existing [JSON_SEP] pattern on Windows. The lower parity guide line is synchronized from reviewed Linux evidence; the final composed parity artifact remains byte-identical to its actual Linux recording. No production behavior changed after the published ff2b07da head; current-head CI remains pending.
