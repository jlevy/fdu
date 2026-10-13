---
type: is
id: is-01m4pfhrrvbd19h3wwrqte3zan
title: Prepare the 0.5.0 release identity, notes, demo, and stability evidence
kind: task
status: in_progress
priority: 1
version: 17
delegate: claude-code@spud10
labels: []
dependencies: []
child_order_hints:
  - is-01m4pgz5p91v6x7v1yx3eafq0j
  - is-01m4phmzw6t4vc86hpgmkspfef
  - is-01m4ppd9j28xdn96t9pw9ngeve
  - is-01m4ps2pk8vt2j5kqhvrtnazec
  - is-01m4pszg95306272ng2aq96vjy
  - is-01m4q9ea978cq5yqvswrpksbyc
  - is-01m4qd8phf7xrm86zsnymre3ft
  - is-01m4swms8xg4h4r2x9trvgptkq
  - is-01m4t5mgqyj857ap8je2mzsbss
  - is-01m4t5r6f12yjxnpyq8nd7chy5
hold: null
hold_until: null
created_at: 2026-10-11T21:57:50.991Z
updated_at: 2026-10-13T08:23:33.588Z
started_at: 2026-10-11T21:58:48.661Z
---
Follow the release checklist after merging stack #193. Prepare the version bump and changelog, update the approved demo asset declaration and GIF, document current Linux evidence without unsupported lead claims, run release stability and rehearsal, and stop before tagging or publishing pending explicit release authorization. Parent stabilization fdu-6uwu. Include fdu-eruq citation and fdu-8k4k claim qualification.

## Notes

Release preparation merged in #197 at 33a3a890, identical tree to d0862487 and rechecked 6117ae41. Preflight and release-body passed. Non-publishing rehearsal38187708393 passed; eight files downloaded and checksum-verified, all platform install smoke checks passed. Evidence and procedure records merged in #198 at 3d296c15, exactly the independently reviewed abcb4e29 tree; complete make check passed in1313s and all20 PR CI jobs passed. All20 merged-main CI jobs passed in38192001461. Earlier disk-full documentation gate is explicitly incomplete and superseded by this complete run. Correctness-runbook issue fdu-6iqp closed with merged evidence; review corrections fdu-uibg/fdu-53y2 closed. Tag remains blocked by fdu-wo7c live Library peer discrepancy, fdu-4arl Linux performance disposition, and human terminal-window QA. No tag or publishing dispatch.

2026-10-12: Corrected immediate-start 26.4s video and matching README GIF/declaration merged in PR #200 at 924007e477eab2ffc2ee9daaaa73fc0645751e92, tree identical to reviewed/gated a0f8796. Independent strong review B and review A found no issues; all20 PR checks passed. All local make check components passed across the disk-guarded run and exact preserved-binary terminal continuation; not an uninterrupted exit0. Demo tasks fdu-op78 and fdu-ztv6 closed. Previous untagged candidate33a3a890 retired with documented exact-commit release-cleanup --abandon; evidence preserved. Fresh final-candidate directory release-0.5.0-924007e4 under durable external release evidence. Preflight passed; body generated; non-publishing rehearsal38295552345 and main CI38295432083 running. Final stability preflight refused3.5GB free against12GB requirement; no waiver. Bounded Library diagnosis did not reproduce the historical29232B residual and attributed288B correction drift; fdu-wo7c remains unresolved. No quiet Linux host route available for fdu-4arl, and human/Windows terminal QA remains pending. No tag or publication.

Final update: rehearsal38295552345 completed successfully; all8 packages downloaded and checksum-verified, all platform smoke installs passed, publishing and announcement skipped. Main CI38295432083 passed all20 jobs. release/v0.5.0 names924007e477eab2ffc2ee9daaaa73fc0645751e92; v0.5.0 tag absent. Final readiness recorded under durable evidence/release-0.5.0-924007e4/readiness.json; prior candidate record preserved. Independent preparation complete; remaining stability disk capacity, fdu-wo7c, fdu-4arl, and human/Windows terminal gates are not waived. Heartbeat paused pending required host capacity and maintainer input.

2026-10-13 final documentation refinement: PR #201 merged at e6b8850b287785fbea38f2b89fed643b5846ef6f, tree identical to independently reviewed head 06f3fb4e. Complete local make check exited 0; all 20 PR checks and all 20 merged-main checks passed (main run 38359485996). All independent review findings resolved. Bead fdu-sbjn closed. No open fdu PRs remain; local checkout is clean main.

Previous untagged 924007e4 candidate retired with exact-commit lease; evidence preserved. New release directory release-0.5.0-e6b8850b has passing preflight, derived/inspected release body, and successful non-publishing rehearsal 38359873752. All eight files downloaded and checksum-verified; platform install and API compatibility checks passed; publication and announcement skipped. Exact rehearsed Mac arm64 wheel installed through uv as fdu 0.5.0, installed public-API smoke passed, existing .agents and .claude skills refreshed.

Cleanup removed three fully merged owned worktrees, obsolete fdu build/demo artifacts, and the completed validation target/cache after preserving its exact tested binary and complete evidence. Other-project cleanup was rejected by automatic approval review; an additional explicit authorization question is pending, and no such data was removed. Last physical free space was about 9.8 GiB (10.5 GB), below final stability's 12 GB floor. Final candidate stability preflight refused 9.6 GB at its observation; no override. Library fdu-wo7c, Linux performance fdu-4arl, and human/Windows terminal QA remain open. User has authorized end-to-end publication after gates, but no tag or registry publication occurred. Heartbeat remains paused. Final readiness is preserved under release-0.5.0-e6b8850b/readiness.json.
