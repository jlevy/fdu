---
type: is
id: is-01m2nh3kky1kvjwrpwa8mtdv3n
title: "The --view parser accepts docs for documents, but no help, --docs, or skill text names it: document the alias or drop it"
kind: task
status: open
priority: 3
version: 1
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-16T16:34:26.045Z
updated_at: 2026-09-16T16:34:26.045Z
---
At 16efcd0, crates/fdu-core/src/query/query_report.rs:151 parses both "documents" and "docs" as ViewSpec::Documents, for the CLI and the Python binding alike. ViewSpec::vocabulary() lists only documents, so --help, the --docs View row (crates/fdu/src/cli.rs, pinned whole against the vocabulary by the_guide_only_names_views_and_analyzers_that_parse), and crates/fdu/src/skills/SKILL.md never name docs. Found by the doc-drift audit (Documents that should exist, item 3). Needs a decision, not a text fix: either the alias is part of the contract (then the vocabulary, the guide test, --help, and the skill name it) or it is not (then the parser drops it, which is a behaviour change that rejects an input 0.1.0 would otherwise ship accepting). Left unchanged by the doc-drift package 5 PR, which changes text only.
