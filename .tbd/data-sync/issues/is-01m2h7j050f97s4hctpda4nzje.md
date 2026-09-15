---
type: is
id: is-01m2h7j050f97s4hctpda4nzje
title: Every shipped surface still says pre-release, unpublished, or Pre-Alpha
kind: task
status: open
priority: 1
version: 1
labels:
  - release
dependencies: []
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-09-15T00:30:34.143Z
updated_at: 2026-09-15T00:30:34.143Z
---
Found by the 2026-09-14 release-readiness audit (scratchpad reviews/release-readiness.md). These statements become false the moment 0.1.0 is on crates.io and PyPI, and they are user-facing: README.md:14-25 ('Status: pre-release' callout), README.md:245-282 ('Until the crate is published, install from source', 'Publishing is Phase 1 work. cargo install fdu and uvx --from fdu==<version> fdu are future commands; neither package should be presented as available'), README.md:565-575 ('without implying that a public release exists'); crates/fdu-core/README.md:14-24 ('Status: pre-release', 'The crate is not published yet', path dependency example instead of a version) and its feature paragraph (fixed by #60); crates/fdu-py/README.md (final 'Status: pre-release, not yet published to PyPI', 'That registry command is conditional until the first release is actually on PyPI'); crates/fdu-py/pyproject.toml:14 classifier 'Development Status :: 2 - Pre-Alpha' (use 4 - Beta or 3 - Alpha); SECURITY.md:5-6 ('Before the first public release, only the current main branch receives security fixes'). Also: crates/fdu/src/skills/SKILL.md:40-44 conditions the uvx example on 'if this release is published on PyPI' (fine once true, re-read it); docs/project/guides/release-process.md says 'Within the 0.1 series, incompatible Rust or Python API changes require a documented deprecation path when practical', which the planned 0.2.0 (default-on .gitignore, schema bump) would not honour; decide the 0.x rule and state it once. Do this in the release PR alongside the CHANGELOG [0.1.0] section, with make docs-format. Note crates.io renders README.md from the crate root, so relative links into docs/ do not resolve there; consider absolute GitHub links for the crate READMEs. Acceptance: no shipped surface (README, crate READMEs, wheel metadata, --docs, --skill, SECURITY.md) describes fdu as unpublished or pre-release; install commands are the real ones.
