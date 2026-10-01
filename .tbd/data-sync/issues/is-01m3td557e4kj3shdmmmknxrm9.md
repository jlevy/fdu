---
type: is
id: is-01m3td557e4kj3shdmmmknxrm9
title: "PR #170 review R11: three docs gaps"
kind: bug
status: closed
priority: 3
version: 3
delegate: claude-code@spud10
labels: []
dependencies: []
parent_id: is-01m3td4m86df8f2h9kqsqrhdm7
hold: null
hold_until: null
created_at: 2026-10-01T00:17:16.517Z
updated_at: 2026-10-01T03:26:33.825Z
started_at: 2026-10-01T00:17:36.436Z
closed_at: 2026-10-01T03:26:33.823Z
close_reason: "Fixed in a2e5d541, ffa10315 and 9e53c996: probe fallback, exit 3's meaning, --label. PR #170, disposition map https://github.com/jlevy/fdu/pull/170#issuecomment-5924110534"
resolution: null
duplicate_of: null
---
tests/qa/cli-installed-e2e.qa.md:131-132 (FDU_QA_PROGRESS_TREE default exists only under make release-stability); docs/project/guides/release-process.md:63 (what exit 3 means for the release); :405-408 (default labels keep paths under home; --label unmentioned). Review: https://github.com/jlevy/fdu/pull/170#issuecomment-5917248148 (head eb359c49).
