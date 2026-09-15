---
type: is
id: is-01m2k2pp6jw10vq759yyetmcmb
title: README library examples and schema prose name APIs the release does not have
kind: bug
status: open
priority: 1
version: 1
labels:
  - stack-followup
  - release
dependencies: []
parent_id: is-01m2h6a9wd6f6xexfaw93whryr
created_at: 2026-09-15T17:44:13.777Z
updated_at: 2026-09-15T17:44:13.777Z
---
Found while fixing fdu-k4ad and fdu-ih88 (branch claude/release-shipped-text, commit bc704c5). These are shipped surfaces: README.md is the fdu crate's crates.io page, crates/fdu-core/README.md is fdu-core's, and crates/fdu-py/README.md is the PyPI long description.

Library examples that would not compile or run:
- README.md:555 and :560 use fdu::content::AnalysisProfile and config.analysis.profile = AnalysisProfile::Full. No AnalysisProfile type exists in fdu-core; OpenConfig::analysis is a content::AnalysisRequest over AnalysisSet.
- crates/fdu-core/README.md:49 and :54, the same Rust example against fdu_core.
- README.md:579 and crates/fdu-py/README.md:26 call fdu.AnalysisOptions(profile=fdu.AnalysisProfile.FULL). The Python package has no AnalysisProfile, and AnalysisOptions takes analyze= (an fdu.Analysis value or comma-separated set) and workers=.
- crates/fdu-py/README.md:60 says AnalysisProfile covers none, basic, code, documents, and full 'as the Rust CLI'. The CLI's --analyze accepts none, lines, code, words, and all.

Schema and analyzer prose that contradicts the binary:
- README.md:463 says metadata-only machine reports use fdu.report/1; README.md:471 and :513 say fdu.report/3. The binary emits fdu.report/4 and fdu.report/5 (--docs and --skill say so and a unit test pins them).
- README.md:500 and :503 describe a 'documents profile' and 'full combines'; those are the old profile names for the words analyzer and all.

Neither README example is compiled: no include_str!(README) doctest exists for either crate, which is how both drifted. Fix: rewrite the examples against AnalysisSet and fdu.Analysis, correct the schema strings, and consider a doctest or a test that the README's Rust snippet compiles so the next rename cannot leave it behind.
