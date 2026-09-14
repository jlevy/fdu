---
type: is
id: is-01m2esgtj6nb0g30608wd3bjna
title: "Close PR #44 as superseded by #48 (needs the user)"
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-09-14T01:46:46.725Z
updated_at: 2026-09-14T01:46:46.725Z
---
Close PR #44 (https://github.com/jlevy/fdu/pull/44, "docs(specs): the interactive-client contract, from a measured metabrowser deep-dive", branch claude/metabrowser-fdu-integration-7nqx8b) as superseded by PR #48. As of 2026-09-13 it is still open.

**Why.** fdu-7pcz (closed) reconciled #44 into #48's active opened-root plan at c8acfee:
- its measured 120,001-entry comparison, source checks, two-level extension correction, no-callback rationale, and requirement inventory are preserved in the PR #47 review and the plan's artifact disposition map;
- fdu-u7vo was repointed from #44's unmerged spec to the active plan;
- the decision was not to merge or cherry-pick #44.

fdu-7pcz's note: "PR #44 may close as superseded after the reconciliation commit is visible, but closure is not performed without explicit user direction."

**Needs the user.** An agent attempted this during the 2026-09 stack stabilization and the permission system blocked it. Closing a PR is a public, user-owned action.

**To do (user).** Close #44 with a short comment. Point at #48 and at the plan's artifact disposition map in `docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md`, so a later reader does not take the closure for a dropped design, as #47's close note did. Then close this bead.
