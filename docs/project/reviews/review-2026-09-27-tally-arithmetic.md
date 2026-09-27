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
The Linux-recorded parity artifact still needs CI refresh for schema 10 and the changed
accounting note; it has not been regenerated locally.
The new candidate is not installed yet.
CI and installation evidence will follow in the PR review record.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
