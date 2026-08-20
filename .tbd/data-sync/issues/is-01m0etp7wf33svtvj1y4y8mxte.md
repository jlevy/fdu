---
type: is
id: is-01m0etp7wf33svtvj1y4y8mxte
title: Summary and tree directory counts ignore the selection filters
kind: bug
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01m0erhq35tpxzjecxn3p9jzx2
created_at: 2026-08-20T05:35:49.134Z
updated_at: 2026-08-20T06:21:09.542Z
closed_at: 2026-08-20T06:21:09.540Z
close_reason: "Fixed in 5647d4b: the dirs tally moved into the pre-order pass beside the file tally, gated by the same selection.admits call, and the post-order fold now only adds what each child subtree found. Summary, tree, and files views agree under every selection; unfiltered reports are untouched because they read the pre-computed roll-up and never call walk. Four tests, two of which fail against the old fold, plus two goldens."
---
Found during the end-to-end round on macOS. Reproducible on the committed fixture tests/golden/fixtures/project (6 files, 3 directories).

The same query gives two different directory counts depending on which view answers it:

  fdu --cache off --view files --kind dir --include '*.rs' project   -> lists nothing
  fdu --cache off --view summary --include '*.rs' project            -> '8.0 KiB  2 files, 3 directories'

The clearest case needs no glob at all:

  fdu --cache off --view summary --kind file project                 -> '24 KiB  6 files, 3 directories'

--kind file says report only files, and the summary still reports 3 directories. Likewise --min-size 1G reports '0 files, 3 directories' when nothing in the tree is a gigabyte.

Cause is in walk() in crates/fdu/src/query/query_report.rs, in the post-order fold (around line 493):

    if let Some(sub) = walked.per_directory.get(&child) {
        merge_summary(&mut total, &sub);
        if index.kind_of(child) == Some(EntryKind::Dir) {
            total.dirs += 1;
        }
    }

files, bytes, allocated and the files-view rows are all gated on selection.admits(&candidate) in the pre-order pass, but this dirs increment consults only the entry kind. Every directory is given a per_directory entry unconditionally, so every directory in the retained tree is counted whether the selection admits it or not.

Scope: SummaryRow.dirs reaches text and all three machine formats, and TreeNode.dirs takes the same value, so tree nodes carry it too. A scripted consumer gets a directory count that does not correspond to the filter it asked for.

No test or golden currently pins dirs under a filter, so either reading could be made the intended one. Decide which: filter directories like every other entry, or keep dirs as a deliberate structural count and say so in --help, the README, and a named test.
