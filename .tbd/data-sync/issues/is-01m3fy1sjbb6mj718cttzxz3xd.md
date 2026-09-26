---
type: is
id: is-01m3fy1sjbb6mj718cttzxz3xd
title: Simplify uv setup and present agent-first usage
kind: task
status: open
priority: 1
version: 1
labels:
  - release
dependencies: []
created_at: 2026-09-26T22:40:53.311Z
updated_at: 2026-09-26T22:40:53.311Z
---
Refine PR #129 after published-wheel verification: remove the normal-use Python version override from uv commands, keep a free-threaded interpreter troubleshooting note, make the generated skill fallback wheel-only, and present agent skill, CLI, Python, and Rust setup concisely in that order across the README and self-documenting CLI.
