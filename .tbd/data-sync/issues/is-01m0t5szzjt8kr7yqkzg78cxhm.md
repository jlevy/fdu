---
type: is
id: is-01m0t5szzjt8kr7yqkzg78cxhm
title: "Gitignore rule: the feature-gated ignore dependency and its evaluator"
kind: feature
status: open
priority: 1
version: 2
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-integration.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0prgyv1eq0g0mzgntn1p4n6
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-24T15:21:45.174Z
updated_at: 2026-08-24T15:22:34.575Z
---
The first Path-tier tag rule, and the only one carrying a dependency. Decided
2026-08-24: fdu-core takes the `ignore` crate behind a `gitignore` cargo feature,
DEFAULT-ON beside `watch` -- notify's exact precedent: "the shipped binary matches
gitignore; --no-default-features and library consumers do not." The tag model itself
(fdu-mvt3) is always-on and dependency-free; only this rule costs.

MEASURED EVIDENCE the decision rests on (2026-08-24): +1.06 MiB on a stripped LTO
release binary against a realistic use (GitignoreBuilder + match), 9 new crates
(ignore, globset, aho-corasick, bstr, regex-automata, regex-syntax,
crossbeam-deque/-epoch/-utils), lockfile 73 -> 82, ~13s cold compile against fdu's ~59s
full release build, and no lean mode (ignore has one feature flag; regex-automata is
mandatory). fd takes the same crates among 16 direct deps; ripgrep owns them as
workspace members. The library/binary asymmetry is what the feature gate answers.

MSRV TRAP, found by checking rather than assuming: ignore 0.4.31+ and globset 0.4.20
declare rust-version = 1.88, above fdu's MSRV 1.85. PIN `ignore = "=0.4.30"`
(published 2026-07-17, clears the 14-day cool-off; rust_version null) and hold globset
at 0.4.19 (2026-07-15, also clear) in the lockfile via `cargo update -p globset
--precise 0.4.19`. The exact-pin precedent is fdu-core's own `pulldown-cmark =
"=0.13.4"`; comment the pin with the MSRV reason so the next upgrade attempt reads it.
A null rust_version is unenforced, so verify the whole subtree with
`cargo +1.85.0 check --all-features`, and run `make cross-lint` -- this code is not
platform-gated but the gate exists for exactly this kind of addition.

WHAT LANDS:
- Cargo: feature `gitignore = ["dep:ignore"]` in default features; the pins above;
  Cargo.lock committed; deny.toml confirmed passing (BurntSushi crates are
  Unlicense OR MIT -- verify the license allowlist covers Unlicense).
- The rule: Path tier, id `gitignore`. An index-held evaluator builds
  ignore::gitignore::Gitignore from the .gitignore files under the root and answers
  matched_path_or_any_parents at apply time -- one computation site, so a watch upsert
  is tagged identically to a scan upsert. Correct negation is the point; the closed
  spike fdu-p35d (0.39-1.76 us/entry) proved the matcher and its cases become tests.
- Control-file lifecycle: an upsert, modify, or remove of a .gitignore rebuilds the
  evaluator and escalates InvalidateSubtree for the directory it governs, so watch
  re-tagging re-walks exactly the affected scope.
- Enabling the rule without the feature is an enable-time error naming the feature.
- tag_rules_fingerprint covers the enabled set including this rule, so a snapshot taken
  without gitignore is not reused for a scan with it.
- OUT OF SCOPE v1, recorded rather than silent: global core.excludesFile,
  .git/info/exclude, nested-repository boundary semantics. A later bead extends the
  evaluator, not the model.

Blocked by fdu-mvt3 (the model this rule plugs into).
