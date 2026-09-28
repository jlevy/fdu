---
type: is
id: is-01m3ke9gwm906z28mvhbdapjxw
title: Add a golden for the snapshot save-warning path
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-09-28T07:22:27.091Z
updated_at: 2026-09-28T07:22:27.091Z
---
REG-7 from the stack 141 regression review. The promise that a read-only cache directory still answers under --cache on/analysis (warn: I/O error at .../.<key>.metadata.bin.tmp...: Permission denied) is covered only by a fixed-string unit test (cli.rs:3177-3192). Add a tryscript golden using [SCAN_PATH]/[OS_ERROR]-style patterns.
