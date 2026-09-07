---
type: is
id: is-01m1dtr903vj783j9ajaxfnczf
title: Prove one-shot parity and add deterministic regression guards
kind: task
status: in_progress
priority: 0
version: 22
spec_path: docs/project/specs/active/plan-2026-08-31-fdu-streaming-performance-parity.md
delegate: codex@spud10.local
labels:
  - performance
  - validation
dependencies:
  - type: blocks
    target: is-01m1x444q4jz0680n8a057r5z8
parent_id: is-01m1dtq2kd9dex87vs7mzajejc
child_order_hints:
  - is-01m1edc4xady6k86e0hsbzfsk1
  - is-01m1eek06tcb89yygyc1xz2yz5
  - is-01m1egf3aa4wt4kc2z5qmhspqp
  - is-01m1egxbrdj757jr3bk8bhv1ce
  - is-01m1ejqfv4khft8mbkfw7f3q0f
  - is-01m1ekg6ewkj2mr9wf1xs9g01y
  - is-01m1xahn7a7m85y4xqd76xk3x4
hold: blocked
hold_until: null
created_at: 2026-09-01T06:33:23.201Z
updated_at: 2026-09-07T08:48:14.280Z
started_at: 2026-09-01T11:13:09.191Z
---
Re-profile after every accepted experiment, close only profile-named residual costs, meet the plan wall/component/allocation thresholds on control-free and control-rich real trees, add negative-tested per-entry allocation and detached zero-work guards, run the full and cross-platform gates, and record every experiment.

## Notes

Final engine ad52469 and evidence head afbb2ee are pushed through formal stack #53; full isolated make check, candidate cross-lint and all 19 CI checks pass (run 34101768829). Manifest 9957822e6bee5900e8bafcc8de4f648e3b08e7c0e8ef2ad49c7b459afa879f30 binds each immutable binary to its own source checkout, including ad52469 separately from the documentation-only harness head. Repeated final scoped checks pass both nominated subjects: default alloc/byte ratios versus historical are 0.6463/0.9474 Rust and 0.6673/0.9147 source; cold and opened allocation/reallocation/byte ratios also remain below 1.05, with detached ancestry/effect/impact/journal counters zero. Compared semantic summaries are exact after explicitly classifying historical-only serialization differences: v3 adds an 8-byte type-rules fingerprint plus 4-byte empty control-table count and the newer probe adds a null commit diagnostic. Neither differs between the current structural control and candidate. These are instrumented allocation/oracle checks, not timing proof. The first final-ad52469 historical-Rust attempt refused before trials at 26.9% CPU busy; a later snapshot with our builds and checks finished was 39.86%. No final timing samples exist. Do not interrupt unrelated work or relax the quiet threshold. Hold for an idle host, then run the original fixed 12 pairs/3 warmups across both subjects and mutation confirmation, preserving all original historical, structural and opened thresholds. Local raw results and drivers remain intact.
