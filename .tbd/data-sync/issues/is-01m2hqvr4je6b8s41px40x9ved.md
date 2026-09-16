---
type: is
id: is-01m2hqvr4je6b8s41px40x9ved
title: Re-measure the README headline performance figure on the 0.1.0 release candidate
kind: task
status: closed
priority: 1
version: 3
labels:
  - release
dependencies: []
created_at: 2026-09-15T05:15:30.833Z
updated_at: 2026-09-16T10:10:44.183Z
closed_at: 2026-09-16T10:10:44.182Z
close_reason: "Re-measured on the 0.1.0 release candidate (fdu 0.1.0-dev+g16efcd0a, release build of main at 16efcd0). The 901,963-entry subject no longer exists and is not reproducible by design, so the run used the committed balanced recipe at its 1,000,000 scale point: 1,000,001 entries, seed fdu-balanced-v1, digest 4bbd97c0d3d4e2ad. Indexed tree 5.206 s median, fastest of dumac 5.637, diskus 6.972, dust 8.292, dua 8.744, BSD du 51.226, GNU du 65.775; 12 paired trials each, 0 invalid samples, 0 semantic or oracle mismatches, no drift. Host regime uncontrolled at load 7.7-9.9 on 10 cores, stated in the figure. README headline and speed section rewritten with the date, the revision and both caveats; full method in docs/project/reports/report-2026-09-16-fdu-live-tool-comparison.md. pdu and Go gdu are not installed here and could not be re-measured; fdu-hkyh filed for the transient-summary contract."
resolution: null
duplicate_of: null
---
DECISION (user, 2026-09-15): re-measure the README headline figure ('3.324 s over 901,963 entries') on the release candidate before tagging 0.1.0. There is no timing sample on the merged engine, and fdu-pro1/fdu-lj4h are open. Use the performance loop's protocol on the same subject, record the regime and binary identity, and update or keep the figure with its date and revision. On a noisy host, state the caveat. Parent: fdu-gjc2.
