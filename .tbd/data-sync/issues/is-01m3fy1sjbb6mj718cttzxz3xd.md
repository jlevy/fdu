---
type: is
id: is-01m3fy1sjbb6mj718cttzxz3xd
title: Simplify uv setup and present agent-first usage
kind: task
status: open
priority: 1
version: 3
labels:
  - release
dependencies: []
created_at: 2026-09-26T22:40:53.311Z
updated_at: 2026-09-26T23:43:41.055Z
closed_at: 2026-09-26T23:19:55.048Z
close_reason: Removed normal-use Python pins, verified unpinned uvx and uv tool installs against PyPI with builds disabled, added agent-first README and CLI setup, made the installed skill wheel-only, and passed the full handoff gate including parity and path-independence follow-through.
resolution: null
duplicate_of: null
---
Refine PR #129 after published-wheel verification: remove the normal-use Python version override from uv commands, keep a free-threaded interpreter troubleshooting note, make the generated skill fallback wheel-only, and present agent skill, CLI, Python, and Rust setup concisely in that order across the README and self-documenting CLI.

## Notes

User review found that the one-line agent prompt still depended on current-session memory. Revising it to describe durable self-contained skill installation and making the generated skill explicitly context-independent, while reducing explanatory prose.
