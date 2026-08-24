---
type: is
id: is-01m0rk5bvxy8xcbcz6fwarmj4d
title: Content sidecar path validation uses is_absolute, which is not the guard on Windows
kind: bug
status: open
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-08-24T00:36:40.444Z
updated_at: 2026-08-24T15:22:37.294Z
---
content_cache.rs rejects an absolute relative_path when parsing a sidecar:

    if relative_path.is_absolute() { return None; }

That is the wrong question on Windows. `/escape.txt` is rooted but carries no drive
prefix, so is_absolute answers false and the path is accepted; `..` slips past on every
platform. A sidecar is untrusted input read from disk, so the guard is the whole
protection.

FIX: use index::path_is_representable, which asks about components instead --
Component::Normal only, rejecting ParentDir, RootDir and Prefix. It was made pub(crate)
when the same bug was fixed in the scripted-events path guard (that one was caught by a
Windows CI failure; this one has no test pointing at it).

Found while fixing the scripted-events guard on PR #47. Not fixed there because it is
outside that PR's subject and its own test needs writing: a sidecar fixture carrying a
rooted path, asserting the parse is refused.

## Notes

DECIDED 2026-08-24: not folded into PR #47. Unrelated to the contract subject, and the session's designated branch is pinned to that PR. Fix as its own small change with its own sidecar fixture (a fixture carrying a rooted path, asserting the parse is refused) on a fresh branch when one is authorized.
