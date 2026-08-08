---
type: is
id: is-01kzg4c75tvbrg6rgh3803nwzj
title: Ratify proposed goals 6 (extensibility) and 7 (trustworthy results)
kind: task
status: open
priority: 1
version: 1
spec_path: docs/project/specs/active/plan-2026-08-08-fdu-phase-1.md
labels: []
dependencies: []
parent_id: is-01kzg48ekn4sm0azybr010qgmn
created_at: 2026-08-08T07:28:39.097Z
updated_at: 2026-08-08T07:28:39.097Z
---
The research states five user-set goals plus two the research proposed:
(6) Extensible metrics and pluggable file typing — new roll-up dimensions and type rules must be registrations against stable interfaces (the reducer registry, the type-rule dialect), never engine changes.
(7) Trustworthy results — caching and watching may never silently lie: size+mtime+ctime+inode fingerprints, InvalidateSubtree escalation rather than guessing, corrupt cache treated as empty, stale-while-revalidating always labeled as such.

Both already shape the architecture and are implemented in the scaffold, so a veto would ripple through the reducer design, the type-rule dialect, the fingerprint choice, and the escalation semantics. An explicit yes makes them binding; an explicit no needs to come before phase 1 builds further on them.

Needs maintainer sign-off.
