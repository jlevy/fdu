---
type: is
id: is-01m3gv9czxppnb6wxq55b8vz7v
title: Sync upstream and consolidate docs, skill, and plan contracts
kind: task
status: closed
priority: 1
version: 5
labels: []
dependencies: []
created_at: 2026-09-27T07:11:51.292Z
updated_at: 2026-09-27T07:55:15.494Z
closed_at: 2026-09-27T07:55:15.493Z
close_reason: Upstream integrated across the stack; emitted skill, offline guide, plan status, and docs index consolidated and reviewed. Local verification passed after the reviewed Linux parity recording update; all 19 final CI jobs passed on every stack layer. Review and exact commit/run evidence are in notes and PR descriptions.
resolution: null
duplicate_of: null
---
User requested upstream integration and consolidation of documentation/embedded skill against the current plans and new design. Sync the existing PR stack to origin/main, reconcile semantic overlaps, audit current and historical guidance, verify skill and docs, run required gates, push and wait for CI.

## Notes

Synced all three PR layers cleanly onto upstream 4c4917f4. Auditing embedded skill, offline CLI guide, usage/schema docs, current plan status, and TODO navigation. Preserve dated measurement/research evidence; current docs must match implemented alpha contracts.

2026-09-27 consolidation: synchronized stack onto upstream 4c4917f4; implementation 44f106ae and docs 91c8f93f. Emitted skill and offline docs now match current analyzer/view defaults, limits, cache/ignore semantics, inventory totals, and allocation caveats. Plans/TODO distinguish implemented contracts from historical defaults and real follow-ups. Full make -k check (1002.16 s) passed all targets except exactly four offline-help lines in the Linux-owned parity record. Adopted the reviewed artifact from CI run 36303485989 and make parity-check now matches all 25 deviations. Goldens 185; path independence 2267 cases; docs formatting and isolated skill installation passed. Final CI pending: implementation 36303716655, docs 36303715622. Research 36302371253 passed all 19 jobs.

Final completion: all 19 CI jobs passed on research 430abbe3 (run 36302371253), implementation 44f106ae (run 36303716655), and docs 91c8f93f (run 36303715622). The duplicate docs run 36303715303 also passed all 19. Separately scheduled full path-independence workflow was skipped, not counted as passed. Consolidation review published at https://github.com/jlevy/fdu/pull/133#issuecomment-5853882783 with no unresolved findings. Verification fdu-arv8 is closed; fdu-65x1 stays open for fdu-2udc, fdu-uea9, fdu-afc4. No PR merge, release, or tag was performed.
