---
type: is
id: is-01m042qv5msnzbg96m0nt1b7mc
title: Narrow the getattrlistbulk attribute set to what the plan consumes
kind: task
status: open
priority: 2
version: 2
labels:
  - performance
  - macos
dependencies: []
created_at: 2026-08-16T01:24:51.506Z
updated_at: 2026-08-16T01:31:46.906Z
---
Candidate hypothesis (H94 when the ledger takes it; H86-H93 are all consumer-side or
I/O-path -- none touch the kernel-side attr request).

Evidence: on /Users/levy/wrk/aisw/trading (494k entries), fdu's transient summary and
dumac perform identical enumeration (one getattrlistbulk sweep per directory, zero
fallbacks) yet fdu spends 22.0 s aggregate sys time where dumac spends 18.3 s (-17%),
with fdu user time already lower (622 ms vs 685 ms). The remaining gap is kernel work
per entry, and the requested attribute set is the obvious mechanism.

macos_bulk requests REQUIRED_COMMON = NAME|DEVID|OBJTYPE|MODTIME|CHGTIME|FLAGS|FILEID
plus sizes. CHGTIME and FILEID exist for the snapshot fingerprint (size/mtime/ctime/
inode trust rule); DEVID for device checks; FLAGS for classification. The compact
transient tier persists nothing and fingerprints nothing: it consumes NAME (recursion),
OBJTYPE, MODTIME (newest-mtime tally), and the size pair. Every extra group is kernel
packing and copyout paid per entry for a value the plan provably discards -- the same
cost rule plan_report now applies to snapshot reads, applied one layer down.

Sketch: thread the requested attr mask through ScanConfig from the plan (full set for
index/cache paths, narrow set for RetainedState::Summary); portable path unaffected.
Unsafe-boundary surgery in macos_bulk, so cross-lint and the differential harness are
mandatory.

Predicted signal, pre-registered: transient-summary aggregate sys time down >=10% on
APFS, wall down 3-8%; cold-scan-index unchanged (keeps the full set). Refutation
criterion: if narrowing moves sys <5%, the dumac gap is elsewhere (buffer size,
open/close pattern, allocation) and this closes.

Needs a quiet host to clear the keep bar (fdu-ow8y); the loop's paired protocol applies.

## Notes

Paired head-to-head confirming the gap this bead predicts (same tree, 10
counterbalanced pairs, bootstrap CI): fdu --cache off --view summary vs dumac came out
fdu +226 ms median (+7.0%), 95% CI [+39, +387] -- entirely above zero, so dumac leads
the scalar class decisively on this host, not within noise. Combined with the -17%
aggregate sys-time gap at identical enumeration counts, the kernel-side attr cost is
the live mechanism. (Work-class caveat unchanged: fdu returns five tallies including
counts and newest-mtime; dumac returns the allocated total only.)
