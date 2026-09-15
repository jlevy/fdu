---
type: is
id: is-01m2gv6zy176z64mkjng34kmc4
title: CLI shows gitignored share in summaries and tree rows, with --exclude-ignored and --only-ignored
kind: feature
status: closed
priority: 1
version: 4
labels:
  - stack-followup
  - release
dependencies: []
created_at: 2026-09-14T20:54:50.553Z
updated_at: 2026-09-15T22:17:59.839Z
closed_at: 2026-09-15T22:17:59.838Z
close_reason: "ae46261, 3ddec4c, ea671de (PR #65): summary, tree, and extension rows carry an ignored share, text '(N ignored)' after the detail (Q5), zero object in JSON and no text suffix when nothing is ignored (Q6), null under --no-gitignore; --exclude-ignored and --only-ignored filter entries so sizes, sort, and --min-size follow (Q4); goldens over a project/.gitignore fixture; grouped views follow in fdu-12zs (Q8)"
resolution: null
duplicate_of: null
---
DECISION (user, 2026-09-14): once reports observe .gitignore by default, the CLI shows how much of each size is gitignored, for example '1.2 GB (340 MB ignored)', in summary and tree rows, and adds --exclude-ignored and --only-ignored filters. JSON output carries the ignored/unignored split. Human output changes, so goldens and the Python parity corpus change too; read every diff. With --no-gitignore the split is omitted and never shown as zero. Depends on the default-on bead.

## Notes

2026-09-15 DECISIONS (user), PR B design (plan: scratchpad/reviews/plan-gitignore-default-on.md, section 7):
Q4: sort and --min-size use the displayed size: total by default, unignored under --exclude-ignored.
Q5: text output uses a plain detail suffix everywhere, '(N ignored)' after the size, in summary and tree rows. No bar shading.
Q7: the transient summary tier falls closed to FullIndex so it can classify. Measure aggregate-summary wall and RSS in the speed check; if the gate fails, build the streaming classifier before merging.
Q10: an unreadable .gitignore is an operational error with exit 2 unless --allow-partial; --no-gitignore is the escape.
Recommendations taken without asking:
- Q6: zero ignored under observation omits the text suffix, and JSON carries a zero object.
- Q8: annotate extension rows in B, not types/families/languages/documents; follow up afterwards.
- Q9: the flag is --no-gitignore (the user's name).
Release: all of this ships in 0.1.0.
