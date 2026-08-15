---
type: is
id: is-01m03b8f0qwm5yp2kv0cv0t0nn
title: Stop persisting a snapshot where it will not be read profitably
kind: task
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-15-fdu-cache-layers-and-defaults.md
labels:
  - performance
  - macos
  - cli
dependencies: []
parent_id: is-01m03bjey08898z8t9a2vhakm1
created_at: 2026-08-15T18:34:30.295Z
updated_at: 2026-08-15T22:47:08.785Z
---
Designed but deliberately unimplemented pending macOS measurement.

Measured on Linux/ext4 over /usr (84,539 entries), 9 interleaved paired trials, warm OS cache:

- fdu --cache off --view summary: 71 ms (0.84 us/entry)
- fdu --cache off --view tree:   132 ms (+93%)  -- index construction, nothing persisted
- fdu --cache auto --view tree cold: 230 ms (+222%) -- adds the snapshot write
- fdu --cache auto --view tree warm: 162 ms (+126%) -- still loses to the transient index
- fdu --cache only --view summary: 81 ms (+18%) -- deserialisation costs about a warm walk

Decomposition: index construction +61 ms, snapshot write +82 ms. Warm 'auto' performs 84,539 metadata stats, i.e. revalidation walks the whole tree regardless, so for a metadata query the snapshot avoids no filesystem work and is purely additive.

Where the snapshot does pay, both confirmed on the same host:
- content analysis (--analyze code, warm): 639 ms without cache vs 325 ms with, -49%
- metadata with a COLD OS cache via --cache only: 277 ms scan vs 118 ms snapshot read, -57%

Rule the evidence supports: persist when the retained state costs more to recompute than to load and revalidate. Analysis always qualifies. Metadata qualifies only when the tree is large enough that a future cold read saves meaningful time.

Proposed change: on the cold-scan path in open_with_pending_save, replace SaveTargets::all() with a gate that persists when analysis was requested, or the policy is Refresh (explicit), or the index entry count is at or above a threshold. Gate on ENTRY COUNT, not wall time: entry count is a deterministic property of the tree, so behaviour stays reproducible and paired benchmarking is not made ambiguous.

NOT implemented here on purpose. The only defensible threshold from this data (~250k entries, roughly a one-second cold walk on the measured host) is derived solely from Linux/ext4, and shipping it would rewrite 8+ golden blocks in cli-lifecycle.tryscript.md that encode the documented cache lifecycle. Per the project's own rule, a constant tuned in one regime is inherited rather than proven in another, so the threshold needs Apple Silicon/APFS measurement before it becomes a default.

Acceptance: reproduce the decomposition above on macOS/APFS; choose the threshold from that data; implement the gate; update the affected golden contracts deliberately rather than by regeneration; document the constant and its measured provenance in platform-tuning.md; confirm --cache-status behaviour after a default run is still intuitive.

## Notes

macOS/APFS measurement done (2026-08-15). It argues AGAINST the threshold rather than
supplying one, so the acceptance criteria are answered but the proposed change should not
ship.

Subject /Users/levy/.rustup, 175,128 entries, warm, 9 interleaved paired trials, both
binaries in each trial. Host UNCONTROLLED (load average ~25, desktop apps running), so
absolute values are exploratory per the Makefile evidence qualifiers; the ratios between
interleaved arms are the load-bearing quantities.

  transient summary (--cache off --view summary)  521 ms   2.97 us/entry
  no-scan read      (--cache only --view summary) 146 ms   0.83 us/entry
  snapshot write    (auto vs off, --view tree)     90 ms   0.51 us/entry

The inversion vs ext4: deserialisation costs about the same on both filesystems (0.83
here vs 0.96 there), but an APFS metadata walk costs ~3.5x an ext4 one (2.97 vs 0.84).
So the comparison that came out +18% AGAINST the snapshot on ext4 comes out >3x FOR it
here. The write is repaid ~4x by a single later --cache only read, at any tree size.

That removes the premise of the proposed gate ("metadata qualifies only when the tree is
large enough that a future COLD read saves time"): on APFS a WARM read already saves
2.14 us/entry. A size threshold would give up real value on exactly the trees it gated.

Verdict: SNAPSHOT_MIN_ENTRIES stays None as a measured decision, not a deferred one. No
golden contracts need rewriting. Recorded in platform-tuning.md and the constant's doc
comment.

Residual: the controlled-cold APFS regime is still unmeasured (fdu-rjqx; macOS purge is
diagnostic-only per performance-loop.md). It would have to overturn a >3x warm margin to
revive the threshold, so it is not blocking.

Mechanism re-confirmed on APFS via FDU_COUNTERS: warm revalidation performs 175,129
metadata stats (one per entry) while --cache only performs 0, matching the Linux finding
that a snapshot avoids no filesystem work for a metadata query.
