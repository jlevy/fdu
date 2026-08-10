---
type: is
id: is-01kzmnxh8812bvnjj5v790jtdy
title: Add a portable version-pinned fdu agent skill
kind: feature
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-ux-and-agent-skill.md
labels:
  - cli
  - skill
dependencies:
  - type: blocks
    target: is-01kzmnxy0xvkvazmqvdwsjm20h
parent_id: is-01kzmnx3taexx4cq4m722p0yp0
created_at: 2026-08-10T01:52:09.988Z
updated_at: 2026-08-10T02:09:06.194Z
closed_at: 2026-08-10T02:09:06.194Z
close_reason: Implemented fdu --skill as a self-contained version-pinned local-first Agent Skill. Full rendered output is golden-tested, formatting is Flowmark-clean, and machine output remains ANSI-free.
---
Add fdu --skill as a complete one-file Agent Skills document. Keep it concise and link-free, route details to --help, require agents to inspect exit/completeness/truncation/scope fields, try a local fdu first, and show only an exact-version uvx fallback. Do not add an installer, managed AGENTS.md block, relative resources, or wildcard uvx pre-approval. Golden-test the full rendered skill.
