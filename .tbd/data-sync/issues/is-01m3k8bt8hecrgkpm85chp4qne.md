---
type: is
id: is-01m3k8bt8hecrgkpm85chp4qne
title: "Change-source review follow-ups: choose a whole-home disk-growth accelerator"
kind: epic
status: open
priority: 1
version: 18
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies: []
child_order_hints:
  - is-01m3k8bv5dqnr0phk1a24we46n
  - is-01m3k8bw0k83v0jq6zfdbvv5vj
  - is-01m3k8bwjpa8gbrk4ev4wh6m8f
  - is-01m3k8bx2kj5nsfqmwqnw3ye94
  - is-01m3k8bxjktwh06sdqgqtjr1n1
  - is-01m3k8by8ghsmwr4mjxfykq0p8
  - is-01m3k8bys7tcybgmse3kdkjq28
  - is-01m3k8bze1qsyzxf6xnegjkf7p
  - is-01m3k8bzyw79ywfse1dwa77yp0
  - is-01m3k8c0kbjv92m08nyv6nb8th
  - is-01m3kae7agsc4dtrx6jkffxfwx
  - is-01m3kae8naxe9qex9zd7zkb5az
  - is-01m3kck3xyk6wywhb9xqtrt060
  - is-01m3kck4cnw948m8da29cpsdbv
  - is-01m3kck52bs4kdtp3bmaa4n2gd
  - is-01m3kck5jdzmkjbk9zzmb2pfkk
created_at: 2026-09-28T05:38:50.767Z
updated_at: 2026-09-28T06:52:46.028Z
---
Epic for the 2026-09-27 change-source review (research doc: docs/project/research/research-2026-09-27-disk-growth-change-sources.md; evidence explorations/change-sources/). Findings: FSEvents misses open writers at event generation (live and replay); one-shot replay cost ~0.124 s per compressed journal MB behind the cursor plus ~10 us per matching record; flat snapshot load ~ walk cost; delta-only roll-up log diff 0.077 s for 4,618/95.5k changed dirs; APFS dir-stats gencount sees open-writer writes (verification pending). Children are the ranked experiments and the engine fixes the review found. Related: fdu-vhrb (writer coverage), fdu-uwhl (replay acceptance), fdu-vw9r (disk-pressure profile), fdu-8ybz (checkpoint store).

## Notes

2026-09-28: research doc and evidence landed as PR #142 (stack 141, on top of #139); corrections to the spike's own records pushed to #131 (commit 420f9841). Pending in the doc: resident soak (fdu-2o00) and APFS dir-stats verification (fdu-gpqz).
