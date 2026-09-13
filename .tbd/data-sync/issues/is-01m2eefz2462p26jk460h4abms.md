---
type: is
id: is-01m2eefz2462p26jk460h4abms
title: "PR #52 review PERF-3: opened-discovery probe silently ignores --threads, --no-controls, --max-depth, --order"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m2eeeykapyy2pg6zjj9qxcbt
created_at: 2026-09-13T22:34:04.227Z
updated_at: 2026-09-13T22:55:56.485Z
closed_at: 2026-09-13T22:55:52.929Z
close_reason: "Fixed in the PR #52 probe commit (see notes): opened-discovery refuses --threads, --no-controls, --max-depth, and --order, since OpenOptions fixes one breadth-first producer with controls always observed and has no thread setting to map onto; test_probe.py passes --threads 1 only to scan-index; new probe unit test."
resolution: null
duplicate_of: null
---
PR #52 review PERF-3 (Low). crates/fdu-core/examples/perf_probe.rs:914-926 at afbb2ee. Arguments::parse accepts --threads, --no-controls, --max-depth, and --order for every mode, but opened_discovery builds OpenOptions without them. The harness test passes --threads 1 to opened-discovery and believes it pinned the worker count; it holds only because the fixture is one flat directory. Fix: reject unsupported flags for that mode, or map --threads into the discovery budget.

## Notes

Fixed in 888791b on PR #52.
