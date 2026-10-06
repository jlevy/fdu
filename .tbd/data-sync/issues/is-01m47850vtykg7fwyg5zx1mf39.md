---
type: is
id: is-01m47850vtykg7fwyg5zx1mf39
title: "Docs: help, --docs, skill, usage guide, README, machine-output, output design, surface architecture, design principles"
kind: task
status: in_progress
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-10-05-fdu-view-analyze-redesign.md
delegate: claude-code@spud10.local
labels: []
dependencies:
  - type: blocks
    target: is-01m47850add0aewxrewn0d5hc7
parent_id: is-01m47831pcsbtvygctgzqfvzc8
hold: null
hold_until: null
created_at: 2026-10-05T23:59:56.794Z
updated_at: 2026-10-06T00:45:03.897Z
started_at: 2026-10-06T00:45:03.883Z
---
crates/fdu/src/cli.rs: --analyze help 'Analyzers to run beyond what the views imply: none, lines, code, words, or all'; --view help names code and documents as views that read file contents; DOCS_POINTER and the --docs text add fdu . --view=code, --view=documents, --view=code,documents and keep --analyze lines --view languages and --analyze code --view languages as control examples; rewrite the VIEWS AND ANALYSIS paragraph around the rule. crates/fdu/src/skills/SKILL.md: run list, axes table (Content: 'which file bodies are read beyond what the views imply'), 'Pick the View' bullets, the 'Requesting analysis without naming a view' paragraph; the_skill_only_names_views_and_analyzers_that_parse must still pass. docs/usage.md: start-here commands, 'Analyze File Contents' and 'Measurements, Views, and Headers' rewritten around the rule, keeping the mapping table. docs/machine-output.md: request.analyze is the enabled set, named or implied. README.md: quick start uses fdu . --view=code,documents; question table adds 'Words in documents'. docs/project/architecture/fdu-design-principles.md: restate 'Cost flows one way; display follows cost' (a view with a metadata meaning never enables an analyzer; a view with none is a request for its analysis, enabled when a fresh basis is built; a held basis is never widened), with the reason beside the clause; six-axes content row. fdu-output-design.md 'Content Reports and Names': each content view requests its analyzer. fdu-surface-architecture.md: deviation class text and the report-vs-open asymmetry. Run make docs-format. cli.rs is also being edited under fdu-cmr4/fdu-wzpx; coordinate.
