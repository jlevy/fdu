---
type: is
id: is-01m1xahn7a7m85y4xqd76xk3x4
title: Classify bulk syscalls and fdu-core symbols in profiles
kind: task
status: closed
priority: 2
version: 5
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m1dtr903vj783j9ajaxfnczf
created_at: 2026-09-07T06:58:00.040Z
updated_at: 2026-09-07T07:32:08.689Z
closed_at: 2026-09-07T07:32:08.689Z
close_reason: Red-green regression tests, full isolated gate, cross-platform lint, and all 19 CI checks passed on the pushed formal stack.
resolution: null
duplicate_of: null
---
The final five-job profile at 1a39be9 reports 34.52% cold self samples in getattrlistbulk but classifies it as other. Layer regexes also still prefer the pre-split 3fdu symbols over 8fdu_core. Raw symbols are preserved and usable; update only presentation classification with tests for current and legacy symbols, and retain allocation/path/oracle precedence. Do not alter timing metrics or reinterpret raw samples as new measurements. Publish accurate current attribution with the final evidence.

## Notes

Completed at64c6e61: red tests exposed all current-crate module labels and getattrlistbulk in other. Current/legacy mangled/demangled layers now classify correctly with allocator/path/oracle precedence unchanged. 227 real-tree tests, full isolated gate, cross-lint and all19 CI checks passed. Raw samples and timing metrics are unchanged.
