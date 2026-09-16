---
type: is
id: is-01m2nh3kky1kvjwrpwa8mtdv3n
title: Drop the undocumented --view docs alias for documents, which the no-aliases policy says should not exist
kind: task
status: closed
priority: 3
version: 4
delegate: codex@spud10
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-16T16:34:26.045Z
updated_at: 2026-09-16T18:34:26.739Z
closed_at: 2026-09-16T18:34:26.738Z
close_reason: "Merged PR #72 at 4a4965bdaf39f6a5369b5e0db2a5ff1fbf1011d0; removed the unreleased --view docs and --cache readonly aliases across core, CLI, and Python, with all 19 CI jobs passing."
resolution: null
duplicate_of: null
---
At 16efcd0, crates/fdu-core/src/query/query_report.rs:151 parses both "documents" and "docs" as ViewSpec::Documents, for the CLI and the Python binding alike. ViewSpec::vocabulary() lists only documents, so --help, the --docs View row (crates/fdu/src/cli.rs, pinned whole against the vocabulary by the_guide_only_names_views_and_analyzers_that_parse), and crates/fdu/src/skills/SKILL.md never name docs. Found by the doc-drift audit (Documents that should exist, item 3). Needs a decision, not a text fix: either the alias is part of the contract (then the vocabulary, the guide test, --help, and the skill name it) or it is not (then the parser drops it, which is a behaviour change that rejects an input 0.1.0 would otherwise ship accepting). Left unchanged by the doc-drift package 5 PR, which changes text only.

## Notes

FINDING (PR #70, 2026-09-16): the alias is unintended, not merely undocumented, so PR #70 does not document it.

Evidence at 16efcd0:
- It is the only view alias. ViewSpec::parse (crates/fdu-core/src/query/query_report.rs:151) accepts documents|docs; every other view has one spelling. Case-insensitivity is separate and not at issue.
- It arrived in 60fcbc6 (feat: expose grouped content reports) as a shorthand beside --analyze documents|docs, when documents was also an analysis profile. The analyze alias was dropped when #37 renamed documents to words, under the composable-CLI plan's stated rule that the interface is pre-release and no aliases are retained (docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md:276, :763, :866). The view alias survived that sweep.
- Nothing names or tests it: ViewSpec::vocabulary() and the --view error omit it; --help, --docs, and SKILL.md omit it; no test parses docs; the Python View StrEnum (crates/fdu-py/python/fdu/_models.py:67) has no docs member, so only the CLI and the raw native binding accept it.
- the_guide_only_names_views_and_analyzers_that_parse (crates/fdu/src/cli.rs) asserts the --docs View row lists exactly what --view accepts, which the alias makes false.

Recommended fix (behaviour change, so needs its own PR): remove the docs arm from ViewSpec::parse, add a unit test that docs is rejected with the vocabulary message, and check goldens and the parity artifact for any use of --view docs (none found by grep).

Analogous, not a view: --cache readonly is accepted beside read-only in crates/fdu/src/cli.rs:1127 and crates/fdu-py/src/lib.rs:610, and appears in no help text, error message, or cache_policies contract list (crates/fdu-py/src/lib.rs:1754). Decide it in the same change.
