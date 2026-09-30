---
type: is
id: is-01m3qk1f9s3gfa3k26pnw8688q
title: "H188 with H189: byte-wise paths in the summary fold, and a pre-sized control-file read"
kind: task
status: closed
priority: 2
version: 4
spec_path: docs/project/research/research-2026-09-29-uniformly-faster-than-pdu.md
delegate: claude-code@vm
labels: []
dependencies: []
parent_id: is-01m3qgck5yzpd603akhhkpw7t9
hold: null
hold_until: null
created_at: 2026-09-29T22:02:23.929Z
updated_at: 2026-09-30T01:55:06.273Z
started_at: 2026-09-29T23:08:07.945Z
closed_at: 2026-09-30T01:55:06.273Z
close_reason: Accepted in exp-198 (aggregate-summary -6.15% [-7.94%, -1.80%] on linux-v6.12, placebos at zero); H189 rides as the secondary below wall resolution. Shipped in a0666bf0 and 7a3a7058; confirmed end to end in exp-201.
resolution: null
duplicate_of: null
---
The summary route's consumer is 373M instructions on linux-v6.12, 180M of it std::path (compare_components 94M, parent()/file_name() parsing, Path hashing of ignored heads). Predicted aggregate-summary -5% to -9% on linux-v6.12; placebos on node-modules-dense and default-tree. H189: each .gitignore is read in eight read calls (11 syscalls per file, 3,938 per run) because read_to_end behind take cannot see the statx length; 4 calls with a reserve. Secondary default-tree on linux-v6.12.

## Notes

Built a0666bf0 (fold: byte-wise parent/name, OsString heads; H189 pre-sized control read) and 7a3a7058 (ControlTable byte-keyed source index for chain_for/chain_below, found by profiling the first build). Summary consumer 361.7M -> 202.7M instructions on linux-v6.12 (-44%); read 1,261 -> 729; 54 answer comparisons identical (both builds); compact-equals-indexed differential passes. Cell (exp-198) pending: control h185, candidate perf_probe-h188b-7a3a7058.
