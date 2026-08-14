---
type: is
id: is-01m00hb6pbszzm7vcfm3p6ghfp
title: "Allocation is producer-side: the walk allocates at least three times per entry"
kind: task
status: open
priority: 1
version: 1
labels: []
dependencies: []
created_at: 2026-08-14T16:23:08.234Z
updated_at: 2026-08-14T16:23:08.234Z
---
The per-layer counters (exp-052) invert an assumption this campaign has been carrying. scan-producer, which walks without building an index, allocates MORE than scan-index does - 8.8 million against 6.9 million allocations, and 5.45 million against 4.94 million reallocations. The two jobs differ in what they retain, so this is a direction rather than a clean subtraction, but it points away from the consumer and at the walk. Per entry the walk allocates an OsString for the name, a PathBuf for the joined relative path, and a clone of that PathBuf into Op::Upsert - three before the batch vector, and exp-051's profile still showed 34,256 Op::clone calls after the parent memo landed. This is the same cost fdu-2ubt targets with batch-shaped observations, and the counters now make the reduction provable rather than inferred: a batch-shaped observation should drop allocs per entry from about 15 toward about 12 and cut the PathBuf clones to one per directory. Screen fdu-2ubt with counters on and quote allocs-per-entry alongside wall.
