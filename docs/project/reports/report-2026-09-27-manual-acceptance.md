# Manual Acceptance — Output Design and Complete Inventories

## Candidate and Scope

This pass executes the 33-item manual checklist in
[PR #136](https://github.com/jlevy/fdu/pull/136), including the partial-tree regression.
Three reviewers divided tree/population, analysis/library/watch, and presentation/skill
checks; the main task checked cache behavior, complete inventories, and follow-up fixes.

The installed wheel initially reported `fdu 0.1.0-dev+g8de84067b.dirty`. Its SHA-256 was
`6af1b68254567c118f806873e76d55d98df332d199a331a60c8d459fd5d202f6`. Later debug checks
used the same base commit with additional uncommitted fixes; they are identified
separately in the [retained evidence](manual-acceptance-2026-09-27.json).
This is provisional development validation, not clean release-candidate acceptance.
The repeated version string does not establish identical binaries; the wheel hash and
separate evidence groups identify the tested stages.

Fixtures, caches, and build outputs used external storage on macOS arm64. No real user
cache was cleared. Permissions changed for disposable failure fixtures were restored.
The cache-clear check affected only its dedicated fixture and preserved an unrelated
sentinel. Timings are observations under normal desktop load, not a performance
comparison.

## Checklist Results

“Pass after fix” means the installed candidate exposed a problem and a subsequent debug
run or focused regression verified its correction.
“Partial” identifies checks whose programmatic portions passed but whose remaining
acceptance conditions were unavailable.

| PR item | Result | Evidence or remaining condition |
| --- | --- | --- |
| 1.1 Default tree | Pass | Significant files, root at depth zero, depth-five cutoff |
| 1.2 Individual display limits | Pass | Summary totals unchanged under every listed bound |
| 1.3 Composed bounds | Pass | Each bound lifted independently; mixed fixture retained |
| 1.4 Threshold fixture | Pass | 99 B hidden, 100 B and 101 B shown at a 10,000 B root; eleven qualifying siblings; empty/zero-share cases |
| 1.5 Display versus discovery depth | Pass | Display depth preserves 10,000 B; scan depth observes only 9,100 B |
| 2.1 Ignored populations | Pass | Code partitions 4 = 2 + 2; include/exclude/only read 183/127/56 B |
| 2.2 Ignore telemetry | Pass | Two control files and six accepted rules, including duplicates and negation |
| 2.3 Disabled ignore controls | Pass | Unknown classification; incompatible populations refused with status 2 |
| 2.4 Refused/unreadable rules | Pass | Actual permission failure and incomplete classification remain visible |
| 3.1 Analyzer defaults | Pass | Families, Code, Documents, and combined defaults |
| 3.2 Partitions and denominators | Pass after fix | Shell defect initially produced 14 code/4 comment; corrected to 13/5, with one blank |
| 3.3 Rankings and ties | Pass | Both sort directions, deterministic ties, unavailable counts last |
| 3.4 Language share filtering | Pass | Hidden language row does not reduce the denominator |
| 3.5 Hand-counted syntax | Pass after fix | Shell separator comments and arithmetic shifts repaired; chunk-boundary regressions added |
| 3.6 Invalid analysis/view combinations | Pass | Refusals and explicit metadata-view analysis explanation |
| 4.1 Terminal appearance | Partial | PTY names, counts, parentheses, Unicode, and narrow output inspected; grouped label color fixed; real light/dark visual judgment not performed |
| 4.2 Color controls | Pass | Auto, always, never, NO_COLOR, resets, and positive shares below 1% |
| 4.3 Machine formats | Pass | JSON/YAML/JSONL parse to equivalent values; clean stdout; post-fix remainder fields agree |
| 4.4 Performance diagnostics | Pass | Work/rule counters, cache reuse, rates, final stderr position, zero cache-only walk |
| 5.1 Cache reuse and mutation | Pass | Three cached records on repeat; edited counts equal cold answers |
| 5.2 Cache policies | Pass | Refresh/read-only/only/off; read-only/off preserve cache hashes and mtimes; misses refuse |
| 5.3 Scope invalidation | Pass | Nine population/analyzer combinations and newly eligible ignored subtree match fresh results |
| 5.4 Cache paths and names | Pass | Metadata/analysis filenames; explicit path > environment > XDG > macOS default |
| 5.5 Isolated cache clearing | Pass | Fixture snapshots removed; sentinel survives |
| 6.1 Environment inventories | Pass | Four overlapping directory rows; union counts four files without double-counting |
| 6.2 Directory ages and exclusions | Pass | Newest subtree modification governs age; nested exclusion survives parent selection |
| 6.3 Hard links | Pass | Two linked paths and an independent file each contribute; no reclaimable-space claim |
| 6.4 Python surfaces | Pass | Report/open/scan totals agree with CLI; narrow retained scope refuses widening |
| 6.5 Watch and interruption | Pass after follow-up | File/ignore-rule events and analyzer refusal; real PTY edit observed, terminal modes unchanged after SIGINT |
| 7.1 Skill installation | Pass | Both locations, unchanged rerun, nested Git-root discovery, agent base, handwritten refusal |
| 7.2 Fresh agent session discovery | Not verified | Generated skill inspected; a fresh desktop agent session was not launched |
| 7.3 Published zero-compile upgrade | Pending publication | Current public release predates the changes; local wheel installation is not evidence of a future published upgrade |
| Partial-tree regression | Pass | Known tiny siblings hidden despite unreadable branch; status 2 and warning retained; 0% reveals siblings |

## Follow-Up Design Checks

The initial presentation emitted a remainder at every boundary.
The revised core model combines disjoint hidden subtrees once per tree.
The mixed-bounds fixture has seven files and 10,000 apparent bytes; six hidden files
contribute exactly 8,000 bytes.
Terminal output matches the literal core golden:

```text
                                 … and 7.8 KiB (6 files) more
```

JSON, safely parsed YAML, and JSONL all report the same recursive file count, both byte
measures, and ordered share/depth/breadth/row reasons.
Actual filesystem allocation and the synthetic core fixture allocation are deliberately
different; each agrees with its own fixture.
A hidden unreadable branch produces null counts and sizes rather than claiming an exact
zero.

Full expansion contains all 17 nodes: root, nine directories, and seven regular files.
Its remainder is null, every omission collection is empty, and no omission notes or tips
are emitted. Aggregate measurements remain unchanged.

The `--full` shorthand is equivalent to explicit unlimited depth, breadth, rows, and
zero share. Explicit bounds override the shorthand regardless of order.
Neutral bounds are valid for flat views too; finite bounds still require an applicable
view. Three controlled filename searches match the path sets from installed `find` and
`fd`, including hidden and ignored content.
Directory inventory rows expose recursive usage; file rows expose each file’s own size.
Comparison excludes elapsed age values, which naturally change between invocations.

## Final Local Wheel Check

A later local wheel from the dirty checkout was installed without index access or a
build into an isolated Python 3.12 environment on external scratch.
Its SHA-256 is `88ae7132d0d0e1dd5eb8b40162748a09b872def90de35037ac53feb9e94bc696`. This
identifies a different artifact from the initial installed wheel above.
The installed command reported `fdu 0.1.0-dev+g8de84067b.dirty`.

Both installed-wheel smoke scripts passed: `smoke.py` reported `fdu._native 0.1.0 ok`,
and `public_smoke.py` reported `fdu 0.1.0 public API ok`. Focused checks found `--full`
equivalent to explicit all bounds in Tree, JSON, JSONL, YAML, Paths, and Long output.
An explicit depth-zero bound won in either argument order; the structured remainder
counted three hidden regular files and 103 apparent bytes, and the Python model parsed
the same values. The installed CLI and Python parity shim agreed on tree JSON, file
Paths, directory Long, and their diagnostic lines.

The shared corpus passed 192 sessions against the final debug CLI, including flat file
paths, recursive directory Long rows, and a zero-row human remainder.
Those flat commands use metadata only (`--cache off`, no `--analyze`); their output has
no startup performance footer or content-body reads.
The golden run tested the debug CLI, while the two smoke scripts and focused
cross-format checks tested the installed wheel.

The wheel hash, exact setup and smoke commands, and captured smoke output are in the
[retained evidence](manual-acceptance-2026-09-27.json) under `final_wheel_validation`
and `final-wheel/evidence/`. External raw logs are under `$FDU_WHEEL_SMOKE/evidence/`,
where `$FDU_WHEEL_SMOKE` denotes this task’s external scratch directory.
The wheel was built from uncommitted source; this check does not establish a clean
release build.

## Clean Local Installation

The committed stack candidate `fdu 0.1.0-dev+g4b1bd5f5a` was built as a wheel and
installed with `uv tool install --force --no-build --no-index`. Its SHA-256 is
`6d38ba25670240f5dae83e296fb77288312c455a91043cfcbd8eb183d9c1983c`. The generated user
skill matches the installed command’s `--skill` output byte-for-byte.
An installed-binary probe verified schema 9, null remainder and empty stderr with full
expansion, and one six-file remainder with a depth-zero projection.
Bound notes and the single depth tip appeared on stderr only.
The evidence is retained as `clean_install_verification` in the JSON artifact.
Subsequent parity-harness and test changes do not alter this binary’s behavior; final CI
is still a separate gate.

## Local Handoff Gate

The complete `make -k check` run passed every target except the old parity recording.
That included 897 all-build-feature core tests, 815 library-only core tests, 192 shared
golden commands, strict Python checks and 70 Python tests, concurrency checks, wheel and
source-package smoke tests, documentation, performance-evidence checks,
minimum-Rust-version checks, and dependency audits.
The path-independence subset passed 2,267 cases; all three terminal tests, 37 harness
tests, and 72 release tests passed.

Adopted the exact authoritative `deviations-python-linux` artifact from
[Linux CI run 36349895447](https://github.com/jlevy/fdu/actions/runs/36349895447),
produced at `282d3c1f`. Its SHA-256 is
`0c3019a4e13589897c690445b1d7681c896e11b54d0e66d7cbe02ce3311ba7b8`. `make parity-check`
then passed locally: all eight classification tests and all 52 recorded differences
matched. Every local handoff target has therefore passed; the full invocation and parity
rerun are distinct evidence steps.

Recording review also compared two independent Linux observations after applying the
same normalization: they were byte-identical.
Classification checks concrete values before serialization masks only fields whose
paired golden explicitly uses a typed pattern.
Literal sizes, schema values, and fixture-root identity remain checked.

## Presentation Follow-Up Manual Pass

The debug candidate built from the presentation follow-up on `f184a53e` (identified as a
dirty development build) passed 22 independent assertions on a real filesystem fixture
containing 1,006 regular files, an ignored subtree, and a sparse 2 GiB file.
This is behavioral evidence for the working changes, not final clean-wheel provenance.

- File counts and file rates use grouped integers; sizes at least 1 GiB are bold.
- Notes, tips, and performance are gray; warnings are yellow without bold; fatal
  prefixes are red and bold.
- Directory suffixes are gray, while the root `.` has no suffix.
  The CLI normalizes the requested `..` root to `.`, so the literal `..` renderer case
  remains covered by the focused renderer test.
- Bar widths 20, 0, and -1 behave as specified.
- Quiet mode preserves stdout and silences informational stderr; an actual permission
  failure still produces its warning.
  Fixture permissions were restored afterward.
- Include, exclude, only, and unread ignore classification were exercised in plain and
  forced-color output.
- Full JSON parses; `--full` matches explicit unlimited bounds.
  All 1,009 tree nodes have no hidden remainder or omissions, and machine output has
  empty diagnostic stderr for this complete fixture.

The clean top-of-stack wheel at `1f5bc81a` was then installed as
`fdu 0.1.0-dev+g1f5bc81af`, replacing the prior local installation.
Its SHA-256 is `7c9b680db74ce37e73d153aafc5a41ec929b13d4a006eb78802dfd6c76070cbe`. The
installed skill matches `fdu --skill` byte-for-byte.
Subsequent parity-evidence commits do not change its executable source.

The complete local gate plus corrective reruns passed all targets: 198 shared golden
commands, 98 CLI tests, 906 core tests, library-only and minimum-version tests, 70
Python tests, wheel and source-package smoke tests, concurrency checks, 2,267
path-independence cases, release and terminal tests, documentation, performance
evidence, and audits.
Initial lint and grouped-output expectation failures were corrected and replayed.
The editable Python environment also needed its old native extension refreshed; that
gate improvement is tracked separately as `fdu-ukg6`.

Adopted the exact Linux artifact produced at `c86c50de` in
[run 36355306692](https://github.com/jlevy/fdu/actions/runs/36355306692), SHA-256
`915948b47d9d32ec2827ca15f87716520f377575d04c264e45c3c5c52a3a84aa`. The local parity
replay passes all eight classifier tests and matches all 53 classified differences.
These are separate full-gate and targeted-rerun observations; final current-head CI is
recorded in the PR.

## Remaining Acceptance

The clean committed wheel is installed and verified, and all local handoff targets
passed. Final CI on the artifact-adoption commit is recorded in the PR rather than
inferred from these local observations.
Light/dark visual judgment, fresh-session skill discovery, and post-publication upgrade
acceptance remain explicitly open.

Tracking: `fdu-kwjc` for this manual pass, `fdu-0578` for the output design, `fdu-kqqh`
for shell counting, and `fdu-hfoc` for discoverability of the Code view.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
