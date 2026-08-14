---
type: is
id: is-01m00kc8e9pgxx4gskqt8ymxbb
title: Every read_dir costs two getdents64 calls; half are pure end-of-stream detection
kind: task
status: open
priority: 1
version: 1
labels: []
dependencies: []
created_at: 2026-08-14T16:58:39.945Z
updated_at: 2026-08-14T16:58:39.945Z
---
Found by the three-tier cross-check the instrumentation playbook prescribes, and invisible to any single tier. Application counters report 2,559 directory opens for the 17,128-entry content tree. strace on the same job reports 2,565 openat, 17,131 statx - both matching the app counters within noise - and 5,118 getdents64, which is exactly 2.00 per read_dir. The second call in each pair returns zero to signal end of directory: std's ReadDir cannot know the directory is exhausted until the kernel says so, so half of all directory-read syscalls carry no data. At the 29 microseconds per call strace measured, that is roughly 74 milliseconds on this tree, and it scales with directory count rather than entry count - so it hurts most on the wide shallow trees that are common in practice. Worth investigating whether a raw getdents64 loop can skip the confirming call when the returned buffer is shorter than the one supplied, which is the usual trick, and whether that is actually safe across filesystems - a short return does not strictly guarantee exhaustion on every filesystem, and getting this wrong silently truncates a directory listing, so any implementation needs a differential test against the std walker over a tree with directories sized either side of the buffer boundary. Relates to H77 and to the macOS bulk reader, which already bypasses std for the same class of reason. This is also the worked example the playbook's cross-check section should carry: an application counter that is accurate about what the code called, and wrong about what the kernel did.
