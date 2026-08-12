---
type: is
id: is-01kzt7d78remd9rawjk1a8scty
title: "PR#6 R9: reaper skips bare filename targets"
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-12T05:34:02.007Z
updated_at: 2026-08-12T05:37:16.451Z
closed_at: 2026-08-12T05:37:16.451Z
close_reason: "Fixed and mutation-checked in faeb7df; disposition posted to PR #6"
---
crates/fdu/src/snapshot.rs:697. For a bare relative target such as snap.fdu, path.parent() is the empty path rather than None, so the unwrap_or fallback to '.' never fires. create_dir_all("") succeeds and the write lands in the cwd, but read_dir("") fails, so reap_stale_temporaries returns immediately and never collects abandoned temporaries for that target. Medium.
