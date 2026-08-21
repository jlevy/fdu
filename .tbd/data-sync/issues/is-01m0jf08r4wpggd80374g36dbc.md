---
type: is
id: is-01m0jf08r4wpggd80374g36dbc
title: "H94: Content roll-up ancestor walk re-descends a path-ordered tree per file"
kind: task
status: closed
priority: 1
version: 2
labels: []
dependencies: []
created_at: 2026-08-21T15:28:32.516Z
updated_at: 2026-08-21T16:05:00.632Z
---
**Tier: content** (index plus content records). Not evidence about the aggregate or index tiers.

`ContentIndex::merge_ancestors` is 43.73% of a warm content cache-hit open, measured
inclusive by callgrind on a 15,977-file tree (see the corrected profile on `fdu-926e`).

```rust
fn merge_ancestors(&mut self, file: &Path, analysis: &FileAnalysis, add: bool) {
    let mut directory = file.parent();
    while let Some(path) = directory {
        if add {
            self.rollups.entry(path.to_path_buf()).or_default().add(analysis);
        } else if let Some(rollup) = self.rollups.get_mut(path) { ... }
        directory = path.parent();
    }
}
```

`rollups` is a `BTreeMap<PathBuf, ContentRollUp>`. Two independent defects on the add path:

1. Every ancestor of every file costs an independent path-ordered tree descent:
   O(depth x log n) `compare_components` calls, each walking the path components again.
   Measured at 2,028,997 `compare_components` calls for 15,977 files.
2. `entry(path.to_path_buf())` allocates a `PathBuf` per ancestor per file even when the
   key is already present, which is the overwhelmingly common case -- roughly 128k
   allocations that are immediately dropped.

**The ordering of `rollups` is unobservable.** It is reached only through
`ContentIndex::rollup(&Path) -> Option<&ContentRollUp>`, i.e. point lookup; nothing in
the crate or in fdu-py iterates it. That is the same argument that justified the
candidates `HashMap` in `load_content_cache`, and it is what makes the minimal form of
this change semantics-preserving rather than a redesign.

Minimal screened form: `rollups` becomes a `HashMap<PathBuf, ContentRollUp>`, and the
add path does `get_mut` before `insert` so the common hit allocates nothing. One hash of
O(path len) replaces about log2(n) comparisons of O(path len) each.

**Predicted signal:** `content-cache-hit` component and wall down at least 3% with both
95% intervals below zero; peak RSS no worse; the content digest byte-identical to the
seeding run's. Expect well above 3% if the mechanism is real -- the edge is 36% of
instructions -- so a result near the bar is evidence the mechanism is not what is
being measured.

**What it does not do:** it leaves `files: BTreeMap<PathBuf, FileAnalysis>` alone, whose
`remove` is a further 11.09% through `apply_analysis`, and it does not touch the
per-ancestor walk itself. The structural version -- key roll-ups by `EntryId` and defer
to one bottom-up pass, the shape that won -51.9% on snapshot load in `fdu-91ts` -- stays
open if this clears but leaves headroom.

## Notes

**Confirmed and landed** (commit `8fe6248`, PR #38). Content tier, Linux/virtualized, warm regime.

24 adjacent interleaved pairs, `content-cache-hit`, 15,977-file / 1,045-directory tree:

| Metric | Base | H94 | Change | 95% CI |
| --- | --- | --- | --- | --- |
| wall | 464.4 ms | 346.9 ms | -25.42% | [-26.51%, -24.46%] |
| component | 416.8 ms | 304.9 ms | -27.13% | [-29.00%, -26.24%] |
| total CPU | 0.46 s | 0.35 s | | |
| peak RSS | 42.3 MB | 42.5 MB | +0.5% | neutral |

Mechanism confirmed rather than assumed: instructions 3,462,200,305 -> 2,266,646,925
(-34.53%), `merge_ancestors` inclusive 1,513,915,190 -> 319,007,101 Ir, i.e. 43.73% ->
14.07% of profile and -78.9% absolute. That is the predicted mechanism and not a
coincidental win elsewhere.

Oracles: `engine_digest` and content digest unchanged; the multi-view `--depth 12` JSON
render is byte-identical between arms at 227,198 bytes apart from the two timestamp
fields, which verifies intermediate directory roll-ups rather than only the root total
the probe digest covers.

Headroom deliberately left: `files: BTreeMap<PathBuf, FileAnalysis>` is untouched and its
`remove` was a further 11.09% through `apply_analysis`; and the per-ancestor walk itself
remains. The structural version -- key roll-ups by `EntryId` and defer to one bottom-up
pass, the shape that won -51.9% on snapshot load in `fdu-91ts` -- is still open and is now
the larger of the two remaining items on this path.
