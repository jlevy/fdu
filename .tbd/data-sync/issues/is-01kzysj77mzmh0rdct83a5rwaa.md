---
type: is
id: is-01kzysj77mzmh0rdct83a5rwaa
title: Use canonical language names and align metric text rows
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01kzynmdn70evmzwx3bjcexzkb
created_at: 2026-08-14T00:08:17.907Z
updated_at: 2026-08-14T00:24:50.933Z
closed_at: 2026-08-14T00:24:50.927Z
close_reason: Canonical names now cover every code rule; language text columns align in plain and ANSI output; machine IDs remain stable; README/help/goldens updated; make check and 97 CLI goldens pass.
---
Render canonical human-facing language names such as CSS, Go, JavaScript, C++, C#, PowerShell, and Protocol Buffers in text output while preserving stable lowercase machine IDs. Align metric suffix columns by visible label width, including when ANSI color is enabled, cover all known code rules, update goldens, reinstall globally, and validate PR CI.
