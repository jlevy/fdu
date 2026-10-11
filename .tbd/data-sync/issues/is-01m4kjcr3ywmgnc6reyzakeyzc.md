---
type: is
id: is-01m4kjcr3ywmgnc6reyzakeyzc
title: Make the list view fast over a million-entry tree
kind: task
status: open
priority: 2
version: 7
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: null
labels: []
dependencies: []
parent_id: is-01m4fxa1r9zx7yxe0a8phqm82x
hold: null
hold_until: null
created_at: 2026-10-10T18:49:48.925Z
updated_at: 2026-10-11T22:05:57.957Z
started_at: 2026-10-11T05:29:24.511Z
---
fdu-fy0l's committed measurement (2026-10-10, Linux, evidence/exp-217/python-modified-at.json) labels the candidate 100,000-row fdu.report list over linux-balanced-1m at about 10.6 s, against about 1.2 s for the command line's default tree in exp-217's tool cells. Model conversion is about 1.3 s, suggesting roughly 9.3 s remains in native work; that subtraction is not a separately measured native-call time. Both original wheels report version 0.4.0 and have no recorded commit/hash attestation (fdu-m3z1), so do not treat this as an attested cross-release comparison.

The previously quoted 10.0 s CLI list and 11.4 s native-call timings have no committed evidence and are withdrawn. Re-time the CLI list and native call with build identities and commit the results before profiling. Candidates for profiling are the per-directory subtree measure for list rows, sorting a million rows, and building rows that the output bound discards.

## Notes

Review A6 on PR #194: the 10.0 s command-line and 11.4 s native-call timings above have no committed evidence and were withdrawn from exp-217. What is committed (evidence/exp-217/python-modified-at.json): fdu.report for the 100,000-row list on linux-balanced-1m takes 10.6 s at 0.5.0 (paired median of 20), of which the JSON-to-model conversion is about 1.3 s, leaving about 9.3 s in the native call, against 1.2 s for the 0.5.0 command line's default tree over the same tree (exp-217 tool cells). Re-time the command line and the native call, and commit them, before profiling.
