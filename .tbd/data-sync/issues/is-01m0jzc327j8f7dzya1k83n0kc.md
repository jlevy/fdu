---
type: is
id: is-01m0jzc327j8f7dzya1k83n0kc
title: Paid-for-nothing note says "read 0 B" when served warm from the sidecar
kind: bug
status: closed
priority: 3
version: 3
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-08-21T20:14:37.126Z
updated_at: 2026-09-30T03:48:50.488Z
closed_at: 2026-09-30T03:48:50.488Z
close_reason: "Not reproducible at b1376507: the paid-for-nothing note that quantified fresh bytes ('--analyze ... read 0 B; no selected view displays content metrics') was replaced in f062360d by 'note: requested analysis is not displayed by the selected views' plus a tip naming the views that show it, with no byte count, so a warm run from the sidecar prints the same note as a cold one; the perf footer separately says 'content read 0 B; analysis 0 fresh, N cached (M B)'. Verified with the release binary on a two-file fixture, cold and warm, with --analyze lines --view tree. No code or golden change."
resolution: null
duplicate_of: null
---
The paid-for-nothing note quantifies fresh bytes read, which is 0 whenever the content
sidecar already held every record:

  cold: note: --analyze lines,code,words read 135 B; no selected view displays content metrics
  warm: note: --analyze lines,code,words read 0 B;   no selected view displays content metrics

Both are accurate -- a warm run really did read nothing -- but "read 0 B" undersells the
note's own point, and a reader could reasonably conclude the analysis did not happen.

The run still restored records, still spent the sidecar load, and still displayed none of
it. The note should say what actually happened on the warm path, e.g. "restored N files
from cache" rather than "read 0 B".

Cosmetic; the invariant it reports is correct either way. Found while smoke-testing the
install from PR #37.
