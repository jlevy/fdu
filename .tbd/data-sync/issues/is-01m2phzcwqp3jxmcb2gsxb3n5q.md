---
type: is
id: is-01m2phzcwqp3jxmcb2gsxb3n5q
title: "Decide: exact version pins in the published fdu-core crate"
kind: task
status: closed
priority: 0
version: 3
labels:
  - release
  - packaging
  - decision
dependencies:
  - type: blocks
    target: is-01m2phzegm4b3scda7d1xq3gnm
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-09-17T02:08:51.094Z
updated_at: 2026-09-18T03:07:28.148Z
closed_at: 2026-09-18T03:07:28.148Z
close_reason: "Implemented on PR #87: SIGINT, registry READMEs, caret pins, version stamp/LF, 0.2 API note, 200ms interval, transient-summary --no-gitignore, python-smoke --python, JSON 2^53."
---
crates/fdu-core/Cargo.toml pins `libc = "=0.2.189"` and `pulldown-cmark = "=0.13.4"`, matching the
supply-chain guideline's "pin exact in the manifest". In a published library an exact libc pin makes
any downstream build that needs a newer libc unresolvable.

Options: caret requirements in published crates (Cargo.lock and `cargo install --locked` still enforce
reviewed versions for our own builds; document that libraries declare minimum versions), or keep exact
pins and accept resolution failures until the next release. User decision. Changes the packaged
manifest, so it needs the rehearsal rerun.
