---
type: is
id: is-01kzsa4t6r912tsts6mt9qhywj
title: "PR#6 C1: snapshot root provenance remains Scanned after load"
kind: bug
status: closed
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-11T21:02:37.783Z
updated_at: 2026-08-11T21:29:59.597Z
closed_at: 2026-08-11T21:29:59.596Z
close_reason: "Fixed and verified on PR #6; disposition posted to the PR"
---
crates/fdu/src/index.rs:1285-1295, crates/fdu/src/snapshot.rs:373-405. apply_upsert root special case never copies applying_source into Entry::source, so load().provenance("") reports Scanned and is_verified()==true. Whole-tree totals live at root, so the main cached answer claims a fresh observation. High.
