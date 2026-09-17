---
type: is
id: is-01m2phzcjkthhqf430p6mhdmad
title: "Packaged artifacts identify themselves correctly: version stamp and LF text"
kind: bug
status: open
priority: 1
version: 2
labels:
  - release
  - packaging
dependencies:
  - type: blocks
    target: is-01m2phzegm4b3scda7d1xq3gnm
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-09-17T02:08:50.770Z
updated_at: 2026-09-17T02:08:52.755Z
---
- crates/fdu/build.rs and crates/fdu-core/build.rs append `-dev+g<sha>` whenever any enclosing Git
  repository exists, so a crate installed from crates.io inside an unrelated repository reports a
  stranger's commit. Fix on claude/release-e2e-fixes: skip Git when `.cargo_vcs_info.json` is present.
- The Windows wheel ships CRLF in `.py`, `.pyi`, `py.typed`, LICENSE and the METADATA README text;
  every other wheel is LF. Fix on claude/release-e2e-fixes: `.gitattributes` forces LF for crates/fdu-py/** and LICENSE.
