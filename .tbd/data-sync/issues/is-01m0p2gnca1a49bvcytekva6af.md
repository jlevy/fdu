---
type: is
id: is-01m0p2gnca1a49bvcytekva6af
title: Publish the corrected peer finding to the top-level performance docs
kind: task
status: closed
priority: 2
version: 3
labels: []
dependencies: []
created_at: 2026-08-23T01:07:15.978Z
updated_at: 2026-08-23T01:16:06.539Z
closed_at: 2026-08-23T01:16:06.539Z
close_reason: null
---
The corrected ripgrep-walker result and the floor result are in the research document and
the README. The docs that route readers to evidence still carry neither:

- the campaign status report, which is the stated starting point, in its evidence-weakness
  section: a generated corpus can invert a peer ranking, not merely understate a cost
- the live tool comparison report, which is the peer-comparison home and is macOS-only
- the performance loop's reference-tree section, which requires a real tree and can now
  say what happened when one was not used

Also republish the artifact, which still carries the uncorrected headline.

## Notes

Done. Landed in the campaign status report (evidence-weakness section), the performance
loop (reference-tree section), the live tool comparison report (a Linux walker-level
companion section), TODO.md (fdu-lk9u beside fdu-ow8y), and the README.

The external artifact was replaced with a pointer to the repository copy rather than
updated, per the instruction that context stays in the repo so every agent reads the
same source.
