---
type: is
id: is-01m4fxa1r9zx7yxe0a8phqm82x
title: "fdu 0.5.0: several paths in one report, and an age column in the tree"
kind: epic
status: in_progress
priority: 1
version: 39
spec_path: docs/project/specs/active/plan-2026-10-09-fdu-multiple-paths-and-tree-age.md
delegate: claude-code@spud10
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
hold: null
hold_until: null
created_at: 2026-10-09T08:43:37.094Z
updated_at: 2026-10-11T23:22:40.983Z
started_at: 2026-10-11T23:22:07.655Z
---
Implements the plan spec. Part 1: the tree shows each subtree's age (the list-row activity definition, unknown when incomplete) in text and RFC 3339 + exact ns in machine formats; fdu.report/11. Part 2: PATH... over one or more disjoint roots, merged in the engine before bounds, as if each were run and summed. Answers fdu-onoo and fdu-khu8 open question 2.

## Notes

2026-10-11 stabilization: independently reviewed PRs #191, #192, #194, and #196 merged atomically via stack #193 as d59e6531; its tree equals fully gated f084d9a3. Reviewed README cleanup #195 merged as 3d3f31f3. Full local make check and all five cross-lint targets passed on f084d9a3; 20 CI checks passed on each PR head. Review findings dispositioned; research deferrals retain their own beads. Release prep #197 remains draft at d0862487, tree 67e3ca0f6475d371cec13465569dadc9bdbe3216, with senior/security review complete and 20 CI checks passing. Full release-stability is running from a clean worktree. Merging current main and this candidate produces that same tree. Remaining: final installed-candidate correctness and peer QA, stability record, merge #197, non-publishing GitHub rehearsal/preflight, and pre-tag disposition of fdu-4arl plus the human terminal-window phase. Tag/publication not authorized. The user approved the regenerated color demo; its MP4 and GIF are preserved outside scratch.
