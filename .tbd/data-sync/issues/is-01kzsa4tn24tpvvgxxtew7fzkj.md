---
type: is
id: is-01kzsa4tn24tpvvgxxtew7fzkj
title: "PR#6 C3: parallel default is breadth-first-preferred, not breadth-first"
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01kzsa4b2j0b3rmvkhf4r0ktxz
created_at: 2026-08-11T21:02:38.241Z
updated_at: 2026-08-11T21:02:38.241Z
---
crates/fdu/src/scan.rs:454-475,614-649,1562-1593. DirectoryQueue orders only the pending deque, not outstanding claim depths. Ordering test forces threads:Some(1) so it proves only the serial path. High.
