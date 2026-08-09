---
type: is
id: is-01kzkssg8cxnkr368qyb6dfpjc
title: Make documented builds avoid unsupported direct PyO3 linking
kind: bug
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-golden-tests.md
labels: []
dependencies:
  - type: blocks
    target: is-01kzksn3gepmk01a21gkxxs6bv
parent_id: is-01kzkskszrb20xkk7g3gt32za6
created_at: 2026-08-09T17:40:37.771Z
updated_at: 2026-08-09T17:40:46.969Z
---
The documented cargo workspace build behind make build/release attempts to link the fdu-py cdylib directly and fails on macOS with unresolved Python symbols; the supported Python artifact path is maturin. Scope core build/release targets to fdu with all features, retain python-smoke as the binding build gate, and verify Linux/macOS behavior.
