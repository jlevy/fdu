---
type: is
id: is-01kzy3ea5bks8mfpj1fabv5tm7
title: macOS bulk reader reports all-fork size where the portable path reports st_size
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
created_at: 2026-08-13T17:41:41.163Z
updated_at: 2026-08-13T17:41:41.163Z
---
scan/macos_bulk.rs requests ATTR_FILE_TOTALSIZE for a non-directory's apparent size. getattrlist(2) documents that as the total across all forks, while the portable reference path stores st_size, the data fork alone. Any file with a resource fork therefore gets a different Attrs::size from the two backends, which breaks the equal-observations contract, the cache fingerprint, and the serial-reference equivalence on macOS. The directory branch already uses ATTR_DIR_DATALENGTH, so this looks like a slip rather than a decision. Fix: request ATTR_FILE_DATALENGTH instead, and swap the parse order, because DATALENGTH (0x200) sorts after ALLOCSIZE (0x4) while TOTALSIZE (0x2) sorts before it. Add a bulk-versus-portable case over a file with a resource fork. Found in review of PR #8; not reproducible on Linux, needs a macOS check.
