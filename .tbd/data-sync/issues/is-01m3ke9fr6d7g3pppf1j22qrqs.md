---
type: is
id: is-01m3ke9fr6d7g3pppf1j22qrqs
title: Test the background index release above BACKGROUND_RELEASE_MIN_ENTRIES
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-09-28T07:22:25.924Z
updated_at: 2026-09-28T07:22:25.924Z
---
REG-3 from the stack 141 regression review (https://github.com/jlevy/fdu/pull/139#issuecomment-5865327879). git grep BACKGROUND_RELEASE_MIN_ENTRIES hits only the constant (lib.rs:294) and its use; the only test (lib.rs:1148-1166) uses a one-file index, so the detached-thread release path and the Windows inline decision are exercised by no CI job. Make the threshold test-overridable (pub(crate)) or build a ~66k-entry temp tree, and assert complete answers under auto, --cache on (writer holds the last ref), --stale-ok, and cfg!(windows) inline.
