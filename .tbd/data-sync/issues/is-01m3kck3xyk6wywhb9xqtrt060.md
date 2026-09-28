---
type: is
id: is-01m3kck3xyk6wywhb9xqtrt060
title: "watch: every macOS rename escalates to a full-root reconcile"
kind: bug
status: closed
priority: 1
version: 8
spec_path: docs/project/research/research-2026-09-27-disk-growth-change-sources.md
labels: []
dependencies:
  - type: blocks
    target: is-01m3kfjgq9x471eftadpm5z1v9
parent_id: is-01m3k8bt8hecrgkpm85chp4qne
created_at: 2026-09-28T06:52:44.338Z
updated_at: 2026-09-28T13:21:20.254Z
closed_at: 2026-09-28T13:21:20.252Z
close_reason: "#145 landed on main 6ec77163 via gh stack merge 148 (stack 141 layers 1-11); open-writer re-stat stays P1 as fdu-vhrb"
resolution: null
duplicate_of: null
---
Found by the 2026-09-27 resident soak (explorations/change-sources/watch-soak/). notify 8.2.0's FSEvents backend reports every ItemRenamed as an unpaired Modify(Name(Any)); crates/fdu-core/src/watch.rs record() (~L816-834) escalates any unpaired rename to InvalidateReason::UnpairedRename on the whole root. FSEvents flags are sticky per path, so later events on the same path escalate too. On agent state B (~476k entries) 61.5% of raw events carried ItemRenamed (atomic temp+rename writes): 174 root reconciles in 61 min (one per 20 s, 9.9 CPU s each), 47.8% of a core (sys-dominated), RSS median 442 MB / peak 906 MB. Windows without a root reconcile cost ~1% of a core. Fix direction: scope a one-sided rename to the named path's parent (recursive when the named path is a directory), relying on FSEvents delivering both sides or a drop flag; keep root escalation for drop/overflow flags. Note the accidental benefit: the polling covered the open-writer gap, so a fix must pair with the libproc writer re-stat (fdu-vhrb).

## Notes

2026-09-28: CI on #145 failed on all three OSes at the admission-sites audit: ParentListings' read_dir in watch.rs was unclassified. Classified as a non-inventory reader (a8edd42f) and merged up through #146/#147/#148. The Test job stopped at that step, so the rest of the suite had not run on CI for #145+ until this fix.
