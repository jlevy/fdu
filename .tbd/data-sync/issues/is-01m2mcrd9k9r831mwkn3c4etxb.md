---
type: is
id: is-01m2mcrd9k9r831mwkn3c4etxb
title: "PR #67 delta D67-4: a directory row reports its own st_size as bytes and sums it into the unrecognized total"
kind: bug
status: open
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m2mckkzdt9pgwx782pcr49r5
created_at: 2026-09-16T05:59:10.385Z
updated_at: 2026-09-16T05:59:13.531Z
---
Delta review 5218953084 of PR #67, P3.

`crates/fdu-core/src/cache.rs:453-458@a5e2124`; the text and JSON rows in
`report_format.rs`.

`list_caches` describes a non-regular entry from `symlink_metadata` and reports
`metadata.len()` as `bytes`. For a directory that is the directory entry's own size: 4096
on ext4, variable on APFS, 0 on Windows. It feeds "1 unrecognized file (4096 bytes) is not
an fdu snapshot" and the JSON `bytes` field, where a reader takes it for a 4 KiB file.
Platform-dependent, so it is also the portability hazard if a golden plants a directory.

Fix: report a directory without a byte count, or exclude it from the sum. Add a golden
that plants a directory in the cache directory; none does today.
