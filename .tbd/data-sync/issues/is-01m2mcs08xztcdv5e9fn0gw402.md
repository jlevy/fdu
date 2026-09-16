---
type: is
id: is-01m2mcs08xztcdv5e9fn0gw402
title: "PR #63 delta review PR63D-SNAP-1: record why the control-section layout change keeps FORMAT_VERSION 4"
kind: task
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m2kfr5a9x4sksa32yq00fh8k
created_at: 2026-09-16T05:59:29.819Z
updated_at: 2026-09-16T06:19:25.271Z
closed_at: 2026-09-16T06:19:25.270Z
close_reason: "No code change. The reasoning is recorded in fdu-1onj's notes, the bead that owns the format bump (decision Q11): main is v3 with no 0.1.0 tag, the only old-layout v4 files come from builds of this branch between eed4f62 and 1fd71a9, every path such a file takes through the new reader ends Invalid then Ok(None) then a cold scan, and the note says when to bump to 5 instead."
resolution: null
duplicate_of: null
---
Delta review 5218970886 at 9105768: snapshot.rs:64,181-182,754-761,838-848. The control section changed from a u64 budget to two tagged limits under an unchanged FORMAT_VERSION = 4. No released build wrote a v4 file (main is v3, no tag), and a v4 file from the intermediate 1fd71a9 build fails closed to a cold scan. Record the reasoning in fdu-1onj, which owns the format bump (decision Q11).
