---
type: is
id: is-01m3msv53jeze9d48vt0kanz87
title: "Test hang: deep_rendering_is_stack_safe can spin for hours (no timeout on its child process)"
kind: bug
status: open
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m3mcwynm1rkdjencnq5621mq
created_at: 2026-09-28T20:03:33.605Z
updated_at: 2026-09-28T22:51:02.356Z
---
2026-09-28, macOS arm64, in a #153 worktree gate (no Rust changes vs gated code): during cargo test --locked -p fdu-core --no-default-features, report_format::tests::deep_rendering_is_stack_safe's re-invoked child (--exact ... --nocapture, DEEP_RENDER_CHILD_ENV) spun at ~170% CPU for 85+ min with ~0.8 MB RSS, ignored SIGTERM (needed SIGKILL), and macOS sample crashed (Bus error) attaching to it; the parent test harness also spun at 84-184% CPU and exited on SIGTERM. The same test passed in several full gates the same day, so it is intermittent. Looks like a fault loop on the 64 KiB small-stack thread (overflow handler faulting?) rather than real work. Fixes: (1) bound the child with a timeout (spawn + poll/wait with deadline, kill and fail with its output after e.g. 120 s) so a hang fails fast instead of blocking make check; (2) investigate the spin (stack size margin on macOS debug with --no-default-features, alt-stack/guard-page behavior) and why the parent harness also spun.

## Notes

2026-09-28: implemented on claude/fdu-alternatives-research-qx0xn0 (PR #155), merged at 2d1eea9d/e0923063. 04d640e3: child bounded by a 300 s deadline with output capture; spin analysis in the commit message (no defect found; fixture build is cubic in depth, follow-up). Pending: independent review and CI, then close.
