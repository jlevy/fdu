# Plan: CLI and Agent Skill Follow-Ups

**Date:** 2026-09-27

**Author:** fdu project

**Status:** Active. The original CLI UX, composable query surface, and view vocabulary
are implemented. This plan owns the remaining decisions and acceptance checks described
below; it does not reopen their completed implementation phases.

## Why These Follow-Ups Remain

The [composable CLI plan](../done/plan-2026-08-10-fdu-composable-cli-surface.md) shipped
a six-axis query surface across Rust, the command line, and Python.
The [CLI UX plan](../done/plan-2026-08-09-fdu-cli-ux-and-agent-skill.md) shipped the
portable skill and the wheel’s command entry point.
The
[view vocabulary plan](../done/plan-2026-08-21-fdu-view-vocabulary-and-output-contract.md)
shipped complete `files`, bounded `largest` and `recent`, and parsed machine output.
`fdu --help`, `fdu --docs`, and the [usage guide](../../../usage.md) describe current
behavior; the completed plans retain the reasons for earlier decisions.

The open beads below cover new decisions, platform coverage, and installation checks.
Their acceptance must follow the same surface rule as the delivered CLI: library,
command line, and Python callers get the same answer when a product capability is added.
A decision to leave an optional grammar or flag absent can close a decision bead if the
public contract and tests then agree.

## Query and Output Follow-Ups

Epic `fdu-pxeb` owns the seven items in this section.
Its original five-axis delivery scope is complete; its current work is follow-up
ownership.

| Bead | Current boundary | Acceptance |
| --- | --- | --- |
| `fdu-8nq9` | Extensions have an ignored-byte tally, but their rows still lack the percentage share column used by the other grouped views. | Define the denominator over the full selected population, carry the exact share through the typed report and every machine format with a schema bump, align the text column, and test a truncated section. Keep the ignored tally separate. |
| `fdu-hpbt` | The cross-platform CLI golden drives a real watch on Windows; Rust watch control and persistence integration suites are Unix gated. | Decide which event and persistence expectations hold under ReadDirectoryChangesW. Add Windows-capable real-event integration coverage with meaningful event-service preconditions; retain the existing golden. |
| `fdu-f6dn` | `parse_when` rejects a local civil time and requests an explicit offset or epoch. | Decide whether to keep the offset-required grammar or add time-zone resolution. If added, handle ambiguous and nonexistent daylight-saving times and test them outside a UTC-only environment; if excluded, align grammar and help text. |
| `fdu-g8ks` | `watch-stream` is registered as a benchmark job, but no time-sampled runner executes it. | Define and implement a bounded watch measurement for delivered events, change-to-record latency, idle CPU, startup, and shutdown. Register a reproducible job before making watch performance claims. |
| `fdu-khu8` | Short `--view`, multiple roots, and general grouping remain design questions. | Record a decision for each, including grammar, cache identity, and Rust/Python parity where relevant. Implement only accepted additions; deliberate no-addition decisions are valid. |
| `fdu-558j` | Explicit cache clearing reclaims stale snapshots and leftovers, but valid old snapshots and derived data have no automatic retention or size bound. | Choose and document manual-only, age, or size policy. If automatic, prove a bound and safe treatment of active writers and snapshot/sidecar pairs. Stale-only manual clearing is separate (`fdu-m6lr`). |
| `fdu-qut8` | `make verify-beads` compares all beads with `origin/tbd-sync` on demand and stays outside PR checks because the branch is shared. | Resolve verifier defects under `fdu-f7cf`, then choose a periodic home, failure policy, and shared-branch race behavior. A scheduled check should report actionable metadata drift without making an unrelated PR fail. |

`fdu-qut8` is repository tracking work rather than a CLI behavior change.
It remains listed with its existing epic for continuity; it should not block completion
of the product surface.
`fdu-f7cf`, an independent prerequisite, is resolved: the verifier reads the synced YAML
as tbd writes it and splits the body as tbd reads it, so YAML escapes, empty versus null
`spec_path`, and notes that open with their own `## Notes` heading no longer report
mismatches. The notes case was the verifier’s split, not data loss: tbd splits at the
first such heading and keeps the rest.

`fdu-cw36`, managed tbd skill drift after repository Markdown formatting, is closed as a
tbd defect. tbd 0.9.0 generates the skill in the formatter’s normal form except five
guideline-group notes it writes unwrapped, and `tbd doctor` compares byte for byte, so
the committed skill, which is exactly the formatted output, reads as stale.
The fix belongs in tbd: emit those notes wrapped, or normalize Markdown whitespace
before comparing. Until then that `tbd doctor` warning is expected, and excluding the
skills from the documentation check is not the remedy.
tbd 0.10.0 extends the same warning to the files it adds or refreshes in the formatter’s
path, the tier agents and `AGENTS.md`: flowmark rewraps their prose and makes their
quotes typographic, and they regenerate byte for byte once whitespace and quotes are
normalized.

## Skill Installation Follow-Ups

Epic `fdu-8e1b` owns the installation flow delivered in
[PR #122](https://github.com/jlevy/fdu/pull/122). The wheel-first uv guidance was
completed in [PR #129](https://github.com/jlevy/fdu/pull/129). The two remaining beads
have separate acceptance:

- `fdu-h2bj`: build a rehearsal sdist inside an unrelated scratch Git repository and
  prove its installed `fdu --version` and generated skill name the fdu release, not the
  host repository’s HEAD. Fix the build-version source if the probe fails.
- `fdu-orek`: commit a repository-root `skills/fdu/SKILL.md` discovery copy for
  `npx skills add` and check it against `fdu --skill`, ignoring only the generated
  version line. Keep the CLI-generated skill as the source of truth.

The first public publication and installed-artifact smoke checks remain under the
[release verification plan](plan-2026-09-18-fdu-first-release-verification.md).

## Related Phase 1 Work

`fdu-5ffm` is independent of the two epics above.
A macOS home scan can produce a correct partial roll-up while exiting 2 and printing one
warning per protected path.
Its acceptance is an unattended caller that can distinguish that partial result from a
failed scan without silently weakening the strict partial-result policy.
Coordinate any exit-code decision with `fdu-jej9`, which remains under the
[Phase 1 plan](plan-2026-08-08-fdu-phase-1.md).

`fdu-oqoy` and `fdu-jej9` are also Phase 1 beads, not children of `fdu-pxeb`. Their
older descriptions mix delivered JSONL, `.gitignore`, and native-path behavior with
remaining terminal layout, metric sorting, error-path identity, schema documentation,
and exit semantics.
Reconcile those beads against the current contracts before closing or
moving them; this plan does not duplicate their scope.

## Handoff

For each product change, update the typed core behavior, Python and command-line
surfaces, help and skill text, and the relevant golden or parity evidence.
Apply `make check` and `make cross-lint` when platform-gated code changes.
Close a bead only after its stated acceptance is demonstrated; keep decision records
with no new flag when that is the chosen outcome.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
