---
type: is
id: is-01m045mb3wndcgrz7gxyw202sy
title: Complete the performance record and generate the technical report
kind: epic
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-15-fdu-performance-record-and-report.md
labels:
  - performance
dependencies: []
created_at: 2026-08-16T02:15:22.490Z
updated_at: 2026-08-16T02:15:22.490Z
---
Epic for the four-phase plan in the spec: (A) surface absolute timings already in all
64 artifacts through the ledger generator and add the duplicate-id check (fdu-f8ni);
(B) land the default-CLI harness job (fdu-ao6p) and promote this session's bead-held
findings into artifacts; (C) fill the cross-platform matrix by re-running the accepted
chain on the missing platform against fingerprint-matched subjects, plus the quiet-host
peer cell (fdu-ow8y); (D) emit the generated technical report -- every improvement,
accepted and rejected, absolute walls per platform per subject -- wired into
perf-ledger so it cannot drift.

Audit result the plan rests on: absolutes are recorded (wall_ns control/candidate
medians in every artifact) but unsurfaced; cross-platform coverage is the real gap
(nearly every experiment measured in exactly one regime).
