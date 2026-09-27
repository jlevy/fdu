# Plan and Tracking Consistency Review

**Date:** 2026-09-27 **Tracking:** `fdu-34sa` **Method:**
`tbd shortcut update-specs-status`, full bead inventory, plan checklist triage, current
source inspection, and PR/CI evidence.

## Scope and Result

The starting inventory contained 1,808 beads, 29 nonclosed epics, 25 active specs, and
96 bead references to missing spec paths.
All dependency targets existed.
Three focused audits covered release/engine, CLI/skills, and performance.
A closed child list was insufficient evidence to close a parent; checklist acceptance
and remaining implementation were checked separately.

The [current work map](../../../TODO.md) now separates implemented changes, merged
changes, publication, and unrecorded verification.
It omits manually maintained bead counts and maps each active workstream to its
governing plan or operational bead.
The [completed-work archive](../../../TODO.archive.md) records completed increments.

## Lifecycle and Ownership Repairs

Six completed increments moved from `active/` to `done/`: analysis/presentation,
inventory/accounting examples, view vocabulary, Linux parallel validation, Linux PGO
screen, and Linux performance iteration.
Their inbound and outbound links and historical bead `spec_path` values were repaired.
Open PRs #133 and #135 still require landing; `fdu-0gqc` owns delivery independently
from implementation completion.

The completed CLI UX and composable CLI plans remain done.
Genuine follow-ups now have an
[active plan](../specs/active/plan-2026-09-27-cli-and-skill-followups.md), owned by
`fdu-pxeb` and `fdu-8e1b`. It preserves the extension percentage column, Windows watch
coverage, cache retention, time policy, watch benchmarks, bead-sync infrastructure,
committed skill, and foreign-repository sdist checks.
Existing ignored-byte tallies do not implement an extension percentage column.

Two deleted interactive-client plans have archive reference pages linking immutable
historical source and their current successor.
Completed historical beads point there.
Open integration (`fdu-p02b`), reducer measurement (`fdu-n4gn`, `fdu-2ig2`), and warm
progressive design (`fdu-eu8t`) point to their current governing plans.
These reference pages do not reinstate superseded engine designs.

All active specs have linked beads; all bead spec paths resolve after reconciliation.
Ownership/path fields changed on 177 existing beads.
The installed tbd version refuses bulk `--spec` and `--parent` updates, so those repairs
used individual supported updates; shared closure reasons used bulk closure.

## Evidence-Backed Closures

| Beads | Evidence / disposition |
| --- | --- |
| `fdu-wgu8` | Permission preflight and explicit unavailable-host opt-out in Makefile, Rust test support, CLI tests and AGENTS.md. |
| `fdu-phdm` | Core exports report planning and performance summary; Python depends directly on core; library-only and MSRV checks exercise the boundary. |
| `fdu-f05m` | Frozen pinned Markdown formatter and mandatory CI format check. |
| `fdu-kixs`, `fdu-w53r` | Duplicate self-check findings: grouped and code reports now request all rows and assert tracked TOML. |
| `fdu-e6f8` | `fdu-o5st` records both crates.io trusted publishers and bootstrap-token removal/revocation; release run `36219577994` proves PyPI OIDC publication. |
| `fdu-ne7h` | Named conflict is obsolete: #85 closed without merge, #87 merged; complete watch wording is present. |
| `fdu-gqsl` | Both named PRs #4 and #11 closed without merge, satisfying the requested disposition. No implementation claim is inferred. |
| `fdu-lmxd` | Commit `16af624f`, merged via #97, supplies claim-only evidence classification and the recycled-buffer guard. |
| `fdu-k3ca`, `fdu-tp2p` | Native-watch precondition and raw-extension documentation exist in the merged tree; composed gate evidence is recorded on `fdu-n2ok`. |
| `fdu-yov0`, `fdu-747k` | Real JSONL parsing and JSON report comparison run in `make test`; all three requested CLI plan lifecycle moves and link repairs are complete. |

Current source closures also have passing 19-job CI on #133 run `36303716655`. That
routine CI run is not evidence of a newly executed full path-independence matrix.
Previously closed `fdu-ives`, `fdu-5e17`, and `fdu-ktyl` were removed from the active
map.

## Work Deliberately Left Open

- **Published-channel verification:** `0.1.0` publication, signed tag, eleven assets,
  docs.rs builds and install smoke are recorded on `fdu-9cf0`. They do not prove every
  first-user exercise.
  `fdu-vxvm`, under `fdu-yfej`, owns the remaining exact checklist.
  `fdu-zx9y` remains the separate registry-propagation retry bug.
- **Alpha acceptance:** merged fixes and final candidate rehearsal are recorded, but
  `fdu-yi1a`, `fdu-j7go`, and `fdu-laeo` retain final-main matrix/evidence
  reconciliation. `fdu-1zb6` remains bounded-memory long-line analysis work.
  The alpha and core-model plans stay active rather than treating all publication as
  proof of every criterion.
- **Performance:** #132’s qualified exploratory comparisons do not satisfy the quiet
  native/wheel release gate (`fdu-ow8y`). `fdu-s234` owns the policy discrepancy between
  that gate and broader claims.
  No gate was waived. PGO adoption (`fdu-pdne`) and cold worker calibration (`fdu-tk1b`)
  remain after their completed screening increments.
- **Missing phase owners:** `fdu-j5k6` now has explicit artifact backfill (`fdu-1dtd`),
  matrix (`fdu-uxl0`), and per-platform report (`fdu-72bn`) tasks.
  Existing FSEvents and evidence-scope tasks now belong to `fdu-7w9a` and `fdu-ug4y`. PR
  #131 remains a probe and design increment, not production historical replay.
- **Other source-confirmed gaps:** `fdu-c5v1` is retitled for the current report/8,
  cache/3 and stream/2 schemas; grouped ignored shares (`fdu-12zs`), ignored control
  reads (`fdu-9jfj`), authoritative golden PATH (`fdu-z7sp`), and warm-read phrasing
  (`fdu-3olz`) remain open.
  Current code does not satisfy their full requested scope.

## Tracking Exceptions

An operational review or handoff does not need an invented design spec.
`fdu-82h4`, `fdu-6wvb`, and `fdu-fhde` retain their actual unresolved review findings.
`fdu-4sg3` has no children but references Linux tasks owned elsewhere; it is an
undecomposed handoff, not an all-children-closed completion candidate.
Narrow standalone bugs and decisions retain their bead descriptions as scope owners.

Open children beneath closed review/implementation increments were retained when their
scope is a separate follow-up.
Reopening historical accepted increments would obscure that distinction.
Their descendants and governing plans remain visible in tbd.
Post-0.1 progressive work already began, so it remains active with explicit deferred
release scope rather than moving to a folder for work never started.

## Tracker Health

`tbd doctor` confirmed valid IDs, hierarchy, dependencies, mappings and sync
consistency. Three historical agent assignees moved to the delegate field.
Managed portable/Claude skills and the AGENTS.md integration were refreshed with
`tbd setup`; the exact npm bootstrap version, tarball and integrity were updated
together to 0.9.0. The existing first-party identity exception applies only to release
age; provenance validation remains mandatory.
No package lock or fdu dependency changed.
The two skill drift warnings reproduce after the required Markdown formatter changes
YAML presentation and prose wrapping.
Parsed frontmatter and whitespace-normalized bodies match the current generated skill;
`fdu-cw36` tracks making the two tools agree without hiding real content changes.
Two lock records that this host could not verify were left untouched; the health check
explicitly advises against removing them without proving their processes have ended.

## Final Review

**Verdict:** Tracking design is consistent after corrections; handoff depends on the
validation below. No fdu runtime behavior changes are part of this layer.
The managed tbd instructions and exact bootstrap provenance pin are refreshed to the
installed first-party 0.9.0 release so the documented tracking commands match the
fallback tool.

- **R1 (Medium, resolved):** The moved view-vocabulary plan still described `fdu-yov0`
  as awaiting reconciliation.
  Both statements now record closure.
- **R2 (Medium, resolved):** The packaging checklist treated publisher configuration and
  token revocation as unverified despite the closed maintainer record.
  The box and duplicate task now cite `fdu-o5st` and the successful PyPI publish run.
- **R3 (Low, resolved):** The new CLI follow-up plan’s usage-guide link used the wrong
  relative depth. The corrected destination passes the link sweep.

**Design assessment:** Separate implementation and delivery ownership avoids reopening
accepted increments or implying that green PR checks publish a release.
Historical reference pages preserve deleted-spec provenance without importing obsolete
designs. A single current-work map avoids conflicting hand-maintained counts.

**Documentation:** The active specs, completed archive, documentation index and
performance orientation pages point to the reconciled owners.
Historical experiment measurements and immutable source links remain unchanged.

**Confirmed benign:** Completed implementation plans may have open merge delivery;
operational review tasks may have no design spec.
Neither case justifies silently closing their remaining acceptance work.

## Validation

- `make check` passed, including 185 golden sessions, CLI/Python parity,
  path-independence subset, library-only and MSRV builds, packaging, release and real
  terminal tests. This is the repository handoff gate, not a new full platform matrix.
- `make supply-chain` passed after the tbd bootstrap pin changed: 79 Cargo packages, 60
  npm packages, 25 Python packages, 67 action uses, and all bootstrap pins verified.
- All spec-local links and changed-document links resolve.
  The repository-wide Markdown sweep leaves only the intentional absent-image fixture
  described above.
- All bead spec paths, parent IDs and dependency targets resolve.
  A bulk comparison verified status, ownership, title, priority, spec path and closure
  reason against `origin/tbd-sync`; the full tracking changes are published separately
  from this PR.
- Independent review findings R1–R3 were corrected; Markdown formatting and staged
  whitespace checks pass.
  PR CI must pass before `fdu-34sa` closes.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
