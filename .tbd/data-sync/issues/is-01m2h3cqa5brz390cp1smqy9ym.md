---
type: is
id: is-01m2h3cqa5brz390cp1smqy9ym
title: "PR #56 review PR56B-FS-1: a control file gone at its stat keeps its rules when the listing also had an iterator error"
kind: bug
status: closed
priority: 3
version: 3
labels:
  - stack-followup
dependencies: []
parent_id: is-01m2h3ches22p7k9x4kka6bq6q
created_at: 2026-09-14T23:17:46.947Z
updated_at: 2026-09-14T23:53:41.538Z
closed_at: 2026-09-14T23:53:41.533Z
close_reason: "308f945: a stat NotFound now removes a vanished control file's rules beside its entry, at the serial, wave and revalidate sites via vanished_child_removals. The retained case was already covered by the index's control projection (the Op::Remove arm drops a control file's rules), so the finding as written is a false positive there. The real residue was a hidden-pruned .gitignore, which has no entry. Tests cover both policies with a failing listing injected through the new WalkHookPoint::ListingEnd."
resolution: null
duplicate_of: null
---
Delta review 5203772881 on PR #56. Sites at 8d2eb7f: crates/fdu-core/src/scan.rs:4441-4455 and :4480-4493 (serial reconcile), :4865-4877 and :4902-4915 (wave), :3812-3826 and :3901-3917 (revalidate). The Ok(None) arm removes the entry regardless of listing completeness, but the ControlRemove is pushed only under if listing_complete, and only Op::ControlRemove clears the control table (index.rs:1446). Fix: a stat NotFound is positive evidence for the control file too, so push its ControlRemove beside the entry Remove on the same baseline, shared across the three sites; test with an injected listing error.
