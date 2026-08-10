---
type: is
id: is-01kzmnxr765dahd3cr5j58eews
title: Expose the Rust CLI from the installed Python wheel
kind: feature
status: closed
priority: 1
version: 4
spec_path: docs/project/specs/active/plan-2026-08-09-fdu-cli-ux-and-agent-skill.md
labels:
  - python
  - packaging
dependencies:
  - type: blocks
    target: is-01kzmnxy0xvkvazmqvdwsjm20h
parent_id: is-01kzmnx3taexx4cq4m722p0yp0
created_at: 2026-08-10T01:52:17.125Z
updated_at: 2026-08-10T02:09:06.390Z
closed_at: 2026-08-10T02:09:06.389Z
close_reason: Registered the fdu_py:main console script, delegated Python sys.argv to the shared Rust boundary with the GIL released, expanded installed-wheel smoke coverage, proved direct local-wheel uvx execution, and added a three-OS Python wheel CI matrix.
---
Following maturin's mixed PyO3/CLI guidance, register [project.scripts].fdu = fdu_py:main, extract sys.argv at the Python boundary, release the GIL, and delegate to the same Rust CLI runner. Expand installed-wheel smoke coverage for version, help, JSON scan, usage exit 2, no traceback, and continued module/watch-feature behavior. This prepares but does not perform PyPI publication.
