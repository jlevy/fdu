---
type: is
id: is-01m77c2q82pt2wjj45jfbhrtsv
title: Machine-readable progress for scripts and agents, without a TTY
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m77c2ng8nt6bt6tr2xmjgbsh
created_at: 2026-11-12T07:55:41.185Z
updated_at: 2026-11-12T07:55:41.185Z
---
#207 §3.1 and §3.3. Engine ProgressSnapshot has phase, dirs, files, bytes, allocated, the analysis pair, and root position; elapsed time and last activity are not in it. --progress always never reaches a pipe (tested). Design per the principles: the record type, schema constant, writer, and the definitions of elapsed and last activity live in fdu-core (clock passed in); the CLI keeps the opt-in flag, cadence, and destination; Python needs a progress handle for reports to emit identical records. An outside contributor (sharmis-tp, issue comment 2026-10-27) offered to implement this and asked where the record shape lives; the maintainer has not answered. Decide the channel (stderr JSON Lines vs a caller-named destination) first.
