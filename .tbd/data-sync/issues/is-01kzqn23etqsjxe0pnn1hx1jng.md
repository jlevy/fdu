---
type: is
id: is-01kzqn23etqsjxe0pnn1hx1jng
title: "P1: Python Index.report() mirroring the query API"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-08-10-fdu-composable-cli-surface.md
labels: []
dependencies: []
parent_id: is-01kzqmzewkph9n0w5rzn2a9hyg
created_at: 2026-08-11T05:34:54.426Z
updated_at: 2026-08-11T05:34:54.426Z
---
fdu-py: Index.report(views=[...], include=[...], exclude=[...], min_size='10M', modified_since='2h' | datetime, modified_before=..., kinds=[...], depth=..., limit=..., sort=..., reverse=..., size=...) returning the same Report shape, with identical defaults and names to the CLI and Rust API - a capability reachable by flags but not by a typed call means the library types are wrong (Principle 6). String values accept exactly the CLI grammars through the shared parsers; native datetime and int are accepted wherever a string is. Additive to existing open/scan/Index contracts. Update the installed-wheel smoke to exercise one report per view.
