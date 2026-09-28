---
type: is
id: is-01m3mg15aem39d0chrsv1swtyb
title: "QA: qa_peer_agreement.py cannot pass on 0.2.0 (1% default hides small dirs; files listed)"
kind: task
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m3mcwynm1rkdjencnq5621mq
created_at: 2026-09-28T17:12:04.685Z
updated_at: 2026-09-28T23:04:04.552Z
closed_at: 2026-09-28T23:04:04.552Z
close_reason: "Fixed on claude/fdu-alternatives-research-qx0xn0 (PR #155; merges 2d1eea9d, e0923063); CI green at ae650448; release changes independently reviewed (no blocker; low findings in follow-up, signer pinning fdu-8k8s)"
resolution: null
duplicate_of: null
---
scripts/qa_peer_agreement.py:189 omits --min-share 0% for fdu's reading, so under 0.2.0's 1% default small top-level directories look absent; line ~223 needs child kind == dir, since 0.2.0's tree lists files and du --max-depth=1 does not. A patched copy passed Phase 7 on 2026-09-28 (see PR #152 body).

## Notes

2026-09-28: implemented on claude/fdu-alternatives-research-qx0xn0 (PR #155), merged at 2d1eea9d/e0923063. f5288280: fdu reading passes --min-share 0% and keeps kind==dir children. Pending: independent review and CI, then close.
