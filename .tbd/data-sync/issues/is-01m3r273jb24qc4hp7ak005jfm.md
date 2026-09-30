---
type: is
id: is-01m3r273jb24qc4hp7ak005jfm
title: "0.3.0 stability pass: fix every open bug that is fixable and testable on Linux"
kind: epic
status: closed
priority: 0
version: 44
labels: []
dependencies: []
parent_id: is-01m3rhbj53p4j4cghcyd4d3ak0
child_order_hints:
  - is-01m3jhrr65wen7ak684hvm351w
  - is-01m2f3tr24csc75neqfhsvg2dv
  - is-01m2f0g6bsatn9gksxtt4zvytr
  - is-01m2f0g2ecz6wyen1jzxy966tg
  - is-01m3p2jw3pqs0n77e5ahrgj8qf
  - is-01m3n9myxhkn2y5s7kebqsk7pt
  - is-01m3qazvzbkyxxk1gm1jwvy1h9
  - is-01m0jzc327j8f7dzya1k83n0kc
  - is-01m2pj0eknqard0k8za13xygp8
  - is-01m2ymchsj7j6ayg8j46kc1v00
  - is-01kzypet6xhmxy6e5jtd17ww27
  - is-01m3gqwpvr0n844y9pj0gj3cwj
  - is-01m0pqk1zqhx9tbhjz436n4pse
  - is-01kzsrpjnmtrz73y65sa1w1v33
  - is-01m2gahg0pdyfxr9yxy0wce5e7
  - is-01m3jba39zq8w3nqmtv0npvk5f
  - is-01m1687g2cazrcaxzwkpdcazz5
  - is-01m3neeeq2v5yafq673ntr55zj
  - is-01m3neef6bv0c3baw3gahf4w6b
  - is-01m2et32219k93fnvccckv623q
  - is-01m3neeg5rebr5bebax9e5prw8
  - is-01m3kdqg02e0e65fp35q7gt1zc
  - is-01m3jw3yeztmc9rmctbjbzyv4s
  - is-01m3jfhpv1zppwmdrk2e6ks88r
  - is-01m3nedew24rmcnjfc97jtf0z4
  - is-01m3e280xmajj519s9570gjnz5
  - is-01m396f1gp8fxsnx3w65vq1q15
  - is-01m3neefnak4j3hwn82k91d3z2
  - is-01m2esgr7z3zqmx7fskdj3kfzx
  - is-01m2f3b56x0e7sfe7dkamk1pzp
  - is-01m2f3b5jgcfk96wq4r52xcag7
  - is-01m3ky3643py36em7htzjqmgqw
  - is-01m2esgs0skcccfsvsej970xcp
  - is-01m0t8a3h35a182tbfacgwgzey
  - is-01m3qfdysq94y73x0785c857ck
  - is-01m32ewmqkv1v71f5pjtc3djmx
  - is-01m32exka5myg890p63v94yenw
  - is-01m3rfnvx470zbrwgdc63152ct
  - is-01m3rfz36eh8m7xp52aq5c0kmz
created_at: 2026-09-30T02:27:37.162Z
updated_at: 2026-09-30T13:29:17.775Z
closed_at: 2026-09-30T13:29:17.775Z
close_reason: "Done on #164: every open bug fixable and testable on Linux is fixed with a test, closed as already fixed, or closed as not reproducible, with evidence on each bead; the review round's findings are fixed too. Still open, and not fixable here: fdu-3v0d (the code is in MetaBrowser), the macOS/Windows-only beads (fdu-43bc, fdu-ek21, fdu-9tul, fdu-vhrb, fdu-hb2t, fdu-syyl), fdu-6o5o (reproduced; an exact filtered fold is a design change, analysis in its notes), fdu-v71x (below the 3% bar), and fdu-k90t (the liftable Markdown bound)."
resolution: null
duplicate_of: null
---
Maintainer request 2026-09-30: top priority is that the release is stable and clearly faster, so fix all bugs. Triage of 53 open bug beads: A) 17 engine correctness/stability bugs fixable and testable on Linux (fix now, test first, one commit each); B) 20 gate/tooling/harness bugs (fix now); C) 7 macOS/Windows-only bugs that cannot be reproduced here (need a macOS/Windows session); D) 9 design or measurement tasks filed as bugs (relabel). Work lands on branch claude/stability-fixes, stacked on #162 (claude/readme-comparison-matrix).

## Notes

2026-09-30: the maintainer confirmed the release scope: 0.3.0 = #157 -> #158 -> #161 -> #162, then the stability PR (claude/stability-fixes with claude/stability-tooling merged in, epic fdu-l4u1), then the pdu track (claude/pdu-uniform-lead, epic fdu-faqa) on top, as one linear stack merged bottom to top with merge commits. Still open for the maintainer before tagging: fdu-8f6k (#[non_exhaustive] Counts), fdu-q7hf (reader diagnostics fields in 0.3.0), fdu-4nue (JSON default view).

2026-09-30: maintainer approved the recommendations: fdu-8f6k (Counts #[non_exhaustive]) and fdu-q7hf (reader diagnostics fields, port of bda4ade1) ship in 0.3.0 on claude/stability-fixes; fdu-4nue's output is by design (surface architecture: machine List materializes every row) and the bead is now the performance follow-up for that machine path.
