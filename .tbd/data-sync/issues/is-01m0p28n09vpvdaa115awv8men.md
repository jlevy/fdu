---
type: is
id: is-01m0p28n09vpvdaa115awv8men
title: "Correct the ripgrep-walker comparison: the lead is a generated-corpus artifact"
kind: bug
status: closed
priority: 1
version: 6
spec_path: docs/project/reports/report-2026-08-23-metadata-walk-floor.md
assignee: claude-code@vm
labels: []
dependencies:
  - type: blocks
    target: is-01m0p2gnca1a49bvcytekva6af
created_at: 2026-08-23T01:02:53.449Z
updated_at: 2026-08-23T01:16:14.788Z
closed_at: 2026-08-23T01:16:14.788Z
close_reason: null
---
The metadata-walk physics research reported fdu's aggregate tier as "22% faster than
ripgrep's walker (the ignore crate), and that result held in every sitting". A paired,
interleaved re-measurement across six subjects shows the claim is a property of uniform
generated trees, not of the workload:

  tree     420k synthetic, 21 entries/dir   fdu -26.2% (faster)
  wide     402k synthetic, 201 entries/dir  fdu -16.2%
  narrow   400k synthetic, 5 entries/dir    fdu -12.6%
  usrshape  86k synthetic, generated names  fdu -21.3%
  usrnolnk  84k, /usr's real names          fdu  +1.5%  (tie)
  /usr      85k real tree                   fdu +11.8% (SLOWER)

13 paired trials after 3 warmups, alternating order, median of the paired differences.

The correction is self-inflicted in an instructive way: the same document reports that a
real tree's names and width distribution cost fdu about 15 points of its distance from
the floor while costing ignore nothing. That effect is exactly large enough to consume
the lead, and the /usr row contradicting the headline was already printed in the
document's own matched-control table. The claim was read off the primary synthetic
subject and generalized without checking it against the real-tree row on the same page.

Work:
- Correct the research document's Part 3 and its overview claim.
- Correct the published artifact.
- Add the honest version to the README and the campaign status report, in the register
  the README already uses for Linux scouting evidence.
- Fold the six-subject table into the research document as the evidence.

The underlying mechanism finding is unaffected and still holds: ignore and walkdir stat
by absolute path, fdu stats dirfd-relative, and isolating that costs +37% wall. What
changes is that fdu spends the advantage elsewhere on a real tree.

## Notes

Done. The corrected six-subject sweep, the mechanism that explains both directions, and
the record of how the mistake happened are in the report; the honest version is in the
README and the three routing docs. fdu-lk9u carries what a real ordering would require.

Kept open only if the artifact pointer needs revisiting; otherwise closed with fdu-tfhb.
