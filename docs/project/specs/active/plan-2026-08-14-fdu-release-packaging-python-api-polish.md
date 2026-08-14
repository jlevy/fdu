# Feature: Release Packaging and Python API Polish

**Date:** 2026-08-14 (last updated 2026-08-14)

**Author:** fdu project

**Status:** Implemented for the non-publishing release-engineering scope; registry
publication remains in `fdu-9cf0`

## Overview

Prepare fdu’s public artifacts and Python integration surface for a first release.
The same implementation should install naturally as a Rust crate and command-line tool,
as a PyPI wheel runnable through `uvx`, and as a typed Python library for programs that
need structured directory roll-ups.

The engine and end-user CLI are substantially healthier than the packaging suggests.
The reviewed `main` revision passes its full cross-platform CI matrix; the crate
packages, installs, and works as a dependency outside the workspace; and an isolated
wheel works as both an extension module and an `uvx` command.
At the audited baseline, the repository was nevertheless **not ready to publish**.
Release identity could disagree inside a wheel, license text was absent from both
artifact families, the Linux wheel was tied to a recent glibc baseline, no release
workflow existed, and the Python module did not yet offer a coherent typed contract.

This plan makes those release blockers explicit and resolves the pre-release choices
now, while incompatible cleanup is still inexpensive.

The release design also incorporates a production comparison against Flowmark 0.3.2 and
the current Rust porting playbook.
It adopts the parts that have already worked in the same maintainer’s GitHub, crates.io,
and PyPI accounts, while deliberately diverging where fdu’s mixed Rust/Python library
contract needs stronger guarantees than a binary-only Python distribution.

## Goals

- Publish one coherent `0.1.0` product version as the `fdu` crate and `fdu` PyPI project
- Make `cargo install fdu` and `uvx fdu` expose the same native command-line contract
- Keep the Rust crate usable with minimal dependencies through
  `default-features = false`
- Make `import fdu` expose a typed, documented Python API for reusable structured
  roll-ups
- Preserve provenance, completeness, freshness, scope, errors, and cache semantics
  across Rust, CLI, and Python surfaces
- Build, inspect, and test immutable artifacts before either registry receives them
- Define a repeatable, least-privilege, independently retryable release process
- Establish compatibility, supported-platform, MSRV, deprecation, and security policies

## Non-Goals

- Integrate fdu into metabrowser as part of the first release
- Reproduce metabrowser-specific Git-ignore populations, navigation policy, or UI wire
  format in the fdu core
- Add a network, database, or service API
- Make the initial Python scan nonblocking or expose partially built indexes before the
  blocking batch API is polished
- Add distribution channels beyond crates.io, PyPI, and GitHub release artifacts
- Publish performance claims that have not passed the separate performance evidence
  gates
- Preserve compatibility with the unpublished `fdu_py` import name or raw dictionary
  shapes

## Audit Baseline and Verdict

The audit used `main` at `043e5a7fd2eb556b25c02d046a0e7d8a80c639ad`, fetched from
`origin/main` on 2026-08-14. GitHub Actions
[run 31839766503](https://github.com/jlevy/fdu/actions/runs/31839766503) passed Linux,
macOS, Windows, MSRV 1.85, feature-boundary, documentation, dependency, provenance, and
Python wheel jobs.

The process comparison used fetched, immutable references rather than the local working
branches: Flowmark `015f23989af3e5cfb3f8b58dfc72822c534df25a`, released as `v0.3.2`, and
Rust porting playbook `d24760a3fbd2951c730a199269aeb082abb46a42`. Flowmark’s release was
verified against its successful GitHub Actions run, GitHub release archives and
checksums, crates.io crate, and PyPI wheel and source-distribution matrix.
The playbook review used its stable release, project setup, testing, review, and CLI
rules rather than historical research notes as normative guidance.

Local and artifact-level checks found:

- The required `make check` gate passes: 372 all-feature Rust unit tests, 99 CLI golden
  scenarios, 64 benchmark-contract tests, and 84 real-tree tests pass.
  Rustdoc and pedantic Clippy pass.
- `make cross-lint` checks the host macOS code and passes the installed Windows target;
  the separate macOS cross target is not installed on this macOS host.
- `make audit` passes advisory, ban, license, and source checks.
  It reports only the known duplicate `syn` and `windows-sys` version warnings.
- `cargo publish --dry-run --locked -p fdu --allow-dirty` packages 48 files, verifies
  the extracted crate, and reaches the intentionally aborted upload step.
- The extracted `.crate` installs as a `0.0.1` CLI, and a clean external consumer builds
  and uses `fdu` with `default-features = false`.
- `uv build --no-sources` produces a source distribution and an abi3 wheel.
  The wheel installs on CPython 3.14, imports, scans, reports, and runs through isolated
  `uvx`.
- The packaged CLI completed a content-aware summary and type roll-up over a
  representative 62,769-file downstream tree with `freshness = fresh`,
  `complete = true`, and no errors.
- The `fdu` names returned not-found responses from the crates.io API and the PyPI JSON
  and Simple APIs on 2026-08-14. This is evidence of availability at one instant, not a
  reservation.

### Release Readiness

| Surface | Verdict | Evidence and required disposition |
| --- | --- | --- |
| Rust engine and retained index | Ready for packaging work | Broad unit, model, real-tree, concurrency, and cross-platform tests pass; no engine behavior change is required by this plan. |
| Rust library artifact | Conditional pass | The extracted crate builds and works with default features disabled, but it lacks license text and needs a post-`0.1.0` semver baseline. |
| Native CLI | Conditional pass | Help, machine output, error behavior, golden sessions, and a real downstream scan work; existing Phase 1 CLI, watch, and performance release gates still apply. |
| Wheel console script and `uvx` | Conditional pass | Isolated execution works, but a wheel built from a Git checkout reports a development CLI version and the current platform matrix is too narrow. |
| Python library | Blocked | Runtime capabilities exist, but the import name, class metadata, type information, result models, and status semantics are not a supportable public contract. |
| Artifact metadata and contents | Blocked | License files are absent; Python metadata is sparse; source-distribution and wheel contents are not release-gated. |
| Release automation | Blocked | There is no build-once publication workflow, artifact attestation gate, release rehearsal, or documented recovery path. |
| Registry names | Available when checked | Re-verify immediately before first publication because neither registry reserves an unpublished name. |

The release verdict is therefore **not ready**, despite a strong implementation and test
baseline. The blockers are bounded and do not call for a rewrite.

## Blocking Findings

### R1: One Wheel Can Contain Two Product Versions

The distribution metadata and extension module’s `__version__` both report `0.0.1`, but
the console script in a wheel built directly from the reviewed checkout reports
`fdu 0.0.1-dev+g043e5a7fd`. The build script adds a development suffix whenever it can
find any enclosing Git repository, including an exact release-tag checkout.

A wheel built from the source distribution happens to report the bare version because
Git metadata is absent.
Release correctness must not depend on that accidental build location.
The tagged checkout, crate metadata, Python metadata, module version, CLI version,
report generator, source distribution, wheels, and GitHub release must all be checked
against the same exact version.

### R2: The Python Distribution and Runtime Disagree About Their Name

PyPI metadata calls the project `fdu`, but users can only `import fdu_py`. At the same
time, `Index.__module__` is `fdu`, even though that module does not exist, while
`Watch.__module__` is `fdu_py`. This produces confusing representations, documentation,
pickling behavior, and type-checker diagnostics.

The public Python package should be `fdu`. The compiled extension should be an
explicitly private implementation module, such as `fdu._native`, behind a small Python
package that owns exports, annotations, value models, documentation, and compatibility
policy.

### R3: The Extension Has No Usable Static Type Contract

The wheel contains neither `.pyi` stubs nor a `py.typed` marker.
A strict BasedPyright consumer reports a missing stub and treats `Index`, `scan`,
`report`, and their returned values as unknown.
This is especially costly because the API returns nested dictionaries whose field names
and variants are otherwise undiscoverable.

Extension-module stubs are required for the native surface.
The public package also needs inline annotations or stubs, a `py.typed` marker,
runtime-to-stub export parity, and a strict downstream type-checking fixture.

### R4: Completeness and Freshness Are Conflated

`Index.complete` currently requires both a complete operation and `fresh` currency.
`Index.report()["complete"]` reports operation coverage alone.
The existing wheel smoke test consequently requires a cache-only index to have
`index.complete is False` while its report says `complete is True` and its freshness
says `stale`.

Completeness answers whether the requested scope was covered.
Freshness answers whether that complete information was verified against the filesystem
now. They must remain independent in every language binding, as they already are in the
machine report.

### R5: Python Exposes Stringly, Partial Parity

`Index.report()` is a useful bulk boundary, but its only public model is a nested
mutable dictionary assembled by hand.
Errors are flattened to strings.
`total()`, `rollup()`, and `children()` do not expose provenance.
Python cannot request `one_filesystem`, even though it is part of Rust `ScanConfig` and
the CLI scope. `watch()` exposes fewer selection axes than `report()`, and Python does
not expose typed equivalents of the Rust `Query`, `Selection`, `ViewSpec`, cache,
freshness, source, or analysis concepts.

This violates the project’s rule that the CLI invents no concepts of its own and makes
downstream programs reconstruct contracts from prose.
The first public Python API needs typed inputs, immutable typed results, structured
operational errors, and mechanical parity tests.

### R6: Published Artifacts Would Omit License Text

The `.crate`, source distribution, and wheel metadata declare MIT, but inspected
archives contain no license file.
The wheel also uses the legacy text-style license metadata and lacks authors, keywords,
Python classifiers, operating-system classifiers, and a typing classifier.
Artifact inspection must require license and README contents, modern license metadata,
and a complete discovery surface.

### R7: The Current Linux Wheel Is Not Broadly Portable

The successful Linux CI jobs build `manylinux_2_34_x86_64` wheels on the hosted runner.
That glibc floor excludes still-common systems that a native `uvx` tool should support.
CI also produces only Linux x86-64, Windows x86-64, and macOS arm64 wheels; running the
same abi3 wheel under Python 3.12 and 3.14 does not add architectures.

Linux release wheels must be built in a controlled manylinux environment rather than on
the host runner. The first release matrix is defined below and unsupported targets must
be stated honestly.

### R8: CI Tests Checkout Builds, Not a Release Pipeline

The repository has no tag or release workflow.
Current Python jobs build directly from the checkout, do not upload artifacts for later
jobs, do not build or install the source distribution, and do not compare the Cargo and
wheel CLIs. There are no Git tags or GitHub releases.

Publication must consume already tested artifacts.
PyPI supports a pending trusted publisher for a first project release.
crates.io trusted publishing requires the first crate version to be published manually
before the publisher can be configured, so the bootstrap procedure cannot honestly be
described as fully secretless.

### R9: Release-Facing Documentation Has Drifted

The changelog says the wheel omits the optional watch dependency, while the wheel
enables and documents it.
Rust API docs call only `cli` a default feature, while the manifest defaults to both
`cli` and `watch`. Installation examples still use `fdu_py` and the longer
`uvx --from ...` form.
These are small defects, but they show why packaged README, help, metadata, and runtime
assertions belong in the release gate.

## Flowmark and Rust Porting Playbook Review

### What the References Prove

Flowmark 0.3.2 demonstrates that the same public maintainer identity intended for fdu
can publish one Rust version through GitHub Releases, crates.io, and PyPI. Its shipped
PyPI matrix includes Linux x86-64 and arm64, macOS x86-64 and arm64, Windows x86-64, and
a source distribution.
Its release automation also establishes useful operating patterns: a dry-run entry
point, independently retryable channels, checked-in release scripts with unit tests,
registry-state probes, native artifact smoke tests, target- and version-qualified
archives, and a checksum manifest.

The current Rust porting playbook independently supports the architectural parts of that
process: one release identity, explicit feature boundaries, MSRV and
`--no-default-features` checks, artifact-first validation, per-job permissions, trusted
publishing, explicit channel audiences, rerun safety, source-distribution testing, and
runbook-driven partial-failure recovery.

### Practices Adopted for fdu

| Reference practice | fdu adaptation |
| --- | --- |
| Cargo metadata is the product-version authority | Keep the workspace version authoritative and derive Python metadata, the extension version, CLI identity, filenames, and release notes from it. |
| One release orchestrator owns planning, building, publishing, and announcement | Use one top-level `release.yml` with narrow jobs and a non-uploading dispatch mode. |
| Cross-platform Maturin builds produce a predictable wheel set plus an sdist | Use the same five native architecture/OS combinations, but build abi3 extension wheels and install-test the sdist. |
| Complex release decisions live in tested scripts | Check in small scripts for release-plan resolution, artifact inspection, registry-state comparison, and archive assembly, with fixture-based unit tests. |
| Published channels are independently retryable | Probe each registry, skip only an identical published artifact, and resume the missing channel without retagging. |
| Native archives are smoke-tested and checksummed | Exercise each supported binary or wheel natively where possible and publish one deterministic checksum manifest. |
| The crate separates CLI conveniences from its core library | Preserve fdu’s default-feature and core-only consumer gates, which are already stronger than the reference release. |
| The runbook covers dry runs and partial publication | Rehearse both the happy path and one-channel-already-published recovery before `0.1.0`. |

fdu retains its existing exact action revisions, dependency cool-off, artifact SBOM,
feature matrix, and `make check` handoff gate.
Workflow or tool versions from Flowmark are examples of structure, not dependency pins
to copy.

### Deliberate Divergences

- Flowmark’s PyPI project is a binary-only distribution with a different distribution
  name and two console scripts.
  fdu uses one `fdu` identity and a mixed project with a private `fdu._native` extension
  behind a typed public package.
- Flowmark’s wheels are `py3-none` binaries.
  fdu’s wheels are abi3 extension artifacts with stubs, `py.typed`, immutable public
  models, structured errors, and CLI/API parity tests.
- Flowmark’s release profile aborts on panic.
  fdu must retain unwind behavior at the Python boundary so PyO3 can convert a Rust
  panic into an exception rather than terminating the host interpreter.
- fdu does not add Homebrew or another distribution channel to `0.1.0`. The initial
  crate, PyPI, `uvx`, and GitHub artifacts already cover the agreed audiences.
- A cross-built artifact that cannot run on its build host is not counted as fully
  smoke-tested. The matrix records native, emulated, or structural evidence explicitly
  rather than treating a skipped architecture as a pass.
- fdu installs and tests its source distribution because it advertises source builds as
  the fallback for unsupported wheel platforms.

### Publication Ownership and Credentials

Use the same public maintainer accounts and ownership model as Flowmark, but create
project-specific trusted-publisher records.
Account reuse establishes continuity of ownership; it does not mean copying a token or
sharing one project’s publisher subject with another project.

| Channel | fdu setup |
| --- | --- |
| GitHub Releases | Publish from `jlevy/fdu` with the workflow’s built-in token, granting `contents: write` only to the announcement job. |
| PyPI | Create the pending `fdu` project for repository `jlevy/fdu`, top-level workflow `release.yml`, and protected environment `release`. The publish job receives `id-token: write`, downloads only validated artifacts, and holds no API token. |
| crates.io bootstrap | Publish `fdu 0.1.0` from the same crates.io owner account with a narrowly scoped, short-lived token because trusted publishing cannot be configured until the crate exists. Remove the token after verifying the release. |
| crates.io steady state | Configure the new `fdu` crate’s trusted publisher for `jlevy/fdu`, `release.yml`, and environment `release`; exchange OIDC only inside the publish job. |

No credential is committed, inherited across workflows, printed, or retained in an
artifact. The protected environment separates approval from build jobs.
PyPI currently matches trusted-publisher identity to the top-level workflow filename and
does not allow a reusable workflow to be registered as that identity, so the OIDC
publish job stays directly in `release.yml` even if non-credentialed build logic is
reusable.

### Upstream Improvement Candidates

The fdu implementation should validate these findings before proposing changes upstream.
They are tracked in `fdu-8bn9` and do not block fdu `0.1.0` unless the same defect is
copied locally.

| Target | Class | Finding and proposed improvement |
| --- | --- | --- |
| Flowmark | FIX — high | Split source checkout, third-party tool installation, build, and validation away from the job holding `id-token: write`; pin actions and installed tools to reviewed immutable versions. |
| Flowmark | FIX — high | Derive the Python parity package version from Cargo metadata instead of keeping the release workflow’s stale `0.7.0` beside manifest `0.7.2`. |
| Flowmark | ADD — high | Make release-plan resolution reject a tag that differs from `v{Cargo version}` or does not identify the release commit. |
| Flowmark | ADD — medium | Compare local artifact hashes and metadata with an existing registry version; skip an identical version but fail on a conflicting immutable version. |
| Flowmark | VALIDATE — medium | Install-test the sdist, record evidence for cross-built wheels, and reconcile the PyPI publisher runbook with the top-level workflow that actually publishes. |
| Rust porting playbook | ADD — high | Add a mixed PyO3 package pattern covering a private extension, typed facade, stubs, `py.typed`, abi3 floor, structured FFI errors, source builds, and unwind-safe profiles. |
| Rust porting playbook | CLARIFY — high | State that trusted-publisher owner, repository, workflow, and environment records are project-specific, and document PyPI’s current reusable-workflow limitation. |
| Rust porting playbook | ADD — medium | Supply executable templates for tag/manifest identity checks, artifact-manifest inspection, and registry hash conflict detection; clarify Cargo’s repackaging exception to literal build-once promotion. |

Historical playbook research that recommends `bindings = "bin"` remains valid for a
binary-only port such as Flowmark, but should route mixed library consumers to the new
pattern instead of appearing universal.

## Design Decisions

### Names, Versions, and Public Artifacts

The first release is `0.1.0`, not another `0.0.x` snapshot.

| Role | Public name |
| --- | --- |
| crates.io package and Rust crate | `fdu` |
| Cargo-installed binary | `fdu` |
| PyPI distribution | `fdu` |
| Python import package | `fdu` |
| Wheel console script | `fdu` |
| Compiled Python extension | `fdu._native` (private) |
| Release tag | `v0.1.0` |

The workspace manifest version is the source version during ordinary development.
A release-preparation command updates all workspace package metadata and the changelog.
The release workflow rejects a tag unless it exactly equals `v` plus that version and
the checkout is clean.
Development builds retain revision information, but an exact matching release tag
reports only the public version.
Every artifact-level smoke test checks all version surfaces rather than trusting
filenames.

`fdu-py` remains `publish = false` on crates.io.
It is build machinery for the PyPI artifact, not a second Rust library.

### Rust Packaging Contract

The crates.io package supports two deliberate consumption modes:

```shell
# End-user CLI; default features include CLI and watch support.
cargo install fdu --version 0.1.0
```

```toml
# Embedded library; no CLI or watcher dependency tree.
[dependencies]
fdu = { version = "0.1.0", default-features = false }
```

The `.crate` must contain the exact license text, crate README, rules used by the build
script, source required by every advertised feature, and the minimized lockfile Cargo
generates. It must build with default features, no default features, and watch-only
features at the declared MSRV where applicable.
An external fixture must compile against the extracted artifact, not the workspace.

`cargo-semver-checks` starts from the published `0.1.0` baseline.
Before that baseline, an explicit public-item inventory and rustdoc gate protect against
accidentally widening the already reviewed Rust surface.

### Python Package Layout

Use maturin’s mixed Rust/Python layout:

```text
crates/fdu-py/
  python/fdu/
    __init__.py
    _models.py
    _native.pyi
    py.typed
  src/lib.rs
  pyproject.toml
```

`__init__.py` re-exports the supported surface and nothing else.
The console script calls a public-package wrapper that delegates immediately to the
native CLI boundary.
All native classes report a consistent module name.
Importing `fdu_py` is not retained as a compatibility alias because no release has
promised it.

Use current PEP 639 license metadata, include license text in both source distributions
and wheels, and add authors, keywords, Python version classifiers, supported operating
systems, and `Typing :: Typed`. The wheel should retain maturin’s generated CycloneDX
SBOM.

### Python API Contract for `0.1`

The supported API stays bulk-oriented.
The native boundary returns a report or child collection in one call; it never makes
Python perform one extension call per indexed entry.
Public configuration and result objects are frozen, slotted value types or enums, with
precise stubs for the private extension.

The initial public inventory is:

- `open()` and `scan()`
- `Index`, including `root`, `clock`, `status`, `total()`, `rollup()`, `children()`,
  `report()`, `refresh()`, `since()`, and `watch()`
- `ScanOptions`, `AnalysisOptions`, `Query`, `Selection`, and `WatchOptions`
- `CachePolicy`, `Freshness`, `ReportSource`, `View`, `EntryKind`, `SizeMetric`, and
  `SortKey`
- immutable report, section, tree, metric, tally, change, provenance, and error records
- cache-path, status, list, and clear functions
- `FduError` plus structured fatal and partial-operation error information

A representative call should read as follows:

```python
from pathlib import Path

import fdu

index = fdu.open(
    Path("."),
    cache=fdu.CachePolicy.OFF,
    scan=fdu.ScanOptions(one_filesystem=True),
)
report = index.report(
    fdu.Query(
        views=(fdu.View.SUMMARY, fdu.View.TYPES),
        selection=fdu.Selection(limit=10, size=fdu.SizeMetric.APPARENT),
    )
)

if not report.status.complete:
    for error in report.status.errors:
        print(error.path, error.kind, error.message)

for section in report.sections:
    print(section)
```

`Report.status.complete` means requested-scope coverage only.
`Report.status.freshness` independently describes currency.
`Index.status` uses the same type and semantics.
A cache-only result may therefore be complete and stale without contradiction.

Report sections form a discriminated union keyed by `view`. Tree, file, metric, and
summary rows are named immutable records.
`Report.as_dict()` emits the exact current CLI machine schema, including `schema` and
`generator`, so serialization-oriented callers do not depend on a second Python-only
shape. The native CLI and Python dictionary conversion must compare equal after
normalizing timestamps and platform-native path encodings.

`Index.total()`, `rollup()`, and `children()` return named records that include or link
to their provenance.
`Index.provenance(path)` is public for callers that need the retained entry’s
observation details.
Operational errors retain path, category, message, and OS error code when present; a
fatal invalid argument remains an exception, while a partial scan remains useful data.

Every CLI scope, selection, view, cache, and analysis value that affects engine behavior
has one Python representation.
Output-format and terminal-color choices remain CLI-only.
Parity is checked mechanically, including defaults and accepted enum vocabulary.

### Downstream and Metabrowser-Shaped Use

The public metabrowser project demonstrates four useful consumer properties:

- strict type checking and an explicit wire-shape contract;
- one retained inventory serving repeated bounded roll-ups;
- explicit scanning, done, truncated, and failed states;
- bounded trees whose omitted children are conserved in aggregate rest buckets.

The `0.1` API satisfies the first two through typed bulk reports over one retained index
and makes partial/error state explicit.
A downstream adapter can map fdu’s immutable records into its own wire models without
parsing terminal output.

Two capabilities stay in a follow-on phase because they are not required for a sound
first release:

1. a progressive `IndexSession` that can serve a stale snapshot immediately, reconcile
   in the background, expose status transitions, and emit authoritative resync signals;
2. generic bounded-tree omitted aggregates when a consumer needs a precomputed `rest`
   bucket rather than deriving display policy from parent and child roll-ups.

Metabrowser’s simultaneous all-files and Git-ignored populations remain client-specific
until more than one consumer demonstrates the reducer belongs in fdu.
The first release must not imply Git-ignore semantics it does not implement.

### Supported Python Artifact Matrix

One abi3 extension targets CPython 3.12 and newer.
Test the oldest supported interpreter and the current stable interpreter; intermediate
CPython versions consume the same ABI artifact.

The first-release binary matrix is:

| Platform | Architectures | Wheel policy |
| --- | --- | --- |
| Linux glibc | x86-64, arm64 | `manylinux2014` / glibc 2.17 baseline in controlled manylinux builders |
| macOS | x86-64, arm64 | Separate native wheels with a tested and documented macOS 11.0 deployment floor |
| Windows | x86-64 | Native MSVC wheel |

Windows arm64 and musllinux x86-64/arm64 are stretch artifacts, not silent promises.
Unsupported systems may build from the source distribution with a compatible Rust
toolchain, but documentation must distinguish that fallback from a zero-build `uvx`
experience.

### Release Pipeline

Use one top-level workflow, one tag, and independently retryable registry jobs.
Registry uploads cannot be atomic, and published versions cannot be replaced, so the
workflow models partial success instead of hiding it.

```text
resolve tag, commit, Cargo version, mode, and expected artifact manifest
                              |
                              v
package crate preview + sdist + wheels + native archives
                              |
                              v
inspect, install, type-check, smoke, compare, checksum, and attest
                              |
                              v
                    protected approval
                           /       \
                          v         v
              crates.io publish   PyPI promote
                           \       /
                            v     v
                    verify registry hashes
                              |
                              v
              GitHub release + retained evidence
```

`workflow_dispatch` runs the same graph in a non-uploading rehearsal mode; a release tag
runs publication mode.
The resolver rejects a dirty or non-tagged source, a tag that differs from
`v{Cargo version}`, a tag that does not identify the workflow commit, an unexpected
package version, or an incomplete artifact matrix.

The build stage uploads immutable workflow artifacts and a machine-readable manifest.
Wheel and source-distribution smoke and publication jobs download those exact files and
verify checksums; they never rebuild them.
Cargo’s supported `cargo publish` command does not accept a prebuilt `.crate`, so its
narrow publish job is the documented exception: it downloads the exact tagged source
bundle, reproduces the package, checks that its checksum equals the validated preview,
publishes it, and then checks the registry checksum.
A mismatch is a release failure, not a reason to rebuild from a different source.

Registry probes classify each expected version as `missing`, `identical`, or `conflict`.
A retry uploads only `missing` channels, safely skips `identical` ones, and stops on
`conflict`. Merely observing that a version number exists is not enough.

The top-level workflow is intentionally thin:

```text
.github/workflows/release.yml
scripts/release/
  resolve_plan.py
  inspect_artifacts.py
  registry_state.py
  package_archives.py
tests/release/
  test_resolve_plan.py
  test_inspect_artifacts.py
  test_registry_state.py
  test_package_archives.py
```

Decision logic belongs in these checked-in, fixture-tested scripts rather than inline
shell or YAML expressions.
The scripts use machine-readable registry APIs, reject unknown fields or missing
artifacts, and produce summaries suitable for retained release evidence.

Build and validation jobs have no publication authority.
The protected `release` environment approves the two narrow publisher jobs.
The PyPI job has no source checkout and only downloads verified files before its OIDC
upload.
The crates.io job uses only the exact source bundle and Cargo tooling required to
publish. The GitHub announcement job alone receives `contents: write`. Every action is
pinned to a reviewed commit, every installed tool is pinned through the repository’s
supply-chain policy, and no job persists checkout credentials.

PyPI uses a pending trusted publisher for the first release.
The crates.io bootstrap uses a narrowly scoped short-lived API token for `0.1.0`, then
configures the repository’s trusted publisher for later releases.
Both records use the same owner account as Flowmark but identify the `jlevy/fdu`
repository, top-level `release.yml`, and protected `release` environment.
The runbook records the unavoidable asymmetric case where one registry succeeds and the
other fails: verify the successful checksum, resume only the missing upload, and never
retag or overwrite the released version.

## Implementation Plan

The implementation branch completes the package, API, artifact, test, read-only
registry-audit, workflow-rehearsal, and runbook work authorized before publication.
It deliberately contains no registry credentials, publisher configuration, release tag,
upload, or GitHub Release operation.
Those irreversible steps remain in `fdu-9cf0`. The progressive `IndexSession` and
upstream-reference improvements also remain separate follow-up work.

### Tracked Work

| Bead | Scope | Release relationship |
| --- | --- | --- |
| `fdu-3d8c` | Packaging and Python API polish epic | Blocks final publication |
| `fdu-2orl` | Release audit and this plan | Closes when the reviewed plan and evidence land |
| `fdu-vhbz` | Flowmark and Rust porting playbook comparison | Closes when the adopted practices, divergences, credential model, and upstream findings land in this plan |
| `fdu-t5lh` | Typed `fdu` Python package and roll-up API | Release blocker |
| `fdu-8d28` | Version identity, license, metadata, and documentation exactness | Release blocker |
| `fdu-5eqk` | Portable abi3 matrix and artifact-first release workflow | Depends on API layout and artifact identity |
| `fdu-wp21` | Installed-consumer, CLI parity, typing, and downstream acceptance | Depends on API layout and artifact identity |
| `fdu-lidi` | Policy, rehearsal, and first-release evidence | Depends on workflow and acceptance gates |
| `fdu-9cf0` | Existing final crates.io and PyPI publication gate | Depends on `fdu-3d8c` plus its earlier Phase 1 blockers |
| `fdu-eu8t` | Progressive Python `IndexSession` | Post-release; does not block `0.1.0` |
| `fdu-8bn9` | Upstream validated release-hardening findings | Post-release; does not block `0.1.0` |

### Phase 0: Track and Freeze the Public Contract

- [x] Link the existing publishing bead and all implementation beads to this spec
- [x] Record the Python `0.1` export baseline and installed runtime/stub parity
- [x] Add a machine-readable or tested parity inventory for engine-facing options and
  default values
- [ ] Resolve the existing CLI, agent-schema, watch-hardening, and performance blockers
  already attached to `fdu-9cf0`

### Phase 1: Typed Python Package and Roll-Up API

- [x] Move the public import to `fdu` and the extension to `fdu._native`
- [x] Add the pure-Python package, private extension stubs, `py.typed`, and
  runtime-export parity test
- [x] Add typed options, enums, immutable result records, structured partial errors, and
  exact report-to-dictionary conversion
- [x] Separate completeness from freshness and use the same status model on indexes,
  reports, refresh results, and cache-only answers
- [x] Add missing `one_filesystem`, provenance, report-schema, and watch-query parity
- [x] Document a one-scan/many-roll-ups example and a downstream adapter example

### Phase 2: Artifact Identity, Contents, and Compatibility

- [x] Move the workspace to `0.1.0` and make exact-tag builds report the public version
- [x] Include license text and correct metadata in the crate, source distribution, and
  every wheel
- [x] Correct feature, watch, import, version, and installation documentation drift
- [x] Build and install the crate and source distribution from extracted artifacts
- [x] Establish compatibility, supported-platform, MSRV-change, deprecation, security,
  and incident-response documentation
- [x] Capture a Python public API baseline for post-`0.1.0` compatibility checks

### Phase 3: Artifact-First CI and Release Automation

- [x] Add tested release-plan, artifact-inspection, registry-state, manifest, and
  checksum tooling; keep release decisions out of inline YAML
- [x] Make `release.yml` reject tag, commit, Cargo, Python, runtime, and filename
  identity disagreement before building
- [x] Define and validate `manylinux2014` x86-64 and arm64, macOS x86-64 and arm64, and
  Windows x86-64 abi3 wheels
- [x] Upload and reuse immutable artifacts between build, smoke, and evidence jobs
- [ ] Compare Cargo and wheel CLIs over representative golden and partial-result
  sessions
- [x] Test CPython 3.12 and the current stable interpreter against installed wheels
- [x] Classify registry state as missing, identical, or conflicting by artifact hash and
  metadata before every upload or retry
- [x] Pin every action and installed release tool immutably and keep publication
  authority out of build and validation jobs
- [ ] Add direct, minimal, protected PyPI trusted publishing and the documented
  crates.io bootstrap path under the same maintainer accounts as Flowmark
- [x] Emit checksums and SBOM evidence from the non-publishing workflow
- [ ] Add attestations and a GitHub release only after registry state is verified

### Phase 4: First-Release Rehearsal and Publication

- [ ] Re-verify registry names through authoritative APIs immediately before release
- [ ] Configure the PyPI pending publisher for `jlevy/fdu`, `release.yml`, and the
  protected `release` environment
- [ ] Run the workflow’s non-uploading mode from the exact proposed tag and rehearse an
  identical-channel retry and a simulated conflicting-channel stop
- [ ] Inspect every archive and execute every supported install path
- [ ] Publish `0.1.0`, verify registry metadata and fresh-user installs, and retain
  release evidence
- [ ] Remove the one-time crates.io bootstrap token, then configure trusted publishing
  for `jlevy/fdu`, `release.yml`, and the `release` environment

### Phase 5: Progressive Downstream Adapter

- [ ] Specify `IndexSession` status, stale-first serving, reconciliation, cancellation,
  and resync semantics before exposing background work to Python
- [ ] Prototype a metabrowser-shaped adapter over immutable fdu reports and deltas
- [ ] Add bounded-tree omitted aggregates only if the adapter proves they belong in the
  engine rather than presentation policy
- [ ] Measure conversion and retained-memory cost before promising progressive results

## Testing Strategy

### Repository Gates

- `make check`
- `make cross-lint` on every installed non-host target after platform-gated changes
- `make audit`
- clean working-tree and exact-tag verification

### Rust Artifact Gates

- `cargo publish --dry-run --locked -p fdu`
- exact archive manifest, license, README, and unexpected-file assertions
- build, test where applicable, and `cargo install` from the extracted `.crate`
- compile a minimal external library consumer with `default-features = false`
- verify default, core-only, and watch-only feature behavior at the declared MSRV
- verify `fdu --version` and machine-report generator identity

### Python Artifact Gates

- `uv build --no-sources` and maturin source-distribution completeness verification
- archive manifest, metadata, license, SBOM, stub, and `py.typed` assertions
- install only the wheel into a fresh CPython 3.12 and current-stable environment
- install only the source distribution in a clean environment with a Rust toolchain
- run `uvx --isolated fdu@0.1.0 --version` and representative CLI sessions
- run a strict BasedPyright consumer against the installed distribution
- assert `import fdu`, public exports, class module names, signatures, and version
  identity

### API and Downstream Gates

The acceptance fixture builds a small tree containing files, directories, a symlink,
non-Unicode names where supported, content-analysis coverage cases, and an unreadable or
racing path. It then:

1. opens one index and requests summary, tree, type, family, language, document, and
   file views without rescanning;
2. verifies typed access, stable ordering, scope, provenance, complete/partial state,
   and structured errors;
3. compares `Report.as_dict()` with native CLI JSON after normalization;
4. tests cache-only `complete + stale` semantics and refresh convergence;
5. tests delta truncation, watch cleanup, GIL release, and same-object borrow behavior;
6. demonstrates a bounded downstream roll-up transformation without parsing CLI text.

### Publication Gates

- trusted-publisher owner, repository, top-level workflow, protected environment, and
  workflow-permission assertions
- no mutable action tags, unpinned tool installs, inherited secrets, or source checkout
  in the PyPI publisher job
- artifact checksums unchanged between build, test, and upload jobs
- crates.io package preview checksum reproduced from the exact source bundle and matched
  against the published registry checksum
- registry-state tests distinguish missing, identical, and conflicting immutable
  versions
- TestPyPI or equivalent install rehearsal for the Python artifact
- crates.io dry run and manual first-release bootstrap checklist
- fresh registry installs, docs.rs build, PyPI metadata, and `uvx` verification after
  publication
- documented retry and incident paths exercised without overwriting a version

## Rollout and Compatibility

No compatibility shim is required for the unpublished `fdu_py` package.
Release notes should call the new `fdu` import the first supported Python surface, not a
rename of a previously supported API.

Rust follows SemVer from `0.1.0`; during `0.x`, minor releases may contain breaking API
changes, but each must be documented and checked.
Python uses the same product version and gives at least one documented deprecation cycle
for supported symbol removal unless a security or correctness defect makes that unsafe.
Machine report schemas retain their own explicit version and do not change merely
because the package version changes.

The existing publishing bead remains the final release gate.
This spec refines its packaging and Python API prerequisites; it does not bypass its
outstanding Phase 1 CLI, agent-schema, watch, or performance dependencies.

## Risks and Mitigations

| Risk | Mitigation |
| --- | --- |
| A pure-Python facade adds conversion overhead | Keep the boundary bulk, use frozen slotted records, benchmark representative large reports, and avoid converting the retained index itself. |
| Python models drift from Rust and CLI concepts | Generate or mechanically compare enum vocabulary, defaults, schemas, and normalized fixture output. |
| The platform matrix becomes expensive | Build one abi3 wheel per OS/architecture rather than per Python minor; smoke the oldest and current stable interpreters. |
| One registry publishes while the other fails | Use independent idempotent jobs, immutable versions, checksum verification, and a documented resume path. |
| A retry silently accepts a different artifact under the same version | Compare registry hashes and metadata; treat existence with disagreement as an unrecoverable conflict requiring a new version. |
| Publication authority leaks into a build step | Put OIDC or the one-time bootstrap token only in protected, single-purpose publisher jobs and install no mutable tools there. |
| PyPI publisher configuration points at a reusable workflow | Register the top-level `release.yml` identity and keep its minimal publish job direct until PyPI documents support for reusable workflow identities. |
| Cargo cannot promote a prebuilt `.crate` through `cargo publish` | Reproduce it from the exact source bundle, compare with the validated preview, and verify the resulting registry checksum. |
| A package name is claimed before release | Re-check through authoritative APIs at the protected approval boundary; stop rather than silently renaming one ecosystem. |
| Progressive serving leaks concurrency complexity into `0.1` | Ship the blocking typed API first and specify `IndexSession` separately against the existing Rust ownership rules. |
| A downstream-specific reducer bloats the core | Keep Git-ignore and UI ranking policy in adapters until repeated consumers justify a general engine concept. |
| A reference repository changes after this review | Keep immutable reference commits in the plan and re-evaluate newer practices deliberately rather than copying a moving branch. |

## References

- [fdu design principles](../../architecture/fdu-design-principles.md)
- [Phase 1 plan](plan-2026-08-08-fdu-phase-1.md)
- [Rust engineering quality plan](plan-2026-08-09-fdu-rust-engineering-quality.md)
- [Composable CLI surface plan](plan-2026-08-10-fdu-composable-cli-surface.md)
- `fdu-9cf0`: existing crates.io and PyPI publishing bead
- [Cargo publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [maturin mixed-project layout](https://www.maturin.rs/index.html#mixed-rustpython-projects)
- [maturin distribution and manylinux guidance](https://www.maturin.rs/distribution.html)
- [uv tool execution](https://docs.astral.sh/uv/guides/tools/)
- [Python typing information in packages](https://typing.python.org/en/latest/spec/distributing.html)
- [PyPI first-release trusted publishing](https://docs.pypi.org/trusted-publishers/creating-a-project-through-oidc/)
- [PyPI trusted-publisher setup](https://docs.pypi.org/trusted-publishers/adding-a-publisher/)
- [PyPI trusted-publisher security model](https://docs.pypi.org/trusted-publishers/security-model/)
- [PyPI trusted-publisher troubleshooting](https://docs.pypi.org/trusted-publishers/troubleshooting/)
- [crates.io trusted publishing announcement](https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07/)
- [Flowmark 0.3.2 source](https://github.com/jlevy/flowmark-rs/tree/015f23989af3e5cfb3f8b58dfc72822c534df25a)
- [Flowmark 0.3.2 release](https://github.com/jlevy/flowmark-rs/releases/tag/v0.3.2)
- [Flowmark publishing runbook](https://github.com/jlevy/flowmark-rs/blob/015f23989af3e5cfb3f8b58dfc72822c534df25a/docs/publishing.md)
- [Rust porting playbook reference](https://github.com/jlevy/rust-porting-playbook/tree/d24760a3fbd2951c730a199269aeb082abb46a42)
- [Rust porting playbook release rules](https://github.com/jlevy/rust-porting-playbook/blob/d24760a3fbd2951c730a199269aeb082abb46a42/guidelines/rust-release-rules.md)
- [metabrowser](https://github.com/jlevy/metabrowser)

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
