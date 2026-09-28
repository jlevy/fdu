---
type: is
id: is-01m3ke9ggpmbn73frv8wtgb9w0
title: Align the fdu tagline across --guide, the fdu-core crate docs, and the READMEs
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-09-28T07:22:26.709Z
updated_at: 2026-09-28T07:22:26.709Z
---
REG-5 from the stack 141 regression review. --help, README, crates/fdu-py/README.md and pyproject.toml say 'Fastest native du replacement...', while fdu --guide (crates/fdu/src/cli.rs:137), the fdu-core crate doc (lib.rs:1) and its Cargo description say 'a fast, incremental file roll-up engine'. The README's own Linux table shows the default indexed command 21-23% slower than pdu/diskus, so 'fastest' holds for macOS and Linux summary mode. User's call on wording; align the surfaces either way.
