---
type: is
id: is-01m3stdvsbm5f4adzh7cxptp25
title: Run make release-stability --only gates,candidate for real before 0.3.1
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-09-30T18:49:58.827Z
updated_at: 2026-09-30T18:49:58.827Z
---
The stability pass's gates and its wheel build (scripts/release/stability_pass.py, PR #170) have run only against the test suite's fake host. The QA and correctness stages ran for real against the 0.3.0 wheel. Before the 0.3.1 stability pass relies on the command, run it for real on a release candidate with --only gates,candidate, then check the following, and fix anything that differs:
- each gate's log and exit status;
- the target-owner step before the maturin build;
- the cross-lint target check;
- the created-tree bookkeeping in state.json;
- cleanup.
