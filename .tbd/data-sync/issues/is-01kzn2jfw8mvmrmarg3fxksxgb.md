---
type: is
id: is-01kzn2jfw8mvmrmarg3fxksxgb
title: Use fresh fingerprint metadata on Windows
kind: bug
status: closed
priority: 0
version: 3
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - windows
dependencies: []
parent_id: is-01kzkzm62q1vwxbv9hbp39bxxm
created_at: 2026-08-10T05:33:19.622Z
updated_at: 2026-08-10T06:06:28.056Z
closed_at: 2026-08-10T06:06:28.055Z
close_reason: All local handoff gates and the complete Linux, macOS, Windows, and installed-wheel CI matrix pass with dedicated regressions and documented evidence.
---
Windows CI proves that std::fs::DirEntry::metadata can return cached enumeration attributes: unchanged revalidation reports one spurious update in both the Rust suite and installed-wheel smoke test. Keep the measured DirEntry path on Unix, use a fresh non-following path stat on non-Unix platforms, add a mutation-after-enumeration regression, correct the performance evidence's platform claims, and verify all CI surfaces.
