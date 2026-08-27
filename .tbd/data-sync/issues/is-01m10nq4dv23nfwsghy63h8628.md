---
type: is
id: is-01m10nq4dv23nfwsghy63h8628
title: Run the installed-wheel MetaBrowser end-to-end spike lifecycle
kind: task
status: closed
priority: 1
version: 3
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - opened-root-rewrite
dependencies:
  - type: blocks
    target: is-01m10nq4rd347gj3makrvtrd5m
parent_id: is-01m0y1sfedr9qf3sc7e4bf6fd7
created_at: 2026-08-27T03:55:15.258Z
updated_at: 2026-08-27T06:12:06.249Z
closed_at: 2026-08-27T06:12:06.248Z
close_reason: Completed on MetaBrowser branch codex/fdu-opened-root-e2e-spike at commit 2743064 against exact fdu wheel revision 0583a1a. The normalized evidence and reproduction commands are under explorations/fdu-inventory-adapter; MetaBrowser make verify and strict exact-wheel typing pass. The lifecycle covers progressive and settled reads, live change/reread, refresh, root replacement, iterator cancellation, concurrent close, and zero surviving workers.
resolution: null
duplicate_of: null
---
Run the unchanged provider contract cases and provider-neutral inventory routes, then one installed-wheel lifecycle covering cold open, useful progressive read, completion, live mutation, changes, coherent reread, refresh, root replacement, iterator cancellation, and joined close. Include filesystem-to-SSE and browser-lifespan boundaries with fdu explicitly selected by the spike harness.
