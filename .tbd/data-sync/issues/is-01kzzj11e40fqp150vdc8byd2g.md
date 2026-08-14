---
type: is
id: is-01kzzj11e40fqp150vdc8byd2g
title: "S5 note: do not schedule per-parent roll-up accumulation separately"
kind: chore
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-08-14T07:15:49.316Z
updated_at: 2026-08-14T07:15:49.316Z
---
Recorded so nobody re-runs a dead end. merge_upward walks to the root per entry, so a depth-20 tree merges twenty times per file, and accumulating consecutive same-parent contributions looks obviously worth doing. H13 tried exactly that and was refuted at -2.5 percent, because H18's interning had already taken the expensive part of each merge and the two competed for one cost. The current cold-scan profile puts merge_upward at about 2 percent of engine work, so as a standalone change it is below the bar. It is not worthless - S4's bottom-up pass computes each directory's roll-up once from its children and the O(N*D) disappears as a consequence. Fold it into S4 rather than scheduling it, and if S4 is abandoned do not revive this on its own without a fresh profile.
