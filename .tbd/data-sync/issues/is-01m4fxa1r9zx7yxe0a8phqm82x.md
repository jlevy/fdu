---
type: is
id: is-01m4fxa1r9zx7yxe0a8phqm82x
title: "fdu 0.5.0: several paths in one report, and an age column in the tree"
kind: epic
status: open
priority: 1
version: 37
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
labels: []
dependencies: []
child_order_hints:
  - is-01m4fxammtqwc8mb1362jgtmhn
  - is-01m4fxankw8s2qrmnnkn3ejs75
  - is-01m4fxapwnccxwmbfezf7gh0tk
  - is-01m4fxar5gf3qrax2k0rwfczn0
  - is-01m4fxasm0nae6c2tbjnqhd6rz
  - is-01m4fxattv1jb2w21ae6814ex9
  - is-01m4fxaw4tnmt77j099fhnk8j6
  - is-01m4fxax3s3274rk55njmnrksk
  - is-01m4fxay0x5wxrjarhc892gfhp
  - is-01m4fxayv9jpnd3nnh65f2q1nw
  - is-01m4fxazfga262r6aj9xfrcqfc
  - is-01m4fyjg32yetj9va263jqezc2
  - is-01m4gc336c85bew881nh38n3xp
  - is-01m4gc33xdg0re1zed4cjdtrmd
  - is-01m4gc34r0v9f5aj7ymepcsacv
  - is-01m4ghgje0a1raa8fwqy6m085k
  - is-01m4ghh5h0wmyx09t533pfm7zx
  - is-01m4gkdgh3n1dxknsxzfxa6avh
  - is-01m4gpwpyt8amak04a7we0xqkz
  - is-01m4gw4em1dbpcwbmgqzw911k9
  - is-01m4gy069faa60h3xjhsfzkdz0
  - is-01m4gy0cxh70ysdbye8wtr2zr5
  - is-01m4gydqnjxszecw5902fe75ve
  - is-01m4hahez6scpwtc6yrg9qws5e
  - is-01m4kjcpw6x6zpv87c9z4m277d
  - is-01m4kjcqgpkc6nf1qtkkj0mmzy
  - is-01m4kjcr3ywmgnc6reyzakeyzc
  - is-01m4kjcrqhyg5h35x1gkf9d8bm
  - is-01m4kjcscwzcrtw7gymgkxeg2k
  - is-01m4kkw2gzm232vd6qgmyqab5p
  - is-01m4kwnfb5r464bewb2nc5bgm3
  - is-01m4mpb135bvc7rjcgran42387
  - is-01m4mybrzvh20q4nkmk5hjs3t1
  - is-01m4p4a90e3qdm7vyv7any3tj3
created_at: 2026-10-09T08:43:37.094Z
updated_at: 2026-10-11T18:42:03.792Z
---
Implements the plan spec. Part 1: the tree shows each subtree's age (the list-row activity definition, unknown when incomplete) in text and RFC 3339 + exact ns in machine formats; fdu.report/11. Part 2: PATH... over one or more disjoint roots, merged in the engine before bounds, as if each were run and summed. Answers fdu-onoo and fdu-khu8 open question 2.

## Notes

Handoff 2026-10-10. Stack: #191 claude/tree-age-column -> main (f2d304a5, ready), #192 claude/several-roots -> #191 (72a8fa79, ready), #194 claude/linux-0.5.0 -> #192 (1a93434c, draft). GitHub stack #193 holds #191+#192; #194 still needs 'gh stack link' into #193 (no valid gh token in the session that pushed). Review E (on #192) is dispositioned: https://github.com/jlevy/fdu/pull/192#issuecomment-6102700391. Remaining before merge: CI green on all three heads; #194 has had NO independent review yet (one was started and stopped unposted to save credit) - run 'tbd shortcut review-github-pr' on #194 with a reviewer that did not write it, address it, mark #194 ready; link #194 into #193; merge bottom-up with merge commits on the maintainer's go-ahead (github-merge: confirm-session). Before tagging: fdu-6iqp (correctness runbook), fdu-8k4k (peer standing), fdu-4arl.
