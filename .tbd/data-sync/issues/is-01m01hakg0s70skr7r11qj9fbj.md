---
type: is
id: is-01m01hakg0s70skr7r11qj9fbj
title: Make content self-check enumerate every tracked file type
kind: bug
status: in_progress
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-08-15T01:42:03.007Z
updated_at: 2026-08-15T01:42:06.901Z
---
The self-check asserts that specific tracked types exist but relies on the CLI's default top-10 report limit. Large report assets can push a valid type such as TOML below that display limit. Request all rows before asserting repository-wide type coverage.
