---
type: is
id: is-01m2h6a9wd6f6xexfaw93whryr
title: "Release readiness: stabilize, verify, and publish fdu 0.1.0"
kind: epic
status: in_progress
priority: 0
version: 38
labels:
  - release
dependencies: []
child_order_hints:
  - is-01m2h7hdwc2v5fa8c065ypascy
  - is-01m2h7hnm1bh7s4abvp8b8czgt
  - is-01m2h7j050f97s4hctpda4nzje
  - is-01m2h7jd6sqqr8n05g4375s6yt
  - is-01m2jzadk7w8m1xcsewzwg5wj1
  - is-01m2k27z25tt9ygs4c1nchhzez
  - is-01m2k2pp6jw10vq759yyetmcmb
  - is-01m2nrqeqa5pf3j4d548wygd0a
  - is-01m2phvqccpjdrbbsr5y3kcydr
  - is-01m2phzbwx3qzvf77e6xajdsj9
  - is-01m2phzc8bg4js1chd86rs8pyj
  - is-01m2phzcjkthhqf430p6mhdmad
  - is-01m2phzcwqp3jxmcb2gsxb3n5q
  - is-01m2phzd7cxjn134d9t78xzw6d
  - is-01m2phzdhny16vmrqfv9a6cejz
  - is-01m2phzdw3ynn562960wk9jfw0
  - is-01m2phze6caa00vy4yfkrf6xm3
  - is-01m2phzegm4b3scda7d1xq3gnm
  - is-01m2phzevyf68fdz9zcs3yzncw
  - is-01m0k512k9a6dq2k51fbfe5xn4
  - is-01kzypf1yd2v4g8q8tk2v1xmxs
  - is-01kzg4c6vnh98mqrpkzw7ydne0
  - is-01m2pmr9aftb3vzfyjyanndny3
  - is-01m2pj0hc166t091a4k04kks9t
  - is-01m2q1amctg37zcrd417xcmjzs
  - is-01m2s0531fnn7j34zfs8t00e10
  - is-01m2s053k7cjccm6c6c4rwck2f
  - is-01m2s0tq4ppsygrs129nw1m86n
  - is-01m2sgadh8f84z9wxtrhhzszk9
  - is-01m36ajmynyejms4zkynsyrvcz
  - is-01m395wcbdbx8ndh16agfhmrhy
created_at: 2026-09-15T00:08:53.387Z
updated_at: 2026-09-24T07:46:03.321Z
---
User goal (2026-09-14): bring fdu to a stable state that can land on main and cut a release. Keep making progress, track everything as beads, stack PRs as needed, and make sure the final PR stack is complete.

Scope, to be refined by the readiness audit:
1. Land the open PRs: #56 -> #57 -> #60 (GitHub stack), plus #58 and #55. Each must be reviewed through its head, fixes addressed, and CI green.
2. The default-on .gitignore work the user decided (fdu-elnn, fdu-5ryb), with its prerequisites (fdu-1onj, fdu-okne, fdu-szkg), stacked on #60.
3. Release blockers found by the audit: open P0/P1 beads that affect correctness, docs accuracy, CHANGELOG and release notes (fdu-apbl), the version, and the packaging/release workflow rehearsal (docs/project/guides/release-process.md).
4. The final pass: make check and cross-lint on the combined tree, the local release rehearsal, and release notes. The user cuts the release.

Tracking: fdu-6nyd (the earlier merge-readiness bead) covered the first stack, which has merged.

## Notes

2026-09-15 ~00:30 PDT status:
- Open PRs: GitHub stack #59 (#56 -> #57 -> #60), plus #58 and #55, which stand alone on main. Every one is 19/19 green and CLEAN. #58 and #55 are independent of the stack and merge cleanly with it and with each other, so they are deliberately not stacked.
- Review disposition audit:
  - #58: complete, both reviews answered.
  - #60: technical review answered; delta review running.
  - #55: delta review being addressed (fixer running).
  - #56: its technical review's LIFE-1/2/3 and delta review being fixed (fixer running; it must post replies for both).
  - #57: technical review 5200240760 never got a disposition map, and delta review 5204082880 is unanswered. Assigned to the #57 agent with the address-pr-review shortcut.
- Release-readiness audit (Fable) is running; it will create 'release' beads under this epic.
- Next: propagate #56's fixes into #57 and #60, run final delta reviews, then the default-on .gitignore PRs (fdu-elnn, fdu-5ryb, prerequisites fdu-1onj/okne/szkg) stacked on #60, unless the audit recommends deferring them.

2026-09-15 DECISIONS (user): release 0.1.0 (not 1.0.0). All of the default-on .gitignore work ships IN 0.1.0: PR A (bounds degrade, dedup, liftable budget: fdu-1onj, fdu-okne, fdu-szkg) and PR B (default on for every surface, --no-gitignore, the CLI (N ignored) split, --exclude-ignored/--only-ignored, the JSON schema bump: fdu-elnn, fdu-5ryb). The README headline performance figure is re-measured on the release candidate before tagging. The first release is published by hand from the signed tag (fdu-core, then fdu, then the PyPI wheel), following the release guide; automated publish jobs come later.
The readiness audit report is at scratchpad/reviews/release-readiness.md. Release blockers: fdu-y5zc, fdu-0t6d, fdu-k4ad, fdu-ih88, fdu-1onj, fdu-qy8e, fdu-ls14, fdu-9cf0.

2026-09-15 MERGED to main, with user authorization ("be sure all issues are addressed fully, and then you can merge the PR"). Every review on each PR had a disposition reply, and each PR was 19/19 green and CLEAN.
- Stack #59 (#56 fix(engine), #57 contract decisions, #60 gitignore build feature removed): merged via gh stack merge --merge as 5f03062.
- #58 (exp-104 evidence): 8856b4d.
- #55 (checkpoints plan): 3373134.
- #61 (release-workflow fixes): delta review of its review-fix commits running; merges after that.
Next:
- CI on main at 3373134.
- Then PR A (fdu-1onj/okne/szkg) and PR B (fdu-elnn/5ryb) on main, blocked on disk.
- Release-prep PR: fdu-k4ad, fdu-ih88, fdu-qy8e, fdu-y5xr.
- Combined make check and release rehearsal (fdu-ls14), then a local install for final testing.

2026-09-15 verification of merged main at 3373134:
- CI green (run 34999813497).
- Local `UV_PYTHON=3.12 make check` and `make cross-lint` both pass through the lock wrapper; parity holds (21 recorded deviations matched).
- A release CLI built from a clean checkout reports fdu 0.1.0-dev+g337313468 and passes the smoke checks.
Open for 0.1.0:
- #61 (release workflow): all findings fixed, final check running.
- #62 (shipped text): fdu-8dou fix running.
- PR A (control bounds, fdu-1onj/okne/szkg): in progress.
- PR B (default on and CLI split): after PR A.
- Release prep: fdu-qy8e CHANGELOG, fdu-y5xr perf figure, fdu-ls14 rehearsal.

2026-09-15 merged:
- #61 (release workflow; crates.io audit fixed and verified against the live registry) as 2007d82. Main CI green.
- #62 (shipped text: fdu-k4ad, fdu-ih88, fdu-8dou; README doctests and a Python README-example test) as f047dab. Main CI running.
All reviews on both have disposition replies. Release blockers now closed on main: fdu-y5zc, fdu-0t6d, fdu-pd1b, fdu-k4ad, fdu-ih88, fdu-8dou.
Remaining for 0.1.0:
- PR A (fdu-1onj/okne/szkg), in progress;
- PR B (fdu-elnn/5ryb);
- fdu-qy8e CHANGELOG and release notes;
- fdu-y5xr re-measure;
- fdu-ls14 rehearsal dispatch (needs the user's go-ahead);
- fdu-9cf0 publish by hand (user).

2026-09-17 STATUS AND RE-SCOPE. Everything planned for 0.1.0 merged by 5f2d36d (#63-#76); CI and release
rehearsal 35156068769 green; the 8 artifacts match SHA256SUMS. Five read-only audits and end-to-end testing
of the rehearsal wheel on macOS then found release blockers, now children of this epic:
- fdu-gija: content analysis answers depend on cache history (design problem: the analysis request is
  index state, not a report parameter; post-release redesign in fdu-azz3). Fix approach awaits the
  maintainer: exact-match reuse or request-scoped projection.
- fdu-snv3: watching an analyzed index serves stale metrics (fix on claude/release-e2e-fixes).
- fdu-18vk: wheel console command ignores Ctrl-C during --watch (fix on the branch).
- fdu-c2ml: YAML contract defects (fixes on the branch; evaluation fdu-4xy9 recommends an owned policy and
  streaming JSON/YAML sink, fdu-fft9).
- fdu-i142, fdu-nwud: registry pages and artifact identity; fdu-obh2: runbook and plan specs (docs PR).
- Decisions: fdu-p7vb exact pins in published fdu-core; Rust API extensibility is a P1 decision bead under this epic.
- Maintainer actions: fdu-atbv repository security settings; fdu-znb0 SSH signing identity.
- fdu-tyvq end-to-end verification, then fdu-9cf0 publication by hand (tag, crates.io, PyPI, GitHub
  Release children), then the global dev install chore.
Not blocking (documented limitations or post-release): memory beads fdu-if7o, fdu-syyl, fdu-6o5o; parity
proof fdu-lj4h and fdu-pro1; release automation epic fdu-zr73. The audits also closed or reprioritized ~60
stale beads (every open P0 is now release work).
