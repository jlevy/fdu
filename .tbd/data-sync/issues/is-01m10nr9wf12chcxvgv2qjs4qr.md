---
type: is
id: is-01m10nr9wf12chcxvgv2qjs4qr
title: Bind the injected registry, identities, and one discovery budget in the Python provider
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nra8p8h2pcb121nrqfe7c
parent_id: is-01m0y1sg8emg0sgyv1pj8sa6x7
created_at: 2026-08-27T03:55:53.614Z
updated_at: 2026-08-27T07:56:13.520Z
closed_at: 2026-08-27T07:56:13.519Z
close_reason: "Completed in MetaBrowser commits 0a6ddbb and 45266a8: injected registry content, derived identities, and a single discovery budget pass the full gate."
resolution: null
duplicate_of: null
---
Update PythonInventoryBackend construction/start and _PythonInventoryStore state to parse and retain the supplied registry, derive scope and semantic identities, and enforce one DiscoveryBudget across the initial walker, subtree rewalk, refresh, and live application. A refused expansion leaves the handle readable and prevents watcher startup or later expansion.
