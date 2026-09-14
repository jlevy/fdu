---
type: is
id: is-01m2et6mnsj47svnsfdnybpns6
title: "PR #48 review FIX48-5: a control-bound refusal is retained as ProviderFailure with no path"
kind: bug
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels: []
dependencies: []
parent_id: is-01m2et5s9yyv9e6tz783rp2m6c
created_at: 2026-09-14T01:58:41.593Z
updated_at: 2026-09-14T01:58:41.593Z
---
Low, from c801d4e. At f917cb7: opened.rs:1072 -> engine_contract.rs:480-489 (Issue::from_error catch-all). The refusal is retained as kind ProviderFailure, path None, though a control bound is a resource bound and the directory whose .gitignore tripped it is known at the call site. Fix: retain it with IssueKind::ResourceBudget and that directory's root-relative path. Review: https://github.com/jlevy/fdu/pull/48#pullrequestreview-5193206420
