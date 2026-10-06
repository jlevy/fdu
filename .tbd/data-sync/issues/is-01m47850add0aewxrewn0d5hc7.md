---
type: is
id: is-01m47850add0aewxrewn0d5hc7
title: Goldens, parity artifact, and path-independence matrix for view-implied analysis
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-10-05-fdu-view-analyze-redesign.md
delegate: claude-code@spud10.local
labels: []
dependencies: []
parent_id: is-01m47831pcsbtvygctgzqfvzc8
hold: null
hold_until: null
created_at: 2026-10-05T23:59:56.236Z
updated_at: 2026-10-06T01:52:26.888Z
started_at: 2026-10-06T00:49:31.537Z
closed_at: 2026-10-06T01:52:26.887Z
close_reason: Goldens (94938ed3, 3710882f merge), parity classes/shim (0b1b9be7), parity re-record (616816b3, macOS; CI Linux authoritative), PI implied phase + v_code/v_documents/v_code_documents, QA sheet, perf probe (e5f5b8f2); make check exit 0
resolution: null
duplicate_of: null
---
tests/golden/cli-content.tryscript.md: the '--view documents content-project' session turns from an exit-2 refusal into a report; add a '--view code,documents' session beside the '--analyze code,words' one with identical stdout; the not-displayed sessions (--analyze lines --view summary, --analyze all --view tree, stale-ok variant) and the --view full session carry the new note/tip lines; add the converse of 'The Analyzer Set Chooses the View' (the view chooses the analyzer set; languages/full alone read nothing). tests/golden/cli-surface.tryscript.md follows help/docs/skill text from the docs bead. Read every diff; never accept tryscript --update output with expanded named patterns (make check's portability gate). Re-record tests/parity/deviations-python.diff via make parity-update and confirm only sameAnalysisTip-class lines moved. tests/path_independence/matrix.py: add v_code, v_documents, v_code_documents specs built from views alone and assert equality with a_code, a_words, a_all on content, tree status, and sidecar identity across cold, warm, and stale_ok histories. tests/qa/cli-installed-e2e.qa.md: the documents-no-analyze row expects a report. Rust CLI tests in crates/fdu/tests/cli_exit.rs that assert the documents refusal, if any.
