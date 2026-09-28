---
type: is
id: is-01m3kw451a9tawhpbycn65bktj
title: "watch: gate the moved-directory relist on a changed fingerprint"
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-09-28T11:24:11.177Z
updated_at: 2026-09-28T11:24:11.177Z
---
fdu-822y review finding 3: a sticky FSEvents ItemRenamed flag on a directory path turns every later event naming it (chmod/touch/xattr/Finder) into a full subtree relist (watch.rs:1117-1126). Relist only when the upsert changes the directory's fingerprint (Attrs carries inode/dev/ctime; a rename changes ctime; a matching fingerprint is already a no-op).
