# Feature: End-to-End CLI Behavioral Specification with Golden Tests

**Date:** 2026-08-09

**Author:** fdu project

**Status:** Complete

## Overview

Specify the `fdu` command as an executable public contract and protect that contract
with a small tryscript suite that runs the built binary in isolated filesystem
sandboxes. The suite covers complete user sessions rather than individual implementation
units: invocation and failure behavior, human output, structured output, and the cache
lifecycle.

The suite uses four session files and one portable fixture.
Exact output is the default.
Patterns are limited to values the test does not control: sandbox paths, filesystem
timestamps, allocated block counts, and operating-system error text.
Tryscript 0.2.0 expresses exact blank stderr lines directly and rewrites duplicate
blocks by source position, so the suite needs neither a newline regex nor distinct
option spellings as updater workarounds.
Focused Rust tests remain authoritative for behavior that cannot be represented portably
by a shell-driven golden test, including permission failures, non-UTF-8 names, terminal
detection, and broken pipes.

This plan is a completed child of
[plan-2026-08-08-fdu-phase-1.md](../active/plan-2026-08-08-fdu-phase-1.md).
It advances the human and agent CLI work tracked there without coupling the CLI contract
to the portable walker that the syscall layer will replace.

## Goals

- Exercise the actual `fdu` binary from argument parsing through filesystem scan,
  rendering, cache I/O, stdout and stderr, and process exit status
- Make the desired CLI behavior readable as Markdown and reviewable as ordinary diffs
- Cover every current flag and every public exit status with the fewest orthogonal
  scenarios that preserve full output context
- Run deterministically on Linux, macOS, and Windows
- Keep stable values exact and normalize only values proven to vary by run or platform
- Pair whole-output goldens with focused semantic tests where exact text alone cannot
  prove identity, terminal, or operating-system behavior
- Turn every product defect found while writing a scenario into a failing test, a row in
  this spec, and a linked bead before fixing it

## Non-Goals

- Benchmark walker or cache performance; the Phase 1 benchmark harness owns those gates
- Test index, snapshot parser, watcher, Python binding, or reducer implementation
  details through the CLI
- Duplicate edge cases already covered more clearly by focused unit or property tests
- Make platform-dependent allocation values identical across filesystems
- Use broad `[..]` or `...` elisions to make unstable tests pass
- Automatically approve regenerated output in CI
- Add a bespoke Rust golden runner when tryscript already supplies sandboxing, command
  sessions, exact stream and exit matching, diffs, and controlled updates

## Evidence and Constraints

### Current Coverage

The current CLI coverage has two strong but narrow tests:

- `cli::tests::schema_v2_golden_covers_kinds_and_partial_errors` renders a synthetic
  index and compares exact JSON, including all four entry kinds and a partial error
- `crates/fdu/tests/cli_exit.rs` invokes the built binary on Unix and verifies partial
  exit status 2 and `--allow-partial`

These tests should remain.
They do not specify the installed command’s help, human format, argument failures,
ordering and limits, cache transitions, or cross-platform behavior.

### Tryscript Findings

The implementation and documentation at
[jlevy/tryscript](https://github.com/jlevy/tryscript) were reviewed at release `0.2.0`
and source commit `58172b585e16a9a79605a4e04fdbfe9a11ebe02a`.

The features this design relies on are:

- A fresh sandbox per `.tryscript.md` file, shared by every command in that file
- Recursive fixture copies before the first command
- Separate stdout, stderr, and exit-code assertions
- Project-root and frontmatter `PATH` configuration suitable for `target/debug/fdu`
- Exact matching with named regular-expression patterns for unstable fields
- Explicit `--update` and wildcard-expansion workflows with readable diffs

The relevant limitations are:

- Commands use the platform shell, so fixture construction should not depend on POSIX
  utilities
- Output matching strips ANSI escapes, so it cannot prove that color was emitted
- Filesystem allocation and timestamp values are inherently platform-dependent
- Temporary paths differ by run and JSON escapes Windows path separators
- `--update` replaces a failing block’s named patterns with one run’s literal values,
  making the next sandbox fail even after an intentional stable-output update

The suite therefore uses committed fixtures and `node -e` only for the two state changes
that must occur between commands: modifying a fixture file and corrupting a generated
snapshot. Node is already the runtime executing tryscript.
Every invocation has its own console fence.
Release 0.2.0 resolves the exact blank-stderr, multiple-prompt, and duplicate-block
hazards reported in [tryscript issue 45](https://github.com/jlevy/tryscript/issues/45),
[issue 46](https://github.com/jlevy/tryscript/issues/46), and
[issue 47](https://github.com/jlevy/tryscript/issues/47). The immediate comparison after
`make golden-update` still fails safely if update mode literalizes an approved pattern;
reviewers restore only the named unstable fields and rerun the suite.
Preserving those patterns automatically remains tracked in
[tryscript issue 49](https://github.com/jlevy/tryscript/issues/49).

## Behavioral Contract

### Output and Exit Status

| Situation | Stdout | Stderr | Exit status |
| --- | --- | --- | --- |
| Complete scan | Complete selected human or JSON view | Empty | 0 |
| Partial scan | Complete selected view labeled incomplete | Empty; details are part of the selected view | 2 |
| Partial scan with `--allow-partial` | Same incomplete view | Empty | 0 |
| Fatal filesystem or cache I/O failure | Empty | `fdu:` error plus preserved cause chain | 1 |
| Argument or value error | Empty | Clap diagnostic and usage | 2 |
| Downstream reader closes stdout | Any prefix already accepted by the reader | Empty | 0 |

`--help` must state the three application outcomes and clarify that Clap also uses exit
status 2 for command-line usage errors.
Scripts consuming JSON can distinguish a partial result from a usage error because a
partial run emits a valid JSON document and a usage error does not.

### Human View

- The summary names the canonical root, descendant file and directory counts, and the
  selected byte measure
- Rows sort by selected size descending and then by name ascending
- `--depth` limits rendered levels without changing scanned totals
- `--number` limits rows independently for each rendered directory
- `--apparent-size` selects apparent bytes for the tree summary, ordering, rows, bars,
  and percentages
- `--by-type` is a human-output view and always reports apparent bytes because the
  current extension reducer does not retain allocated bytes; its summary and rows must
  use the same measure
- `--no-color`, `NO_COLOR`, JSON output, and a non-terminal stdout suppress ANSI output

### JSON View

- `schema`, generator, root, cache source, scan completeness, freshness, errors,
  extension totals, and the selected tree remain deterministic and ordered
- `complete` means every path in the requested *scan scope* was read; it does not claim
  that every retained entry was included in the rendered tree
- The document exposes `display_depth`, `entries_per_directory`, `scan_max_depth`, and a
  `tree_truncated` boolean so consumers can distinguish a complete scan from a limited
  projection
- `tree_truncated` is true when a retained child is omitted by `--depth` or `--number`;
  `--max-depth` is an explicit scan scope and is reported separately
- `by_extension` covers the complete requested scan scope even when the tree view is
  truncated
- `--by-type --json` is rejected rather than silently ignoring `--by-type`; JSON already
  carries both the tree and extension projections
- Invalid Unicode never collapses two retained node identities.
  A node whose name is not valid Unicode retains the lossy `name` for display and adds
  `name_raw` with a documented platform encoding and hexadecimal payload.
  The root has the corresponding optional `root_raw` field

Raw identity fields are omitted when the adjacent string is valid Unicode.
On Unix, `encoding` is `unix-bytes` and `hex` is the lowercase byte sequence returned by
`OsStr`, without a `0x` prefix.
On Windows, `encoding` is `windows-wtf16le` and `hex` is the lowercase little-endian
sequence of raw 16-bit code units, also without a prefix.

The completeness and raw-name fields are additive to `fdu.tree/2`; they do not change
the meaning or type of existing fields.
A future breaking shape change still requires a schema-version bump.

### Cache Lifecycle

- `--no-cache` neither loads nor writes a snapshot and reports `cold_scan`
- The first cached invocation reports `cold_scan` and writes a complete snapshot outside
  the scanned fixture
- An unchanged second invocation reports `warm_revalidate` and returns identical stable
  data
- A file change is detected on the warm path and updates totals before output
- A different `--max-depth` is a different semantic scan scope and misses the prior
  snapshot rather than reusing incompatible data
- A corrupt snapshot is treated as absent, produces a cold scan, and is replaced only
  with a complete result

## Minimal Scenario Set

The suite is organized by state ownership.
Each file gets one sandbox and contains the commands that must share that state.

| Session file | Unique behavior covered | `fdu` invocations |
| --- | --- | ---: |
| `cli-surface.tryscript.md` | Exact help and version; default path on an empty tree; unknown option; missing root; file-as-root; stdout/stderr/exit separation | 6 |
| `cli-human.tryscript.md` | Full apparent-size tree; stable ordering and bars; depth and per-directory number limits; extension grouping; compound and case-folded extensions | 3 |
| `cli-json.tryscript.md` | Full schema; normalized unstable fields; display truncation metadata; scan-depth scope; rejected output-mode conflict | 4 |
| `cli-cache.tryscript.md` | No-cache side effect; cold then warm; warm revalidation after mutation; scope mismatch; corrupt-cache fallback | 6 |

This is deliberately not a Cartesian product of flags.
Each additional invocation must protect a behavior not already visible in another full
output. Boundaries such as zero depth, zero row count, classification details, and
snapshot parser corruption remain in focused Rust tests when adding another complete CLI
transcript would only duplicate the same branch.

### Coverage by Option

| Surface | Golden session or complementary test |
| --- | --- |
| `[PATH]` and default `.` | Surface, human, JSON, and cache sessions |
| `--depth` | Human and JSON sessions |
| `--number` | Human and JSON sessions |
| `--apparent-size` | Human and JSON sessions |
| `--by-type` | Human session and JSON conflict case |
| `--json` | JSON and cache sessions |
| `--no-cache` | All stateless sessions and cache side-effect case |
| `--max-depth` | JSON scope and cache-scope mismatch cases |
| `--no-color` and `NO_COLOR` | Human transcript plus focused color-decision test |
| `--allow-partial` | Unix binary integration test |
| `--help` and `--version` | Surface session |
| Exit 0, 1, and 2 | Surface and Unix partial-result tests |

## Fixture and Normalization Design

### Portable Fixture

One committed fixture represents a small Acorn software project: a Markdown overview, a
working Makefile target, a Markdown FAQ, two Rust source files, and a valid release
archive.
The six regular files occupy exactly 263 apparent bytes under three directories.
The files remain intentionally selected, but their names and contents now explain the
scenario rather than acting as anonymous byte-count tokens.

The project preserves the compact fixture’s behavioral coverage.
`Makefile` is extensionless; `README.md` and `docs/FAQ.MD` test extension aggregation
and case folding; equally sized `src/alpha.rs` and `src/omega.rs` test the name
tie-break; and `dist/acorn-0.1.0.tar.gz` tests compound-extension classification with a
real, deterministic archive.
It contains no symlink, device, permission, sparse-file, or timestamp assumption.
`tests/golden/fixtures/README.md` keeps this rationale adjacent to the corpus without
adding a scanned file.

Every substantive session copies the fixture into `project/` inside its sandbox.
The cache directory is a sibling selected with `XDG_CACHE_HOME=.cache`; it is never
inside the scanned `project/`, avoiding a self-observing snapshot.

### Stable Environment

Each session’s YAML frontmatter declares the environment and patterns it needs.
This keeps the Rust workspace from acquiring a JavaScript configuration file and its
otherwise-required lint and type-check toolchain.
The complete suite sets:

- `NO_COLOR=1` and `FORCE_COLOR=0`
- `LC_ALL=C` and `LANG=C`
- `TZ=UTC`
- `XDG_CACHE_HOME=.cache`
- `PATH` beginning with `$TRYSCRIPT_GIT_ROOT/target/debug`

### Allowed Patterns

| Pattern | Why it is unstable | What remains exact |
| --- | --- | --- |
| `SCAN_PATH` | Sandbox root and path separators change | Surrounding field, names, and all other text |
| `MTIME_NS` | Checkout and fixture-copy times change | JSON key, numeric shape, ordering, and aggregates |
| `ALLOCATED` | Filesystem block accounting differs | Apparent sizes and every non-allocation field |
| `OS_ERROR` | Kernel error wording and numeric suffix differ | Error class, failing path, cause structure, stream, and exit status |

No generic multiline elision is used.
Stable version numbers, schema identifiers, flags, counts, sizes, ordering, percentages,
bars, source labels, booleans, and error prefixes remain literal.

## Complementary Rust Contract Tests

Tryscript owns portable console behavior.
Rust tests own cases where using shell patterns would make the test weaker:

- Keep the synthetic exact JSON golden for all `EntryKind` variants and partial errors
- Keep the Unix unreadable-directory binary test for exit 2 and `--allow-partial`
- Add Unix invalid-byte names and Windows invalid-wide names to prove raw identity
  fields differ even when display strings are lossy
- Refactor color selection behind a small deterministic decision function and cover
  `auto`, explicit off, `NO_COLOR`, JSON, terminal, and non-terminal inputs
- Exercise broken-pipe classification without relying on shell pipeline timing
- Test `tree_truncated` boundaries directly against synthetic indices, including depth
  zero, number zero, exact-fit, and one-entry-over limits

These tests supplement rather than repeat the full transcripts.

## Tooling and Supply Chain

- Pin tryscript exactly at `0.2.0` under the maintainer-approved, expiring first-party
  exception, and pin `tsx`’s `esbuild` edge to the audited `0.28.1` release
- Add a root private `package.json` and committed `package-lock.json`; do not use
  `tryscript@latest` or an unpinned zero-install runner
- Disable npm lifecycle scripts through repository configuration and `npm ci`
- Run `npm audit` after installation and keep the existing Cargo audit
- Use Node 24 in CI and the package’s declared minimum of Node 20 for local development
- Build `target/debug/fdu` before running the transcripts
- Pass the transcript glob in portable double quotes, force golden text to LF, and mark
  the compressed archive as binary so Windows observes the same fixture bytes

The test command performs no network access after the locked npm install.
Tryscript is a development-only tool and does not enter either Rust crate’s dependency
graph or published artifacts.

## Implementation Plan

### Phase 1: Establish and Satisfy the Executable Contract

- [x] Add the locked tryscript toolchain, deterministic scenario configuration, fixture,
  and explicit run/update commands
- [x] Add one failing transcript at a time, then make it pass before adding the next
- [x] Fix the human by-type measure mismatch under its failing transcript
- [x] Reject ignored output-mode combinations under an argument-contract transcript
- [x] Add explicit JSON scan-scope and tree-projection completeness under exact goldens
- [x] Preserve invalid filesystem names in machine output under platform-focused tests
- [x] Make color selection and broken-pipe completion deterministic under focused tests
- [x] Expand `--help` so the golden document is genuinely sufficient for scripts and
  agents, including exit statuses and limit semantics
- [x] Report the blank-stderr, multiple-command, and duplicate-block update hazards
  upstream with minimal reproductions and regression-test suggestions
- [x] Add the cache lifecycle session and fix every discrepancy it exposes before
  accepting output

### Phase 2: Make the Contract a Handoff Gate

- [x] Add `make test-golden` and `make golden-update`; include the former in `make test`
  and `make check`
- [x] Run the golden suite on Linux, macOS, and Windows in CI with a SHA-pinned Node
  setup action and a cache-free locked install
- [x] Document the update-and-review workflow without duplicating tryscript syntax
- [x] Verify that an intentional stable-output mutation fails, that `--update` changes
  only the intended block, and that reverting the mutation restores a clean pass
- [x] Review every golden for unnecessary patterns and every focused test for duplicate
  coverage
- [x] `fdu-12q9`: replace the opaque byte-token corpus with a documented project fixture
  while preserving the same behavioral coverage
- [x] `fdu-ytvu`: upgrade to tryscript 0.2.0, remove released workarounds, and prove
  ordinary and update-mode execution against the exact locked install

## Issue Ledger

Every new discrepancy is added here before its fix.
The linked bead carries execution status and dependencies; this table preserves the
design decision and acceptance behavior.

| Finding | Required behavior | Bead | Status |
| --- | --- | --- | --- |
| The current JSON `complete` field can be true while depth or row limits omit retained entries | Report scan scope and tree truncation independently | `fdu-y0o2` | Fixed |
| Human `--by-type` rows use apparent bytes while the default summary uses allocated bytes | Use apparent bytes consistently for the entire by-type view | `fdu-msbx` | Fixed |
| `--by-type --json` silently ignores `--by-type` | Reject the conflicting modes with a usage error | `fdu-msbx` | Fixed |
| Lossy JSON names can collapse distinct non-UTF-8 filesystem entries | Emit optional raw identity data with a documented platform encoding | `fdu-17to` | Fixed |
| `--help` claims to be complete but omits exit-status and projection-scope semantics | Put those contracts in help and lock the whole output | `fdu-cauc` | Fixed |
| ANSI stripping and non-terminal subprocesses prevent tryscript from proving color behavior | Keep a focused deterministic color-decision contract test | `fdu-qqpt` | Fixed |
| The current binary integration suite covers only the Unix partial-result path | Add four portable end-to-end sessions and retain narrow platform tests | `fdu-ijz4` | Fixed |
| The documented workspace build links the PyO3 `cdylib` directly and fails on macOS outside maturin | Build the core CLI directly and keep maturin as the binding artifact gate | `fdu-f4o2` | Fixed |
| Tryscript cannot express an exact blank stderr line without fragile trailing whitespace | Accept bare `!` as an empty stderr line | `fdu-ms3k` | Resolved and adopted in tryscript 0.2.0 |
| Tryscript concatenates multiple `$` prompts in one fence into one command | Reject the second prompt with a located parse error | `fdu-lz5o` | Resolved and adopted in tryscript 0.2.0 |
| Tryscript `--update` misassigns results among identical raw blocks | Replace blocks by source range | `fdu-hs0l` | Resolved and adopted in tryscript 0.2.0 |
| Tryscript `--update` replaces named unstable patterns with literal values | Preserve matching named patterns while updating stable neighbors; fail the immediate comparison until restored | `fdu-gwe8` | Reported as [tryscript 49](https://github.com/jlevy/tryscript/issues/49) |
| POSIX single quotes in npm scripts become literal on Windows | Quote the transcript glob portably for Unix shells and `cmd.exe` | `fdu-p7wj` | Fixed |
| Git could convert the size-sensitive fixture from LF to CRLF on Windows | Pin every golden input to LF in `.gitattributes` | `fdu-p7wj` | Fixed |
| Anonymous one-character fixture contents make the behavioral intent difficult to review | Use a documented, believable project whose valid contents preserve each required boundary | `fdu-12q9` | Fixed |

## Bead Map

Epic: **fdu-a0w0** — Specify and harden the fdu CLI with golden tests.

| Bead | Work | Blocked by |
| --- | --- | --- |
| `fdu-ijz4` | Locked tryscript harness and deterministic fixture | — |
| `fdu-cauc` | Invocation, errors, help, and human-output goldens | `fdu-ijz4` |
| `fdu-msbx` | By-type measure and output-mode semantics | `fdu-ijz4` |
| `fdu-y0o2` | JSON scan scope and projection completeness | `fdu-ijz4` |
| `fdu-17to` | Lossless non-UTF filesystem identity in JSON | `fdu-y0o2` |
| `fdu-bxbs` | Sequential cache-lifecycle golden session | `fdu-y0o2` |
| `fdu-qqpt` | Partial, color, and broken-pipe contract tests | `fdu-ijz4` |
| `fdu-f4o2` | Core build target separated from the maturin-only PyO3 artifact | — |
| `fdu-ms3k` | Upstream blank-stderr parser issue and local exact workaround | — |
| `fdu-lz5o` | Upstream multi-command parser issue and one-fence-per-command rule | — |
| `fdu-hs0l` | Upstream updater identity bug and distinct-command workaround | — |
| `fdu-gwe8` | Upstream update-mode named-pattern preservation and fail-safe local review | — |
| `fdu-p7wj` | Portable npm glob quoting and fixture line endings on Windows | — |
| `fdu-xuq9` | Make, CI, npm audit, and review workflow | All behavior slices |
| `fdu-12q9` | Self-explanatory project corpus with equivalent exact coverage | — |
| `fdu-ytvu` | Tryscript 0.2.0 upgrade and released-workaround removal | — |

## Acceptance Criteria

1. Exactly four tryscript session files and one substantive portable fixture cover the
   matrix above; additions require a behavior not already visible in an existing output.
2. Every command compares complete stdout, complete stderr, and exit status.
   There are no unknown wildcards and no bare multiline elisions.
3. Running any session twice produces no diff on Linux, macOS, or Windows.
4. All current CLI options and exit outcomes map to a golden or an explicitly named
   complementary test.
5. JSON consumers can distinguish scan completeness, scan scope, and view truncation,
   and can distinguish invalid-Unicode names losslessly.
6. Cold, warm, mutated-warm, scope-miss, no-cache, and corrupt-cache paths are visible
   in the cache transcript and return correct stable totals.
7. `make test`, `make check`, and CI run the golden contract; `make golden-update` is
   the only supported regeneration path and reruns comparison after updating.
   The comparison must reject an update that replaced an approved instability pattern
   with a literal value.
8. Npm dependencies are exact and locked, lifecycle scripts are disabled, and both npm
   and Cargo audits pass.
9. Every finding in the issue ledger has a linked bead and is closed only after its
   failing contract test passes.
10. The final diff contains no generated sandbox/cache artifacts and no unrelated
    changes.

## Completion Evidence

Completed on 2026-08-09. The current local `make check` handoff gate passes, including
145 all-feature library tests, two CLI unit tests, one CLI integration test, two
doctests, 105 core-only tests, 135 watch-only tests, all 25 golden scenarios, both
dependency audits, documentation, two Python concurrency tests, and the installed Python
wheel smoke test.
The golden scenarios pass against the documented 263-byte Acorn project
fixture. The intentional-mutation exercise proved that a stable-output change fails
comparison, that the updater changes only the intended block, and that restoring the
expectation returns the suite to a clean pass.
GitHub Actions [run 31329423861](https://github.com/jlevy/fdu/actions/runs/31329423861)
passes the complete Linux, macOS, Windows, MSRV, documentation, dependency, and wheel
matrix; Windows runs the same 25 golden scenarios against LF-pinned fixtures and
portable npm glob quoting.

The 2026-08-09 maintenance pass upgrades the exact lock to tryscript 0.2.0 and exercises
all 26 current scenarios after a clean `npm ci`. Ordinary comparison and update mode
both pass; update mode leaves every transcript byte-for-byte unchanged.
The online provenance gate verifies the release integrity, its expiring first-party
exception, and the audited `esbuild` override, and `npm audit` reports no
vulnerabilities.

## Rollout Plan

The CLI has not been published, so the contract can be corrected on the existing Phase 1
branch without a compatibility shim.
Once a release is published, exact help and human-output changes remain reviewable
behavior changes, and breaking JSON changes require a new `fdu.tree/N` identifier.
Additive fields remain allowed within a schema version when existing field meanings and
types do not change.

## Backward Compatibility

- **Internal code signatures:** Do not preserve test-only or private rendering helpers
  when a simpler shape makes the contract testable
- **Rust library API:** Maintain; this plan does not require a public library break
- **CLI invocation:** Preserve current flags except the currently silent
  `--by-type --json` combination, which becomes an explicit usage error before release
- **JSON API:** Support existing `fdu.tree/2` fields and meanings; add completeness and
  raw-identity metadata without removing or retyping fields
- **Snapshot file format:** Maintain; tests exercise validity and scope but do not
  change the format
- **Database schemas and server APIs:** Not applicable

## References

- [Golden Testing Guidelines](https://github.com/jlevy/tbd)
- [tryscript](https://github.com/jlevy/tryscript)
- [Tryscript blank stderr issue](https://github.com/jlevy/tryscript/issues/45)
- [Tryscript multiple-command issue](https://github.com/jlevy/tryscript/issues/46)
- [Tryscript duplicate-block update issue](https://github.com/jlevy/tryscript/issues/47)
- [Tryscript named-pattern update issue](https://github.com/jlevy/tryscript/issues/49)
- [Phase 1 plan](../active/plan-2026-08-08-fdu-phase-1.md)
- [File roll-up engine research](../../research/research-2026-08-06-file-rollup-engine.md)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
