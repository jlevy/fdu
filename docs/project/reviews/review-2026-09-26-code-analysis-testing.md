# Code Analysis Testing Review

The [code analysis plan](../specs/active/plan-2026-09-26-code-analysis-presentation.md)
requires evidence across the engine, command line, installed Python wheel, cache routes,
and formats. This review uses `golden-testing-guidelines`, `general-testing-rules`, and
the installed tryscript 0.2.1 reference.
The decision criterion is independent behavioral evidence per maintained test line, with
a useful failure location.
The [implementation review](review-2026-09-26-code-analysis-implementation.md) records
the integrated behavior and performance findings.

## Evidence Owners

| Contract | Best current evidence | Decision |
| --- | --- | --- |
| Lexical counts, language delimiters, chunk splits, CRLF, BOM, unsupported code | `content_code_metrics.rs` tests with hand-counted outputs; the 30-case adjudicated research corpus | Keep focused tests: a CLI golden cannot force every chunk boundary or distinguish a wrong comparator from a wrong implementation. |
| Population admission, negation, refusal, unreadable rules, warm reconciliation | `scan.rs`, `index.rs`, and `query_report.rs` focused tests; path-independence cold oracle | Keep work counters and governed-classification assertions; add public route coverage only where a surface can diverge. |
| Cold, warm, cache-only answers and no-store on partial scans | `tests/correctness` cold oracle, path-independence matrix, cache tests, and CLI lifecycle golden | Keep different mechanisms: a matching answer alone does not prove that a cache served it. Consolidate only repeated fixture setup, not provenance and commit-boundary checks. |
| Code overview, metric ranking, exact shares, unknown population | `query_report.rs` arithmetic/order tests and `cli-content.tryscript.md` full output | Keep arithmetic tests and shared golden. Add one public ordering and bound scenario described below. |
| Adaptive tree bounds and typed omissions | `query_report.rs` exact 1%, eleven siblings, depth and zero-bound cases; CLI axes/full-format goldens | Keep exact boundary tests; avoid expanding the golden with every combinatorial bound. |
| Text color, machine format, escaping | `report_format.rs` span and stripped-output tests; CLI human/JSON/JSONL/YAML goldens | Keep both: spans require a focused oracle, and full output guards the public contract. |
| Cache destination and lifecycle | `cache.rs` path/platform/collision tests and `cli-lifecycle.tryscript.md` | Keep platform-gated model tests plus one real on-disk session. |
| Python package and surface parity | `test_models.py` typed decoder checks, installed-wheel smoke, one tryscript corpus replayed through the Python shim, and path-independence | Keep the shared corpus; do not copy its expected output into Python tests. |
| Native watch and persistent opened state | `watch_session_integration.rs`, opened-root session goldens, and CLI watch golden | Keep as separate platform evidence; native event and retained-state contracts are not visible in a one-shot golden. |

## Current Footprint and Runtime Evidence

Against base revision `4016fd99`, nine tryscript files grew from 4,876 to 5,209 lines
and now hold 183 command blocks.
The 62 tracked fixture files remain at 493 lines and 12,626 bytes; four golden helpers
remain at 536 lines.
The test-source inventory uses the same tracked files at base and now:

| Selector | Files | Base lines | Current lines |
| --- | ---: | ---: | ---: |
| `crates/fdu-core/tests/*.rs` | 6 | 2,884 | 2,880 |
| `crates/fdu-py/tests/*.py` | 7 | 2,824 | 2,961 |
| `tests/path_independence/*.py` | 7 | 2,532 | 2,573 |

Correctness scripts remain at 729 lines.
The 54 inline Rust test modules grew from 34,010 to 35,107 lines (+1,097): core 31,087
to 32,161, Python binding 167 unchanged, and CLI 2,756 to 2,779. This counts top-level
modules under `crates/{fdu-core,fdu-py,fdu}/src` from their `#[cfg(...test...)]`
attribute through the module’s closing brace, including comments and fixture literals.
Of those modules, 51 end at EOF; the three others have separately verified boundaries.
Isolated test-only functions/fields and dedicated support code outside those modules are
excluded. The count includes the installation-guide tests brought in from main.
Rust core and Python-binding test sources contain 962 `#[test]` annotations.
These are inventory counts, not a coverage score.
Generated evidence is outside the listed test-code line totals.
The same tracked file paths and newline count were used at base and at this working-tree
snapshot. The separate opened-root golden corpus has six complete sessions and 185
records at both revisions.
Its trace data grew from 216,284 to 222,869 bytes (+6,585) as the new scope, rule,
selection, and sort fields became observable.
The exact session test and corpus lint pass.
This trace data is larger than the nine CLI session files and must be counted even
though its runner is one Rust test.

On bare-metal macOS 26.5.2/arm64 with external APFS scratch, the integrated local gate
recorded 886 core tests in 25.93 seconds, six parallel-equivalence tests in 26.07
seconds, and 183 shared golden commands in 6.05 seconds.
The six opened-root golden sessions are exercised by one test within the core suite;
they do not have a separately measured tier time.
The two allocation guards passed in 0.04 seconds.
Python package validation passed all 69 tests in 0.44 seconds after a fresh
native-extension build.
These tier times exclude compilation and setup.

The full path-independence matrix passed 16,787 cases against 64 cold answers in 147.496
seconds (147.750 seconds for the make target): 12,674 matching answers, 3,322 expected
refusals, and 791 expected stale-cache outcomes, with no unregistered differences.
Its installed-wheel routes cover `report`, `open`, and `scan`. Linux CI recorded the
shared-corpus parity artifact, and its complete diff was reviewed.
All 19 jobs in
[CI run 36291040472](https://github.com/jlevy/fdu/actions/runs/36291040472) pass,
including native Linux/macOS/Windows tests and Python 3.12/3.14 wheels on all three
platforms. The complete local `make check` passed in 870.17 seconds (14 minutes 30
seconds), including builds, package installation, and all required tiers.
Its 2,267-case subset matrix passed in 35.216 seconds; local parity matched all 24
recorded, classified deviations.
The final gate used the integrated source after the stack incorporated the merged
installation documentation.
The full matrix overlapped the early gate, so these are observed runtimes under shared
host load, not a quiet-host benchmark or evidence of a runtime reduction.

## Findings and Actions

1. **Public metric ordering: addressed.** The shared corpus now records a complete human
   Code view with `--sort code_lines --reverse --min-share 7% --limit all` over the
   existing 15-language fixture.
   The 40-line total remains visible while rows below the threshold are omitted.
   The core file-row oracle separately proves unavailable metrics sort last in either
   direction; copying that edge into the large public golden would add little
   independent evidence.

2. **Path independence axes: addressed selectively.** The subset now asks for a Code
   metric sort with reverse and share threshold, and a Tree with depth, breadth, and
   share bounds. The old depth cases now name Tree explicitly; their previous flat
   projection could pass as matching refusals.
   These cases require a report-shaped cold oracle, so two identical refusals cannot
   pass. Cache destination precedence stays in core path tests and lifecycle goldens;
   adding it to the matrix would multiply cache histories without a distinct answer
   contract.

3. **Root-independent cache-all behavior: addressed.** The Python shim now resolves
   `fdu.cache_directory()` directly from the core cache destination policy.
   A focused Python test covers the public API; two shared lifecycle goldens call
   all-cache status and clear with a nonexistent scan root.
   The installed-wheel replay passes in the final local gate and Linux CI.

4. **Full golden observability: retained.** `check-golden-observability.mjs` rejects
   product-output extraction, `check-golden-invocations.mjs` pins the binary, and the
   portability check rejects local literals.
   The corpus uses one source for CLI and Python replay.
   Keep these guards and review full diffs; do not shorten the 5,209 lines by replacing
   complete responses with `jq` or broad wildcards.

5. **Costly independent proofs: retained.** The cache fault injections, allocation
   bounds, independent reference model, native watch, and Linux-owned parity recording
   prove contracts a short tryscript cannot.
   No test was identified as vacuous enough to delete safely from inspection alone.
   A consolidation should name the independent invariant retained before removing an
   assertion.

6. **Share-filter omissions: addressed.** Code, grouped metric, and extension sections
   now expose `share_omitted` separately from the row-cap bound.
   Text names `--min-share 0%` for share omissions and `--limit all` for row-cap
   omissions; machine output retains selected totals.
   Core arithmetic and the 183-command corpus cover both representations.

7. **Cache fixture identity: addressed.** The lifecycle planter and the orphaned
   analysis-sidecar fixture now use the engine’s recognized suffixes.
   The full cache golden still records unrecognized-file preservation as a separate
   behavior.

8. **Watch child cleanup: addressed in the Rust integration tier.**
   `crates/fdu/tests/watch_persistence.rs` now owns its child with a guard that
   terminates and reaps it when an assertion fails.
   The tryscript watch helpers already kill their children on their expected success and
   failure paths; no separate leak was established there.

9. **Watch route pipes: addressed in the path-independence harness.** The final matrix
   run exposed repeated `ResourceWarning` reports for unclosed stdout and stderr pipes
   from its watch child.
   The route now terminates and reaps the process, joins both readers, and closes both
   pipes on every exit.
   All 37 isolated harness tests pass; direct successful and rejected watch startups
   return without a pipe warning.

10. **Python population route: addressed in the matrix adapter.** The cross-route matrix
    caught 28 unregistered differences for excluded and ignored-only populations: the
    Python `open` and `scan` routes applied the selection only to the report, after
    their retained index had already chosen a different population.
    Both routes now pass the requested population to the producer and use `INCLUDE` when
    the selection leaves it unspecified.
    A rerun also exposed the missing default; the corrected full matrix passes all
    16,787 cases without adding an exception.
    The real cross-route matrix owns this regression, so no mocked forwarding test was
    kept.

11. **Installed-wheel tree contract: addressed.** Linux wheel CI caught an assertion
    that a file-only directory at the display-depth boundary omitted no rows.
    With significant file leaves, that boundary now omits the file row.
    The same smoke case checks the visible child names, truncation, and typed depth
    reason; its installed-wheel replay passes locally.

## Sensitivity and Remaining Limits

The
[correctness evidence](../research/evidence/code-analysis-correctness-2026-09-26.json)
records two deliberately broken cache variants: disabling writes fails all 23 serving
checks as missing snapshots, and falsely serving partial snapshots fails all 23 refusal
checks. The six parity-class tests also reject changed answers disguised as legitimate
surface differences.
The invocation, observability, and portability guards pass, and complete golden diffs
were reviewed rather than blindly regenerated.

The final gate uses the installed wheel for parity and the path-independence subset.
The full matrix additionally covers its configured request/history/mutation combinations
on this host; it does not establish coverage of every possible filesystem or language
construct. Linux owns the parity recording, while native watch behavior has separate
Linux, macOS, and Windows CI evidence.
The macOS correctness fixture could not exercise device nodes or non-UTF-8 filenames;
those omissions are explicit in its evidence artifact.

The review added public cases and repaired harness defects without creating another
harness, duplicating the Python golden corpus, weakening expected output, or deleting an
independent invariant.
Necessary test growth is recorded above; no coverage percentage or runtime improvement
is claimed.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
