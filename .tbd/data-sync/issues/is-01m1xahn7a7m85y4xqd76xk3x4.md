---
type: is
id: is-01m1xahn7a7m85y4xqd76xk3x4
title: Classify bulk syscalls and fdu-core symbols in profiles
kind: task
status: in_progress
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
labels: []
dependencies: []
parent_id: is-01m1dtr903vj783j9ajaxfnczf
created_at: 2026-09-07T06:58:00.040Z
updated_at: 2026-09-07T07:19:52.169Z
---
The final five-job profile at 1a39be9 reports 34.52% cold self samples in getattrlistbulk but classifies it as other. Layer regexes also still prefer the pre-split 3fdu symbols over 8fdu_core. Raw symbols are preserved and usable; update only presentation classification with tests for current and legacy symbols, and retain allocation/path/oracle precedence. Do not alter timing metrics or reinterpret raw samples as new measurements. Publish accurate current attribution with the final evidence.

## Notes

The new profile tests first failed for the bulk syscall and all current-crate module symbols. Current and legacy mangled/demangled modules now classify correctly, allocator/path/oracle precedence is preserved, and all 227 real-tree harness tests pass. Raw samples and timing metrics are unchanged. Full isolated gate is running.
