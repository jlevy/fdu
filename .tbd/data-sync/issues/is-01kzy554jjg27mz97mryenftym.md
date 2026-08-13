---
type: is
id: is-01kzy554jjg27mz97mryenftym
title: Linux performance validation and optimization
kind: epic
status: open
priority: 1
version: 18
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-end-to-end-performance-testing.md
labels:
  - performance
  - linux
  - handoff
dependencies: []
parent_id: is-01kzkzm62q1vwxbv9hbp39bxxm
child_order_hints:
  - is-01kzwt2d8dapafh6tmf9f92gek
  - is-01kzxsh1gqs697dy3eat13fw22
  - is-01kzxsmcabr3shfgh9644tbdtg
  - is-01kzy2ewdvwzgfseqa531rvwvv
  - is-01kzy2fc4js5y0yew4q1vbjhdv
  - is-01kzy2fd99hfynhpk491bsnqp6
  - is-01kzy2qt789svdbes8g3656788
  - is-01kzy2qv7fkcwjcn3g8gas7g4m
  - is-01kzy2rgkz4gjcxknk6jpsr5wd
  - is-01kzy2rhmtepkkstj4wed7am34
  - is-01kzy2rjnap5ewjbz8seft57sb
  - is-01kzg49rw1p40pjc18feb9ghpv
  - is-01kzg4d2saym31t884vf6me2p7
  - is-01kzwk20bb97hagzjeegkxpd77
  - is-01kzn04cqdaaknww7941cbp7aw
  - is-01kzqynv3k4rf6gb6cddsnz93e
created_at: 2026-08-13T18:11:37.668Z
updated_at: 2026-08-13T18:12:14.316Z
---
Own the post-PR-#8 Linux program on controlled Linux hosts. Establish claim-grade warm, pagecache-drop-only, and controlled-cold matrices on local SSD with exact binary/host/filesystem/corpus provenance, full semantic oracle, paired adjacency, resource counters, and pre/post fingerprints. Use the portable backend as the correctness baseline; reproduce the first scouting results before changing production. Then prioritize default-view retained-state/index work, warm snapshot load/save costs, allocator and Linux-only stat-elision hypotheses, worker calibration by cache regime/filesystem, and explicit closure of io_uring or inode-ordering ideas when bare-metal evidence rejects them. Keep macOS-only H69/H70 and controlled-macOS-cold work outside this epic.

## Notes

Handoff order: (1) reproduce the first Linux scouting evidence on a controlled local-SSD host via fdu-nffc; (2) default rendered-tree gap via fdu-sk7v plus retained-state/index work; (3) warm inversion via fdu-maxn/fdu-niuz/fdu-91ts; (4) portable summary via fdu-cckr/fdu-i2f3; (5) cold worker calibration via fdu-tk1b; (6) close or qualify io_uring/inode-ordering with fdu-dzs0/fdu-lf3v. PR #8 already carries cross-platform differential correctness tests; do not mix APFS-only H69/H70 into Linux conclusions.
