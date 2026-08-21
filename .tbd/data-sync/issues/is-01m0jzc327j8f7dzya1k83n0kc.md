---
type: is
id: is-01m0jzc327j8f7dzya1k83n0kc
title: Paid-for-nothing note says "read 0 B" when served warm from the sidecar
kind: bug
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-08-21T20:14:37.126Z
updated_at: 2026-08-21T20:14:37.126Z
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
