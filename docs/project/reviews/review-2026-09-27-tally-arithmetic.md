# Tally Arithmetic Review

## Scope and Contract

This review follows the reported 100% “more files” summary and a predominantly
gitignored row whose bar showed only a non-gitignored cell.
It distinguishes measured accounting, display selection, and whole-cell approximation.
The corrected output contract is in
[output design](../architecture/fdu-output-design.md).

For a complete selected tree, a root row is context.
Its displayed immediate children represent their entire recursive subtrees.
The remainder covers only the other root branches.
These populations must be disjoint:

```text
root = sum(displayed immediate child rollups) + remainder
```

This identity applies independently to regular-file counts, apparent bytes, allocated
bytes, and known gitignored bytes.
Descendant rows explain their ancestor totals and must not be added to them.
Bounds below an already displayed directory change detail, not the population
represented by that directory.

A depth-zero or zero-row display may legitimately have a remainder equal to the root.
A display representing every immediate child has no remainder even if those children are
collapsed. Per-node omission records and diagnostic tips retain that information.
Full expansion has neither remainder nor omission records.

## Review Disposition

The uncommitted-change review used the project’s `review-code` shortcut and independent
engine, query, and command-line audits.

| ID | Severity | Finding | Disposition |
| --- | --- | --- | --- |
| T1 | High | Remainder included usage represented by displayed directories | Corrected shared root-branch reducer and schema |
| T2 | Medium | Independent bar rounding erased the dominant gitignored population | Corrected integer cell apportionment |
| T3 | Medium | Synthetic aggregates beyond `u64` capacity can wrap or panic | Open boundary defect, `fdu-sqyk` |
| T4 | Medium | Nested-only omissions emitted a note referring to an absent remainder | Note now requires an actual remainder; bounds and tips remain |
| T5 | Low | Floating-point conversion could label an extreme value below 1% as 1% | Less-than threshold now uses exact integer arithmetic |
| T6 | Medium | Deep-render test pruned zero-byte descendants before exercising them | Explicit unlimited selection and independent 1,025-node assertion; separate bounded construction/render threads |

The chosen design keeps one engine remainder model for every format.
It changes the summary’s meaning rather than subtracting overlapping directory rows in
each renderer. Tests derive expected populations from a separate leaf ledger.
Documentation and the plan now state the root partition equation and the distinction
between collapsed detail and unrepresented usage.

## Findings and Corrections

### Remainder Meant Unexpanded Leaves, Not Unrepresented Usage

The former reducer summed disjoint omission boundaries throughout the tree.
This did not double-count individual leaves within the remainder, but it included
descendants whose usage was already represented by a displayed directory.
Consequently it could repeat the root’s entire usage under “more files.”
The tests enforced that older interpretation and therefore did not detect the design
error.

The reducer now uses only omission boundaries immediately below the root, or the omitted
root when no root row is shown.
Human and machine formats use this same model.
The changed machine meaning advances the report contract to `fdu.report/10`.

### Independent Bar Rounding Hid the Dominant Population

For 13 GiB out of 212 GiB, a ten-cell bar rounds to one filled cell.
Independently rounding 9.6 GiB of gitignored content against 212 GiB yields zero cells.
The old bar therefore depicted the whole visible cell as non-gitignored.

The corrected calculation first rounds filled width against the root, then apportions
those cells according to the row’s own gitignored fraction.
Integer arithmetic uses a `u128` intermediate, preserving half-cell boundaries even near
`u64::MAX`. Both filled populations stay green: solid `█` for non-gitignored, dark-shade
`▓` for gitignored, medium-shade `▒` when classification is unknown.
Unused width is faint `░`. A whole-cell bar is approximate; exact displayed amounts
remain in the numeric columns.
A ten-cell bar cannot show every small nonzero contribution.
Increasing bar width improves resolution without changing any measured value.

### Less-Than Percentage Labels Need Exact Boundaries

At very large representable totals, converting the numerator and denominator to floating
point can make a value just below 1% appear to equal it.
The less-than label now uses `u128` multiplication and comparison before rounding the
displayed percentage.
Regressions cover `u64::MAX / 100` and `u64::MAX / 1,000`; ordinary percentage
formatting is unchanged.

### Aggregate Overflow Remains a Separate Boundary Defect

Public synthetic observations can supply file sizes whose sum exceeds `u64::MAX`.
Unchecked retained-rollup additions can panic with overflow checks or wrap without them.
Selected-subtree accumulation has the same representability concern.
This requires over 16 EiB in a byte total, so it does not explain the reported
ordinary-size output.

Tracked as `fdu-sqyk`. A correct repair must reject unrepresentable updates atomically,
before mutating retained state, and preserve exact totals.
Saturation is not an exact answer.
Duplicate operations, subtree replacement, conditional operations, and file admission
limits require a projected ledger; merely adding all incoming sizes would incorrectly
reject some representable updates.
This review does not claim that boundary is repaired.

## Independent Checks

The selection test uses a manually enumerated leaf ledger, not index rollups or omission
facts as its expected answer.
It covers ignored populations, scope exclusions, both byte metrics, and composable
display bounds. It checks every displayed directory against its selected leaves and
checks the root partition equation above.

A separate retained-index regression uses hand-counted totals across updates, subtree
removal, and reinsertion.
Apparent sizes and allocation are checked independently.
Two paths sharing an inode count as two file paths under the existing measurement
contract; these totals do not promise deduplicated physical storage or reclaimable
space.

An external filesystem fixture independently compares expected leaf counts and byte sums
with CLI JSON, YAML, and text.
In its regression case the root is 78 bytes and a listed directory represents 28 bytes,
leaving a 50-byte remainder.
The old output incorrectly displayed a 78-byte remainder for this intended contract.

Normalized logical-word estimates are not additive displayed counts.
Their sufficient statistics aggregate first, then the estimator rounds once for each
group.
Summing already-derived per-file estimates is not the group estimator; line, byte,
file, and code counts remain additive.
The audit checks these contracts separately.

Color goldens cover included, excluded, and exclusively gitignored populations plus
unobserved and refused controls.
Boundary assertions cover zero sizes, zero denominators, maximum integer inputs, custom
bar width, and majority-gitignored one-cell bars.

The independent filesystem run against the corrected `fdu.report/10` debug CLI passed 29
of 29 arithmetic assertions.
Its seven regular files contain 78 apparent bytes and 28,672 allocated bytes; one
ignored file contributes 10 apparent and 4,096 allocated bytes.
`files`, summary, tree, type, and extension rows agree with the file-stat ledger.
The simple Rust sources yield 3 code, 1 comment, and 1 blank line; the document share
uses 4 measured words, with 3 in Markdown.
Parsed JSON and YAML report facts match once run timestamps and ages are excluded.
Human tree and grouped percentages match their declared denominators.
The performance file and decimal-GB rates agree with the same displayed elapsed sample
within its rounding precision.

Six display-bound cases conserved root usage.
With depth 1, two rows, and a 20% share threshold, the listed `a/` branch carries 2
files, 28 apparent bytes, and 8,192 allocated bytes.
The remainder carries the other 5 files, 50 apparent bytes, and 20,480 allocated bytes:
`7 = 2 + 5`, `78 = 28 + 50`, and `28,672 = 8,192 + 20,480`. Its human share is 64% by
apparent bytes and 71% by allocated bytes.
Depth zero and zero rows each correctly leave the full root in the remainder; full
expansion has a null remainder.
The former binary reported 78 bytes in the mixed-bound remainder, duplicating the listed
branch.

Raw external-scratch evidence is in the task’s `math-audit-20260927/` directory:
`ledger.json`, `checks-pre-fix.json`, `checks-post.json`, `remainder-matrix.json`, and
captured JSON, YAML, text, and ANSI outputs.
The fixture is `fixture/` under that directory.
The relevant commands use `--cache off` and the same tree with
`--view files,summary,tree,types,extensions,code,documents --analyze all --full` for the
complete report, then `--view tree --depth 1 --limit 2 --min-share 20%` for the
mixed-bound report. The pre-fix and post-fix outputs remain separate.

## Validation Status

The 1,920-case independent leaf-ledger test and all 57 query-report tests passed.
The independent CLI audit passed 29 checks, including six root-partition cases.
All 198 command-line goldens passed with reviewed expected-output changes and portable
patterns preserved. The final nested-only diagnostic regression also passed after its
expectation was corrected to use the core surface’s option vocabulary.
The full handoff targets ran.
Corrective reruns passed the Rust test target, wheel smoke, Python checks (70 tests),
and documentation checks after two stale expectations were updated.
Library-only and minimum-Rust checks, source-package installation, concurrency, 2,267
path-independence cases, 72 release tests, three terminal tests, audits, and
performance-evidence checks passed.
The exact-percentage boundary regression passed.
The Linux-recorded parity artifact was adopted from
[CI run 36359389050](https://github.com/jlevy/fdu/actions/runs/36359389050), commit
`f03eeb551a861a049ece821bfabc8766b7eca8d4`. Its SHA-256 is
`40dac093e2f231e5f5d3b718bdbd3e101978a651ccc37a0f44b4282bd3894747`. The reviewed
differences cover schema 10, the accounting note, root-only remainder values, and skill
wording; no new deviation class was introduced.

The initial corrected and installed candidate `0.1.0-dev+g0d5f9a10a` passed an
independent 77-check audit across 19 invocations, covering both byte measures, full
expansion, bound composition, ignore controls, JSON and YAML facts, ANSI colors, and
quiet diagnostics. The installed skill matches its bundled source byte-for-byte.
Final matrix status is recorded in the
[PR review](https://github.com/jlevy/fdu/pull/136).

### Deep-Tree Validation Gap

The zero-byte fixture originally used the default 1% filter, pruning descendants before
rendering. The corrected test removes every display bound and independently asserts all
1,025 nodes and no omissions.

A separate intermittent Windows failure recurred during report construction in
[run 36362072990](https://github.com/jlevy/fdu/actions/runs/36362072990), while
identical core source passed the implementation and top-layer runs.
The pinned Rust 1.97.1 Windows runtime reserves 20 KiB of each spawned thread’s stack
for overflow handling, leaving little margin inside the test’s 64 KiB reservation.
Review found no depth-recursive call in this fixture’s report-construction path.

The test now constructs and verifies the report on a bounded 128 KiB thread, then moves
it to a separate 64 KiB thread for text, JSON, JSONL, YAML, streaming, and explicit
drop. Phase markers distinguish request, report construction, verification, each
renderer, and destruction.
This preserves the narrow renderer check and bounds construction independently.
The focused macOS test passes; final Windows results and the disposition of `fdu-4793`
are recorded in the
[PR review](https://github.com/jlevy/fdu/pull/136#issuecomment-5860865321).

## Earlier Presentation Follow-Up

The subsequent presentation pass (`593cea57`) keeps remainder quantities in the same
numeric roles as ordinary rows; only the “more files” annotation is gray.
Unused bar cells are dim green.
Directory names use regular cyan without bold only when their own path is gitignored,
directly or through an ignored ancestor.
Merely containing ignored files does not change a directory name, and file-name styling
is unchanged.

A nullable `entry_ignored` field carries that own-entry fact through Rust tree reports,
machine output, and Python models.
Tests distinguish empty ignored directories, inherited ignored directories, mixed
directories, unknown classification, and unignored ancestors retained by
`--ignored=only`. The expanded ANSI golden checks the actual folder and file styles.
Progress and `perf:` now share one elapsed-time formatter, including two-decimal seconds
such as `151.33 s` without minute/hour notation.

Validation passed 98 CLI tests, 912 core tests (one intentional skip), 198 command-line
goldens, 70 Python tests, and three terminal tests.
The full handoff run completed; corrective Rust and library-only reruns passed after the
progress-width expectations and one exact JSON assertion were updated.
Minimum-Rust, wheel and source-package installation, concurrency, documentation, audits,
2,267 path-independence cases, and 72 release tests passed.
CLI/Python parity matched all 53 classified differences.

That presentation pass used the parity record from
[Linux CI run 36361381366](https://github.com/jlevy/fdu/actions/runs/36361381366),
producer `345278eb4cde8b3ac03850bab240eae96ea0c036`, with SHA-256
`1b02283b9e1eadcbfd142ad1345f0f7ef38609273f6a2bb65d9a8436d98b9a6f`. Its changes add the
exact entry-classification facts and corresponding hunk offsets.
The later commit corrects a test assertion and adopts this artifact; it does not change
the report facts. Final installed-candidate acceptance and platform CI results are
recorded in [PR #136](https://github.com/jlevy/fdu/pull/136).

## Shared Formatting and Live Rates

The final presentation follow-up (`0433bcc0`) supersedes the timer and remainder-label
styling above. Only `… and` is gray; the following file count uses the ordinary
foreground. Progress uses seconds with one decimal, while final performance retains its
more precise duration.
A middle dot separates the size and timer.

Live rates appear only after five seconds and only when walk facts exist.
Progress redraws at most every 100 ms and drops the optional rates first on narrow
terminals. Progress and final performance share integer throughput arithmetic,
comma-grouped files per second, and binary GiB per second.
Human classification labels consistently say `gitignored` and `non-gitignored`. Cache
status and cleanup summaries use the shared binary size formatter and color roles;
structured formats preserve exact numeric bytes.
Python cache rendering exposes the same optional color setting as the core renderer.
These rules are documented beside the shared helpers and in the
[output design system](../architecture/fdu-output-design.md).

Validation passed 100 CLI tests, 912 core tests (one intentional skip), 198 CLI goldens,
Python checks and packaging, library build configurations, minimum-Rust checks, audits,
2,267 path-independence cases, 72 release tests, and three terminal tests.
The handoff gate stopped at the expected stale parity recording; after adopting the
Linux-produced record, parity passed with all 53 classified differences matched, and all
remaining handoff targets passed separately.

The new parity recording comes from
[Linux CI run 36366851633](https://github.com/jlevy/fdu/actions/runs/36366851633),
producer `0433bcc090ca484309f9f9673faff317b2bee5e9`, artifact `10946909301`, SHA-256
`fbbf7e43484a9e13b508a204dfa0e50273dcd36e4e06e3b27b156dd49cd3df9e`. Its diff changes
only the expected human wording and throughput-unit help text.
That run also reproduced the existing native-watch setup race tracked as `fdu-21ns`: one
report acquired a `WatchSetupRace` diagnostic during the equality assertion.
This remains a separate open issue; no test assertion was weakened to hide it.

The freshly installed `fdu 0.1.0-dev+g0433bcc09` passed 111 checks across 27
invocations, including colored cache sizes and exact structured values.
A live terminal audit passed six checks over 64 frames: no rates before five seconds,
gray rates afterward, one-decimal seconds, and the 10 Hz redraw bound.
The installed skill matches this build.
The wheel SHA-256 is `f08a19f403b29a699d9af30bd1ee2df195c74631f6e57c83afa0fd12f62667de`.
Final PR heads and CI disposition are recorded in the
[PR review](https://github.com/jlevy/fdu/pull/136#issuecomment-5860865321).

## Code Table and Analysis Vocabulary

The follow-up at `7a499493` implements `fdu-2u22`, `fdu-xjn3`, and `fdu-k96q`. Code is
one table with aligned code-line, share, comment, blank, analyzed-file, and language
columns. Primary TOTAL cells are bold; population details remain gray.
Totals and share denominators cover the complete selected population before row/share
limits. Unmeasured values use a dash; measured zero stays numeric.
A single stderr note explains totals that include hidden languages, and `--quiet`
suppresses it.
The existing Code overview model and `fdu.report/10` schema are unchanged.

The vocabulary review keeps measurement and presentation separate: `lines` selects
Families by default, `code` selects Code, and `words` selects Documents.
Documents selects prose and markup; word analysis can also measure other accepted text,
which is available through Types.
Physical-line analysis remains useful alone and is already included by code/word
analysis. Unsupported SLOC languages retain physical-line metrics.
Analyzer names supplied as views produce actionable guidance using the caller’s CLI
flags or Python fields.
`--workers` replaces the longer CLI spelling and controls content-analysis concurrency;
Rust/Python `analysis_workers` and the separate directory scan pool keep their
semantics.

Review found and corrected trailing padding on rows without population annotations and
stale scope descriptions in the usage/design documents.
A proposed ANSI wrapper golden was removed when the observability gate correctly
rejected its abbreviated response; the existing focused renderer test checks all six
bold TOTAL cells instead.
Four direct golden sessions cover bounded rows, zero displayed rows, quiet output, and
unsupported measurements.
Existing full-output goldens cover the ordinary table.
The README example was captured from the installed build against revision `7a499493`.

The complete `make check` gate passed: 914 core tests (one intentional skip), 100 CLI
unit tests, 202 portable CLI golden sessions, 70 Python tests, package/concurrency
checks, build-feature configurations, minimum-Rust checks, documentation, audits, 2,267
path-independence cases, 72 release tests, and three terminal tests.
All 19 jobs passed in the
[implementation CI run](https://github.com/jlevy/fdu/actions/runs/36371997439).

The reviewed parity record was produced by
[Linux CI run 36371830880](https://github.com/jlevy/fdu/actions/runs/36371830880),
producer `d59f0a39063256bd66153a15e2a6126e06086e5f`, artifact `10949695745`, SHA-256
`f26f1f8334052fce7d53a580df5b6b012c0e478e127bd685fda70c3e48a889ed`. Local replay matches
all 55 classified differences; the two new ones are the expected CLI/Python bound-tip
vocabulary in the bounded Code cases, with identical table output.

Freshly installed `fdu 0.1.0-dev+g7a499493e` passed all 111 existing installed-binary
checks, and its installed skill matches the bundle.
A separate hand-counted fixture confirmed two code lines, two comments, one blank, and
two analyzed files out of three source files even when only one language is displayed.
The wheel SHA-256 is `3003c677e50d24fbb3219bea0118a0da54ed78b02f7f8d721359824280dd5e2b`.
Final upper-stack heads and CI disposition are recorded in the
[PR review](https://github.com/jlevy/fdu/pull/136#issuecomment-5860865321).

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
