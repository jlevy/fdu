# Release Process

fdu has one product version and three public delivery surfaces: the `fdu` crate, the
`fdu` Python distribution, and GitHub release evidence.
The Cargo package version is authoritative.
The release tag, CLI, report generator, Python module, source distribution, wheels, and
release evidence must all identify that same version.

The current workflow is deliberately a non-publishing rehearsal.
It builds the crate preview, source distribution, and five-wheel platform matrix;
smoke-tests every native artifact it can run; inspects metadata, typing, licenses, and
SBOMs; classifies each registry version as missing, identical, or conflicting; and
retains a checksum manifest.
Registry writes remain in `fdu-9cf0` and must not be enabled until both project-specific
publisher records exist and a rehearsal succeeds from the intended release commit.

## Supported Artifacts

There are two Rust crates, and their order is a release invariant.
`fdu-core` is the engine; `fdu` is the command line and depends on it.
So `fdu-core` must be published first: until it exists on crates.io, `fdu` has nothing
to resolve against and cannot be published — or even packaged alone, which is why the
rehearsal and the release workflow package both in one `cargo package` invocation rather
than two.
Both carry the same version, and a release that publishes one without the other
leaves `fdu` unbuildable for anyone who installs it.

The Rust crate supports the default CLI with watch support and a minimal library build
through `default-features = false`. Rust 1.85 is the minimum supported version.

The Python package supports CPython 3.12 and newer through one `abi3-py312` extension.
The first binary matrix is:

| Platform | Architecture | Compatibility floor |
| --- | --- | --- |
| Linux glibc | x86-64, arm64 | manylinux2014 / glibc 2.17 |
| macOS | x86-64, arm64 | macOS 11.0 |
| Windows | x86-64 | Current GitHub-hosted MSVC toolchain |

Other systems may build the source distribution with a compatible Rust toolchain.
That fallback is not the same promise as a zero-build `uvx` install.

Within the `0.1` series, incompatible Rust or Python API changes require a documented
deprecation path when practical.
Machine-report field changes require a report-schema version bump.
Security reports should use GitHub’s private vulnerability-reporting channel rather than
a public issue.

## Local Release Rehearsal

Run the normal handoff gate, then the artifact rehearsal:

```shell
make check
make cross-lint
make release-rehearse
```

`make release-rehearse` sets an explicit matching release identity, asks Cargo to
package and verify the crate, builds the source distribution and host abi3 wheel, and
runs the same artifact inspector used by GitHub Actions.
It does not contact either publishing API.

The GitHub rehearsal performs one additional read-only registry audit against the
validated manifest. A missing version is ready for a first upload, an identical version
is safe to skip during recovery, and any filename or hash disagreement is a conflict
that stops the workflow.
The audit uses public registry endpoints and no credentials.

The manually dispatched top-level
[`release.yml`](../../../.github/workflows/release.yml) workflow extends that rehearsal
to Linux x86-64/arm64, macOS x86-64/arm64, and Windows x86-64. The Linux builds use a
controlled manylinux2014 image rather than inheriting the hosted runner’s glibc.
Cross-built Linux arm64 receives structural artifact validation; the evidence manifest
does not mislabel that as a native execution test.

## Account and Authentication Model

Use the same maintainer accounts that publish Flowmark, with publisher subjects created
specifically for this repository.
No Flowmark token or publisher record is reused.

| Channel | Required setup |
| --- | --- |
| GitHub Releases | Repository `jlevy/fdu`; only the final announcement job receives `contents: write`. |
| PyPI | Project `fdu`; trusted publisher owner `jlevy`, repository `fdu`, top-level workflow `release.yml`, protected environment `release`. The narrow publish job receives `id-token: write` and no API token. |
| crates.io first release | The same crates.io owner publishes `0.1.0` with a narrowly scoped, short-lived token because a trusted publisher cannot be attached before the crate exists. Remove the token after verification. |
| crates.io later releases | Trusted publisher owner `jlevy`, repository `fdu`, workflow `release.yml`, environment `release`; exchange GitHub OIDC through `rust-lang/crates-io-auth-action` only inside the publish job. |

Crates.io publishing is authenticated in both cases.
The bootstrap uses the registry token; steady state exchanges the workflow’s OIDC
identity for a short-lived Cargo credential.
Neither credential belongs in repository files, logs, build artifacts, or reusable
workflows.

## Publication Invariants

When registry jobs are added, they must consume only the validated artifact set.
PyPI uploads the tested source distribution and wheels without rebuilding.
Cargo is the narrow exception because `cargo publish` repackages source: the publish job
must reproduce the validated `.crate`, compare its SHA-256 digest with the retained
preview, and abort on any mismatch before upload.

Before the first registry write:

1. Recheck that `fdu`, `fdu-core`, and the Python name are still available on their
   registries.
2. Protect the `release` environment and configure the PyPI pending trusted publisher.
3. Run the non-publishing workflow from the exact intended commit and retain its
   manifest.
4. Create and verify the signed `v0.1.0` tag only after every required check is green.
5. Approve the registry jobs separately; never let a build job hold publication
   authority.

Registries are independently retryable, not atomic.
Within crates.io the two crates are not independent: publish `fdu-core`, wait for the
index to carry it, then publish `fdu`. After a partial failure, verify the successful
registry’s version and hash, rerun only the missing channel, and stop on any
same-version hash conflict.
Never retag, replace an immutable artifact, or rebuild from a different commit.

The implementation audit, Flowmark comparison, deliberate divergences, and proposed
upstream improvements live in the
[release packaging and Python API plan](../specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md).

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
