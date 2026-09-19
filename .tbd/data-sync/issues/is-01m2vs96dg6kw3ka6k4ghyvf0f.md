---
type: is
id: is-01m2vs96dg6kw3ka6k4ghyvf0f
title: Refresh performance hypothesis registry after 0.1.0 refactor
kind: task
status: closed
priority: 1
version: 8
labels: []
dependencies:
  - type: blocks
    target: is-01m2vseyr1w2b7e4sbyxxckx2g
  - type: blocks
    target: is-01m2w0ye3369e1c80m2e0r3dx8
child_order_hints:
  - is-01m2vseybs22svnm03bp16snc8
  - is-01m2vseyr1w2b7e4sbyxxckx2g
  - is-01m2vsez4fppcbra6kc1h9f742
created_at: 2026-09-19T02:52:44.334Z
updated_at: 2026-09-19T05:06:40.354Z
closed_at: 2026-09-19T02:58:15.683Z
close_reason: "Registry honesty pass on origin/main 1193e87d: documented H91-H106 collisions, added H107-H111 for the 0.1.0 engine (request model, gitignore default-on, opened root, sidecar), marked H86 as landed Darwin / failed Linux floor, and absorbed H7/H19-H22/H60. Next free is H112. List is now current; child beads own the measurements."
resolution: null
duplicate_of: null
---
Gate before the 2-hour quiet-machine performance loop on 2026-09-18.

Review current hypotheses across metadata walk / summary files-per-sec, content analysis first-pass and cached lines-per-sec, RSS, and Linux vs macOS as the ledger distinguishes them. Update the registry so it is complete for the current engine (request model, opened root, watch, .gitignore default-on, content sidecar). Add missing hypotheses; mark stale ones with why (refuted / superseded / needs re-measure after refactor). Close only when the list is honest.

This bead is the registry-honesty gate. Child beads may track individual experiments; do not explode into a PR stack. Single branch, single PR.
