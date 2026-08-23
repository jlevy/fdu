---
type: is
id: is-01m0ptvnmg9p0s1qh9174hpcv3
title: Expose ScanOrder on the CLI and Python surfaces
kind: bug
status: open
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0qs0msk75k8r89b44vqqjnz
  - type: blocks
    target: is-01m0qs19pg77zfmd3s2kg7k905
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T08:12:42.512Z
updated_at: 2026-08-23T17:00:41.381Z
---
ScanOrder is a public engine type and a ScanConfig field (breadth-first default, region-scheduled, measured faster than depth-first on the large heterogeneous tree in exp-037), but there is no --order flag and no ScanOptions field: ScanOptions carries max_depth and one_filesystem and nothing else. A Rust caller can choose the traversal order; a CLI or Python caller cannot. That is the mirror image of 'the command line invents nothing' and the same defect — a capability reachable from one surface and not the others is unfinished. The progressive-results plan recorded '--order on the probe' as done; the probe is not a public surface. Add the scope-axis flag and the ScanOptions field, with goldens and parity rows. Note the ordering only becomes observable to a consumer once the session lands (fdu-4o0m): until then fdu pays breadth-first's cost and collects none of its benefit.

## Notes

WIDENED: covers threads as well as order. ScanConfig has BOTH knobs already — threads: Option<usize> beside order: ScanOrder at scan.rs:135 — and cli.rs:537 constructs ScanConfig with ..ScanConfig::default(), discarding both. FILES: cli.rs (Cli struct :313, ScanConfig construction :537), fdu-py/python/fdu/_models.py (ScanOptions), fdu-py/src/lib.rs (open/scan signatures), tests/golden/. Two #[arg(..., help_heading = "SCOPE")] fields joining scan_depth and one_filesystem; parse_order beside parse_sort (cli.rs:1323); plain usize for threads. Goldens: one session per order over a fixture with several top-level subtrees. The surface-vocabulary class in parity-classes.mjs already covers --scan-depth against max_depth, so --threads/threads needs no new class. THIS BEAD BLOCKS EVERY PROGRESSIVE GOLDEN: a scan trace cannot be sequenced causally the way a watch stream can (the tree is static; emission order is decided by worker scheduling), so its determinism must be injected by fixing worker count and order. Without this there is nothing reproducible to record.
