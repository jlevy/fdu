---
type: is
id: is-01m0ptvnmg9p0s1qh9174hpcv3
title: Expose ScanOrder on the CLI and Python surfaces
kind: bug
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies: []
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T08:12:42.512Z
updated_at: 2026-08-23T08:12:42.512Z
---
ScanOrder is a public engine type and a ScanConfig field (breadth-first default, region-scheduled, measured faster than depth-first on the large heterogeneous tree in exp-037), but there is no --order flag and no ScanOptions field: ScanOptions carries max_depth and one_filesystem and nothing else. A Rust caller can choose the traversal order; a CLI or Python caller cannot. That is the mirror image of 'the command line invents nothing' and the same defect — a capability reachable from one surface and not the others is unfinished. The progressive-results plan recorded '--order on the probe' as done; the probe is not a public surface. Add the scope-axis flag and the ScanOptions field, with goldens and parity rows. Note the ordering only becomes observable to a consumer once the session lands (fdu-4o0m): until then fdu pays breadth-first's cost and collects none of its benefit.
