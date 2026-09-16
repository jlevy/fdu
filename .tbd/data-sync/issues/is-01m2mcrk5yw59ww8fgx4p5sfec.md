---
type: is
id: is-01m2mcrk5yw59ww8fgx4p5sfec
title: "PR #63 delta review PR63D-DOC-1: CHANGELOG says a mismatched control table is refused at load, but load treats it as absent"
kind: bug
status: closed
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m2kfr5a9x4sksa32yq00fh8k
created_at: 2026-09-16T05:59:16.413Z
updated_at: 2026-09-16T06:19:14.192Z
closed_at: 2026-09-16T06:19:14.191Z
close_reason: "d683dc4: CHANGELOG.md now names both paths - save refuses with Error::ControlLimitsOutsideScope, and a snapshot carrying such a pair is treated as absent at load, like any other unreadable one, so the root scans cold."
resolution: null
duplicate_of: null
---
Delta review 5218970886 at 9105768: CHANGELOG.md:166-167 says 'saving or loading an index whose table and scope disagree is refused with Error::ControlLimitsOutsideScope'. At load it is corrupt-equals-absent: parse_stream maps the error to ParseError::Invalid (snapshot.rs:631) and load returns Ok(None) (snapshot.rs:380), so the scan goes cold. Say what happens on each path.
