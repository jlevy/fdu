---
type: is
id: is-01m2wa4h28x10zxdmk3f8zqp08
title: "H120: stream sidecar parse-into-apply to cut cache-hit RSS"
kind: task
status: open
priority: 2
version: 1
spec_path: docs/project/specs/active/plan-2026-09-19-post-h115-remaining-headroom.md
labels:
  - performance
  - campaign-2
  - macos-agenda
dependencies: []
parent_id: is-01m2wa49kmg1xct59bpbdxvs77
created_at: 2026-09-19T07:47:17.191Z
updated_at: 2026-09-19T07:47:17.191Z
---
content-cache-hit peak RSS is ~380 MiB on 146k files. Stream parse into apply so the decoded records Vec and the files map are not both live. Accept: peak RSS >=10% down; wall non-inferior (interval not entirely above +3%); digest identical. Re-screen after H116, which may take the HashMap half.
