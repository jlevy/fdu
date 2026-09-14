---
type: is
id: is-01m2h2vwk53gxk4txd1tpk3xs6
title: "PR #55 review PR55-DOC-3: plan hedges history purge that JournalScoped rustdoc asserts"
kind: bug
status: in_progress
priority: 3
version: 2
spec_path: docs/project/specs/active/plan-2026-09-13-fdu-disk-usage-checkpoints.md
labels: []
dependencies: []
parent_id: is-01m2h2v87crh401e90pfe52w8g
created_at: 2026-09-14T23:08:35.295Z
updated_at: 2026-09-14T23:09:15.823Z
---
PR #55 at dc27c14: FSEvents plan :661-667 and research :1269-1277 hedge 'history silently purged', but origin/main crates/fdu-core/src/engine_contract.rs:263-268 (Source::JournalScoped rustdoc) still asserts it. Docs-only PR: file a stack-followup bead for the rustdoc and list it as a follow-up in the plan.
