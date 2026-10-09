---
type: is
id: is-01m47z5n33np3b15yena1k3hqt
title: "Release fdu 0.4.0: notes epilogue + content views imply analysis"
kind: epic
status: open
priority: 1
version: 29
labels: []
dependencies: []
child_order_hints:
  - is-01m47emwq781svq0crdbgccpbz
  - is-01m47zjtznrvpex6edd3kagpt9
  - is-01m480t2frm92ses02tfr36tgv
  - is-01m482372az7g2jh21mv7abgm9
  - is-01m482enn67fyky9w886zv0f44
  - is-01m483r2p8b4n89whx1q9ny4p5
  - is-01m4890ntmsbhjrc6accmtpay9
  - is-01m4890pbx6ptfke3z0np9wm57
  - is-01m4890pwshjnenvprkn7gh3w5
  - is-01m4890qg7xpqq58xkk47gmx0j
  - is-01m4890r5ejj9emqn1vb9gptdn
  - is-01m489wty6g4m38wvps5k7snn5
  - is-01m48bsg7p862abytve8fz42ks
  - is-01m4d4qejw29ggmcha47ycj9za
  - is-01m4d4r3tg1v9qswn105nrqy1f
  - is-01m4d4r46bedn0er8e3v9ycqe0
  - is-01m4e2x6cbpz86t2tsh7chk48x
  - is-01m4eax1x6y2vth78ghr8rxmtj
  - is-01m4efe7zqcg5ttphf74yv33pj
  - is-01m4femwnqfs8k9tsg40e4yzen
  - is-01m4fhwc5rj2jhyq65vjc6gnrm
  - is-01m4fn9ntt9xy415w7bb35xy86
  - is-01m4fn9paq6cpqznw00ycka7qc
  - is-01m4fn9prqtf3qkq29agaa0q0j
  - is-01m4fs9fy2v3dpsxe85edw5z38
  - is-01m4ftg7y0q3zkgcje4wdvpzfy
  - is-01m4fth9hpx7xx6tdgvqxb0mqf
created_at: 2026-10-06T06:42:14.754Z
updated_at: 2026-10-09T07:55:08.724Z
---
Bring stack #174 -> #177 (plus the document-shares fix) through review, merge, release prep (version, CHANGELOG date, release notes), the release checklist (stability pass incl. correctness runbook, preflight, rehearsal, body), tag and publish (maintainer go-ahead), and a final demo video from the released build.

## Notes

PUBLISHED 2026-10-09 07:26:50Z. fdu 0.4.0 = release commit c041ed1c8ac5c588ac4fe964ab95bc14f1e5b0ac (#189 merge; tree 2c728b23), tag v0.4.0 (annotated, unsigned, object 81eb6a96, GitHub verified). Stability pass 15/15 on tree 2c728b23 (run as f405067d via make release-stability ARGS=... with FDU_QA_* exported; record /Users/levy/fdu-release/0.4.0-pass2/stability/). Preflight all ok (DEMO matched the declaration). Rehearsal https://github.com/jlevy/fdu/actions/runs/37893372604 (8 files verified; local notes.md byte-identical to the rehearsal's announcement notes, sha256 f2ad4aef...). make release-demo created the draft (first attempt hit the listing lag fdu-x81v; rerun reused it) and attached fdu-demo.mp4 (3,784,222 bytes, sha256 31e720b3..., state uploaded). Publishing run https://github.com/jlevy/fdu/actions/runs/37897343884 (all jobs success; release environment approved by the agent on the maintainer's explicit end-to-end go-ahead). Release https://github.com/jlevy/fdu/releases/tag/v0.4.0: immutable, 12 assets. make release-published: crates.io fdu-core, crates.io fdu, PyPI fdu all identical. make release-announced --cargo: body matches notes.md; 12 assets match (11 + declared demo); docs.rs fdu-core and fdu built; uvx fdu@0.4.0 and fdu@latest print fdu 0.4.0; cargo install --locked fdu prints fdu 0.4.0. First release with automatic announcement of a declared demo: Announce on GitHub succeeded and all twelve verified. Homepage: README GIF served from raw/main as image/gif (1,451,807 bytes, sha256 39a54b05...), renders and animates; releases/latest/download/fdu-demo.mp4 returns the declared MP4 byte for byte. release/v0.4.0 deleted by make release-cleanup.
