---
type: is
id: is-01m3tr05r13ydjt9hza3nksarb
title: Run the stability pass's gates stage through the driver for real before a release relies on it
kind: task
status: closed
priority: 2
version: 6
labels: []
dependencies: []
child_order_hints:
  - is-01m4fkm0d134gx8fy5mq1a3b4j
  - is-01m4fkm0tgjsrp8gkypjrr4hcg
created_at: 2026-10-01T03:26:47.551Z
updated_at: 2026-10-09T07:09:02.271Z
closed_at: 2026-10-09T07:09:02.257Z
close_reason: "Met 2026-10-09: make release-stability ARGS='--target-dir … --min-free-gb 2 --label …' with FDU_QA_* exported ran all 15 steps on the 0.4.0 release tree 2c728b23 (f405067d / c041ed1c), every step passed, including make check, cross-lint (5 targets), semver-check and release-rehearse through the driver; the two driver bugs it first exposed are fixed in #189 (fdu-44cd, fdu-ztwr). Record: /Users/levy/fdu-release/0.4.0-pass2/stability/report-2026-10-09-release-0.4.0-stability-pass.md"
resolution: null
duplicate_of: null
---
PR #170's driver (scripts/release/stability_pass.py) has run make release-rehearse, the candidate build, the correctness passes, both breaks, the terminal tests and the peer self-test for real (macOS arm64, 2026-09-30, at 9e53c996), and the QA stage on Linux before the review fixes. make check, make cross-lint and make semver-check have run only against the tests' fake host: the macOS host lacked the reviewed cargo-semver-checks 0.50.0 and three of the five CROSS_TARGETS. Before the 0.3.1 stability pass, run 'make release-stability ARGS="--only gates,candidate"' on a host with both, and the harness, real-tree peer run and pty probe with the review fixes in. Review: https://github.com/jlevy/fdu/pull/170#issuecomment-5917248148 (suggestion 4).

## Notes

2026-10-09: first real run through the driver (0.4.0 pass at 2a643cfd, /Users/levy/fdu-release/0.4.0-pass) found two bugs: (1) ARGS given to make release-stability leaks via MAKEFLAGS into the gates' sub-makes (make semver-check: unrecognized arguments); (2) test_stability_pass assumes FDU_QA_LARGE unset, so exporting the documented FDU_QA_* fails release-test inside make check and release-rehearse. Fixes requested on #189. Product gates otherwise passed: all Rust tests, goldens, parity; QA harness 47/47, peers 26/26, correctness 26/26/72, both breaks caught. pty-probe needs a tree whose scan exceeds 1 s (kernel tree 0.37 s).

2026-10-09: both fixed on PR #189 in f405067d (CI run 37891213321 green; make release-test 387 OK). (1) fdu-44cd: every step now runs without MAKEFLAGS, MFLAGS, GNUMAKEFLAGS, MAKELEVEL, MAKEOVERRIDES, MAKEFILES, MAKE_TERMOUT, MAKE_TERMERR, ARGS (stability_pass.step_environment, in Context.env, the harness env, Host.execute, Host.capture); shown with real GNU make 3.81: an inner make saw ARGS through os.environ and none through step_environment. (2) fdu-ztwr: Scratch sets aside FDU_QA_*, COMMIT, RELEASE, FDU, FDU_BIN, UV_PYTHON, FDU_TEST_ALLOW_NO_NATIVE_WATCH and make's variables; AmbientEnvironmentTests runs the driver tests under them; the whole release suite passes with them exported. Still open: the gates have not yet run through the driver after the fix. Close this once `make release-stability ARGS="--only gates ..."` (the documented invocation, with FDU_QA_* exported) passes check, cross-lint, semver-check and release-rehearse on a host with the reviewed cargo-semver-checks and all CROSS_TARGETS.
