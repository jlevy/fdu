---
type: is
id: is-01kzkssg8cxnkr368qyb6dfpjc
title: Make documented builds avoid unsupported direct PyO3 linking
kind: bug
status: closed
priority: 1
version: 5
spec_path: docs/project/specs/done/plan-2026-08-09-fdu-cli-golden-tests.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzksn3gepmk01a21gkxxs6bv
parent_id: is-01kzkskszrb20xkk7g3gt32za6
created_at: 2026-08-09T17:40:37.771Z
updated_at: 2026-08-09T18:39:22.716Z
closed_at: 2026-08-09T17:44:51.420Z
close_reason: Scoped make build and release to the supported fdu core crate and CLI; make build, make release, and the complete make check gate now pass on macOS while Python continues through maturin in python-smoke.
---
The documented cargo workspace build behind make build/release attempts to link the fdu-py cdylib directly and fails on macOS with unresolved Python symbols; the supported Python artifact path is maturin. Scope core build/release targets to fdu with all features, retain python-smoke as the binding build gate, and verify Linux/macOS behavior.
