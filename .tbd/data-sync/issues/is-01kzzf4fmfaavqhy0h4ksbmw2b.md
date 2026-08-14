---
type: is
id: is-01kzzf4fmfaavqhy0h4ksbmw2b
title: "PR #21 R2: pin uv recovery guidance"
kind: bug
status: closed
priority: 1
version: 3
labels: []
dependencies: []
parent_id: is-01kzzedh4wjer6vyq7b0yj782d
created_at: 2026-08-14T06:25:16.430Z
updated_at: 2026-08-14T06:34:52.522Z
closed_at: 2026-08-14T06:34:52.520Z
close_reason: "Fixed: recovery guidance now pins the reviewed minimum release and documentation explicitly rejects installing an unreviewed latest release. make check passed."
---
Review R2 from https://github.com/jlevy/fdu/pull/21#issuecomment-5290202229. Makefile:98 and AGENTS.md:55 recommend bare uv self update, which installs latest contrary to the repository's 14-day and exact-pin supply-chain policy. Guide users to the reviewed exact minimum.
