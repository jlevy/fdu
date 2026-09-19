---
type: is
id: is-01m2wa4h28x10zxdmk3f8zqp08
title: "H120: stream sidecar parse-into-apply to cut cache-hit RSS"
kind: task
status: closed
priority: 2
version: 3
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
parent_id: is-01m2wa49kmg1xct59bpbdxvs77
created_at: 2026-09-19T07:47:17.191Z
updated_at: 2026-09-19T09:09:32.124Z
closed_at: 2026-09-19T09:09:32.123Z
close_reason: |
  exp-117: accepted. content-cache-hit peak RSS -10.13% [-10.49%, -10.03%] on metabrowser-clone. Wall non-inferior. Streaming restore kept. Overnight queue done.
resolution: null
duplicate_of: null
---
content-cache-hit peak RSS is ~380 MiB on 146k files. Stream parse into apply so the decoded records Vec and the files map are not both live. Accept: peak RSS >=10% down; wall non-inferior (interval not entirely above +3%); digest identical. Re-screen after H116, which may take the HashMap half.

## Notes

H120 / exp-117 pre-register (2026-09-19).

Claim: cache-only restore holds the decoded sidecar records Vec beside the files
map. Streaming parse-into-apply drops that transient copy.
H116 already failed to remove the HashMap; this is the Vec only.

Job: content-cache-hit peak RSS on deciding-scale metabrowser-clone.
Accept: peak RSS down at least 10%; wall non-inferior (interval not entirely
above +3%); content digest identical.
Control: HEAD 984e4618 (H115 in; H116/H118 reverted). FDU_COUNTERS unset.

Quiet first; uncontrolled OK. No RAM disk. Revert engine on reject.
