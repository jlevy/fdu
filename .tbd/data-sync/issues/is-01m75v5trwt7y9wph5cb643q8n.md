---
type: is
id: is-01m75v5trwt7y9wph5cb643q8n
title: "PR #209 I6: documentation: --docs and Python README mount-table rule, scan::scan rustdoc, host gate setup"
kind: bug
status: closed
priority: 3
version: 3
spec_path: docs/project/specs/active/plan-2026-11-08-fdu-recent-growth-and-volumes.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m75v586ca3vv6vremva5h5er
hold: null
hold_until: null
created_at: 2026-11-11T17:41:02.876Z
updated_at: 2026-11-11T18:08:53.051Z
started_at: 2026-11-11T17:41:27.952Z
closed_at: 2026-11-11T18:08:53.050Z
close_reason: "Fixed in 40f7adee: --docs and the Python README state the mount-table rule; scan::scan rustdoc names the root record; gate guide Setup names cargo-semver-checks and Python 3.11+, step 1 runs from the checkout. cli-surface golden updated (make test-golden 252 passed); make parity-update diff is only that sentence."
resolution: null
duplicate_of: null
---
Low. crates/fdu/src/cli.rs:346-347 and crates/fdu-py/README.md:339-340 give the pre-gate rule; crates/fdu-core/src/scan.rs:1821 does not say the stream's first record is the root carrying its device; docs/project/guides/linux-host-gate.md setup does not name cargo-semver-checks or Python 3.11+ and step 1 does not say to run from the checkout. Golden cli-surface and parity move with --docs. Review: https://github.com/jlevy/fdu/pull/209#issuecomment-6802796047 (PR #209).
