---
type: is
id: is-01m0k512k9a6dq2k51fbfe5xn4
title: "YAML output contract: metric-row shape, forbidden characters, number-like names, raw paths"
kind: bug
status: open
priority: 0
version: 6
spec_path: docs/project/specs/active/plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md
labels:
  - release
dependencies: []
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-08-21T21:53:27.657Z
updated_at: 2026-09-17T02:11:50.993Z
---
A byte-stable golden proves the output has not *changed*. It does not prove the output is
*valid*: a consistently malformed document passes forever, and the serializers here are
hand-written -- the project deliberately avoids serde -- so nothing else would catch it.

Audited 2026-08-21:

  json    parsed, by JSON.parse in scripts/content-selfcheck.mjs
  jsonl   never parsed. The watermark integration test greps the output with
          `listing.contains(...)`, which is a substring check
  yaml    never parsed anywhere, by anything

YAML is the sharp end. Its serializer is hand-written, its quoting rules are the fiddly
part of the format, and no YAML parser has ever read fdu's output.

Fix by consuming each format in a golden, which is what a session test is for -- run the
command, pipe it into a parser, print a field. That demonstrates the format working rather
than merely holding still, and the demonstration is the artifact a reader learns from.

  json   node -e "JSON.parse(...)"           -- already the pattern here
  jsonl  parse each line independently, which is the format's entire contract
  yaml   needs a parser: `yaml` as a pinned devDependency is the portable option, since
         CI installs neither jq nor yq and node ships no YAML support

The dependency question belongs to SUPPLY-CHAIN-SECURITY.md and the 14-day cool-off.

## Notes

CORRECTION: jsonl IS checked line by line (report_format tests, 'line is not a JSON document'). Two of three claims stand, one did not. Accurate picture: json is really parsed (JSON.parse in content-selfcheck.mjs); jsonl is only brace-balance checked by the hand-written is_valid_json, which would accept {"a": } -- structurally balanced, not valid; yaml has no check of any kind. The yaml gap is the real one and is unchanged.

2026-09-16 (doc-drift audit package 4, verified at 16efcd0): left open, narrowed to JSONL. PR #39 (a6b670c) closed the YAML half: scripts/check-yaml.mjs parses every view's YAML with the locked `yaml` package and runs as `yaml-selfcheck` under `make test`, so under `make check`. JSON is parsed by scripts/content-selfcheck.mjs and by the Python tests. One-shot JSONL report output is still read by no JSON parser: the only check is the brace-balancing is_valid_json in crates/fdu-core/src/report_format.rs (jsonl_emits_one_document_per_line), which accepts {"a": }. The Python public smoke json.loads a single watch change rendered as JSONL, not a report. To close: parse each JSONL report line with a real parser, in a golden or self-check as the plan describes. The epic fdu-yov0 stays open with this bead; every other child is closed.

2026-09-17: Re-scoped as the 0.1.0 YAML contract blocker and moved under fdu-gjc2. Defects confirmed on the
release candidate: (1) YAML metric rows flatten share/metrics/pages while JSON nests them under the same
fdu.report/6; (2) U+007F-U+009F and U+FFFE/U+FFFF emitted raw, so PyYAML and ruamel reject the document
and NEL folds to a space; (3) names like 0x10, 1_000, 0b101, .inf, .NaN emitted plain and read back as
numbers; (4) root_raw/path_raw missing from YAML. Fixes and a YAML-equals-JSON check in
scripts/check-yaml.mjs are on claude/release-e2e-fixes; held while the conformant-output approach is evaluated
(standard Rust YAML emitter or a reusable utility; see the conformant machine output epic). JSONL parses
cleanly in every view and analysis combination.

2026-09-17 additions before 0.1.0 from the parser-matrix evaluation (fdu-4xy9): quote dot-numeric plain
scalars matching (?i)^\.[0-9_.]*([eE][-+]?[0-9_]*)?$ (covers `._`, `._1`, `.1_0`; `.` itself is a
judgment call because it changes every tree-root golden line); escape U+2028, U+2029 and U+FEFF in
`quote` (valid JSON and YAML); run scripts/check-yaml.mjs with strict: true in both version 1.1 and 1.2
over the corpus; unit-test the escape set in Rust because npm yaml accepts C1 and line separators.
