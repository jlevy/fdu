---
type: is
id: is-01m2esgtxk7ed7k7mh039e51m7
title: "Release note: the type_rules_fingerprint change cold-rescans every cached tree on upgrade"
kind: task
status: open
priority: 3
version: 1
spec_path: docs/project/specs/active/plan-2026-08-25-fdu-opened-root-inventory-engine.md
labels:
  - stack-followup
dependencies: []
parent_id: is-01m0xs2ffhy8av1qm0dn9kyc31
created_at: 2026-09-14T01:46:47.090Z
updated_at: 2026-09-14T01:46:47.090Z
---
Release-note follow-up from PR #48's description ("The command line is unmoved"): "One consequence deserves a release note."

**What.** The opened-root rewrite changes the type registry, so `type_rules_fingerprint` moves. It is invisible in human output (one `cli-content.tryscript.md` golden records the new value), but on-disk caches are keyed on it. The first run of the new binary on every previously cached tree finds its snapshot mismatched and cold-scans. That is correct, since the fingerprint exists to move when the rules move, but a user sees a silent slow first run after upgrading, and the old snapshots stay on disk.

**Also check before writing it.** Later PRs in the same stack change what a cached snapshot is: #51 and #52 make one-shot reports controls-off, and #52's description notes the historical v2 snapshot format differs from v3. Confirm on the release candidate exactly which previously written caches are still served, so the note is accurate.

**To do.** When the stack reaches a release, write one note saying:
- the first run on each cached tree after upgrading re-scans cold;
- why;
- how to reclaim the stale snapshots, with whatever cache-clearing mechanism exists at that point (fdu-558j tracks that none prunes them today).

Put it wherever release notes live for that release, and close this bead with a link.
