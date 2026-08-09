---
type: is
id: is-01kzky8gxazfdstfgbv3m9fa58
title: Preserve filesystem-native identity through classification and Python
kind: bug
status: open
priority: 2
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-rust-engineering-quality.md
labels:
  - filesystem
  - python
  - api
dependencies:
  - type: blocks
    target: is-01kzg4bf862ajh8g2tmv5bznng
  - type: blocks
    target: is-01kzg4bfj2cqzcksgpmfce89w6
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
parent_id: is-01kzky6vqxwd47xz3we21s86zq
created_at: 2026-08-09T18:58:44.265Z
updated_at: 2026-08-09T18:59:07.078Z
---
Index contribution currently calls OsStr::to_str before extension classification, so a non-Unicode name with an ASCII extension disappears from by-extension totals. The Python boundary accepts string-only paths and renders child and delta paths lossily. Make classification operate on native path units and prove ASCII extension behavior for Unix bytes and Windows wide strings without lossy conversion. Accept normal Python os.PathLike inputs losslessly, expose reversible identity for root, children, and since records, and map I/O errors to useful OSError fields including the path and OS cause. Expand installed-wheel tests to the declared minimum and current Python versions, cover representative failures, and clean up only the exact temporary roots they create. Keep the binding bulk-oriented and independent of the watch feature.
