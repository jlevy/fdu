# Release Process

fdu has one product version and three public delivery surfaces: the `fdu-core` and `fdu`
crates, the `fdu` Python distribution, and GitHub release evidence.
The Cargo package version is authoritative.
The release tag, CLI, report generator, Python module, source distribution, wheels, and
release evidence must all identify that same version.

The current workflow is deliberately a non-publishing rehearsal.
It builds both crates, the source distribution, and the five-wheel platform matrix;
smoke-tests every native artifact it can run; inspects metadata, typing, licenses, and
SBOMs; classifies each crate and the Python release on its registry as missing,
identical, or conflicting; and retains a checksum manifest.
`0.1.0` is published by hand from the signed tag, as
[Publishing 0.1.0 by Hand](#publishing-010-by-hand) describes.
Registry jobs in the workflow remain in `fdu-9cf0` and come with a later release.

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

fdu is pre-1.0, so compatibility follows the `0.x` minor rule: a minor release (`0.1` to
`0.2`) may change the Rust or Python API incompatibly, and its CHANGELOG entry names
each such change; a patch release (`0.1.0` to `0.1.1`) never does.
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
package and verify both crates, installs the packaged `fdu` against the packaged
`fdu-core` exactly as the workflow’s crate job does, builds the source distribution and
host abi3 wheel, and runs the same artifact inspector used by GitHub Actions.
It does not contact either publishing API.

The crate install needs a patch, and `scripts/release/smoke_crate.py` explains why: the
packaged `fdu` pins `fdu-core` from crates.io, where it does not exist before the first
publish, so the script resolves it to the packaged sibling and checks that nothing else
in the lockfile moved.

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
| GitHub Releases | Repository `jlevy/fdu`; once automated, only the final announcement job receives `contents: write`. |
| PyPI first release | The project `fdu` does not exist until `0.1.0` is uploaded, and a project-scoped token needs an existing project, so the maintainer uploads with a short-lived account-scoped API token. Delete the token after verification. |
| PyPI later releases | Trusted publisher owner `jlevy`, repository `fdu`, top-level workflow `release.yml`, protected environment `release`. The narrow publish job receives `id-token: write` and no API token. |
| crates.io first release | The same crates.io owner publishes `0.1.0` with a narrowly scoped, short-lived token because a trusted publisher cannot be attached before the crate exists. Remove the token after verification. |
| crates.io later releases | Trusted publisher owner `jlevy`, repository `fdu`, workflow `release.yml`, environment `release`; exchange GitHub OIDC through `rust-lang/crates-io-auth-action` only inside the publish job. |

Crates.io publishing is authenticated in both cases.
The bootstrap uses the registry token; steady state exchanges the workflow’s OIDC
identity for a short-lived Cargo credential.
Neither credential belongs in repository files, logs, build artifacts, or reusable
workflows.

## Publication Invariants

Every upload consumes only the validated artifact set.
PyPI receives the tested source distribution and wheels without rebuilding.
Cargo is the narrow exception because `cargo publish` repackages source: reproduce the
validated `.crate`, compare its SHA-256 digest with the retained preview, and abort on
any mismatch before upload.
When registry jobs are added, they run behind the protected `release` environment and
are approved separately; a build job never holds publication authority.

Registries are independently retryable, not atomic.
Within crates.io the two crates are not independent: publish `fdu-core`, wait for the
index to carry it, then publish `fdu`. After a partial failure, verify the successful
registry’s version and hash, rerun only the missing channel, and stop on any
same-version hash conflict.
Never retag, replace an immutable artifact, or rebuild from a different commit.

## Publishing 0.1.0 by Hand

A maintainer publishes `0.1.0` from the signed tag in this order: `fdu-core`, then
`fdu`, then the Python distribution.
Every upload carries bytes the rehearsal validated, and each registry is checked against
the rehearsal’s manifest before the next write.

The commands assume a POSIX shell with `gh`, `uv`, and `rustup`. `RELEASE` is an empty
scratch directory outside any checkout, and `<run-id>` and `<release-commit>` are
recorded in the first step.

### Rehearse the Release Commit

1. Merge everything the release needs, dispatch the rehearsal on `main`, and wait for it
   to pass:

   ```shell
   gh workflow run release.yml --repo jlevy/fdu --ref main
   gh run list --repo jlevy/fdu --workflow release.yml --limit 1
   gh run watch <run-id> --repo jlevy/fdu --exit-status
   ```

2. Record the commit the run built.
   That commit is the release commit, wherever `main` points later:

   ```shell
   gh run view <run-id> --repo jlevy/fdu --json headSha --jq .headSha
   ```

3. Download every artifact, gather the files into one directory, and verify them against
   the run’s own checksums.
   `gh run download` writes each artifact to its own subdirectory.
   Artifacts expire (90 days by default), so keep this directory until the release is
   announced.

   ```shell
   gh run download <run-id> --repo jlevy/fdu --dir "$RELEASE/download"
   mkdir "$RELEASE/files"
   find "$RELEASE/download" -type f -exec cp {} "$RELEASE/files/" \;
   (cd "$RELEASE/files" && shasum -a 256 -c SHA256SUMS)
   ```

   The check must list both crates, the source distribution, and five wheels.

### Tag the Release Commit

1. Create, verify, and push the signed tag:

   ```shell
   git fetch origin
   git tag -s v0.1.0 <release-commit> -m "fdu 0.1.0"
   git tag -v v0.1.0
   git push origin v0.1.0
   ```

2. Clone the pushed tag into a clean directory, and confirm that it names the rehearsed
   commit and the Cargo version:

   ```shell
   git clone --branch v0.1.0 https://github.com/jlevy/fdu "$RELEASE/fdu"
   cd "$RELEASE/fdu"
   uv run --no-project --python 3.12 python scripts/release/resolve_plan.py \
     --mode release --ref refs/tags/v0.1.0 --commit <release-commit> --validate-checkout
   ```

3. Recheck that both crate names and the Python name are still free.
   Each command must print `404`:

   ```shell
   curl -sS -o /dev/null -w '%{http_code}\n' -A 'fdu-release (https://github.com/jlevy/fdu)' \
     https://crates.io/api/v1/crates/fdu-core
   curl -sS -o /dev/null -w '%{http_code}\n' -A 'fdu-release (https://github.com/jlevy/fdu)' \
     https://crates.io/api/v1/crates/fdu
   curl -sS -o /dev/null -w '%{http_code}\n' https://pypi.org/pypi/fdu/json
   ```

Every remaining command runs in `$RELEASE/fdu`, whose `rust-toolchain.toml` selects the
pinned Rust.

### Publish the Crates

1. Create a crates.io API token with the `publish-new` and `publish-update` scopes,
   limited to the crates `fdu-core` and `fdu`, with the shortest expiry crates.io
   offers. Read it without echoing it or writing it to shell history:

   ```shell
   read -rs CARGO_REGISTRY_TOKEN && export CARGO_REGISTRY_TOKEN
   export FDU_RELEASE_TAG=v0.1.0
   ```

2. Reproduce both crates from the tag and compare them with the rehearsal’s digests.
   A mismatch means crates.io would receive bytes nothing tested: stop, and publish
   nothing until the difference is explained.
   No run has yet compared a maintainer’s packaging with the Linux rehearsal’s, so on
   the first release rule out a platform difference first.
   If the extracted file trees are identical and only the archives differ, reproduce on
   Linux x86-64 with the pinned toolchain rather than relaxing the comparison.

   ```shell
   cargo package --locked -p fdu-core -p fdu
   (cd target/package && grep '\.crate$' "$RELEASE/files/SHA256SUMS" | shasum -a 256 -c -)
   ```

3. Publish `fdu-core`. Cargo waits until the index carries it.
   The audit must then report `fdu-core` as `identical` and `fdu` as `missing`:

   ```shell
   cargo publish --locked -p fdu-core
   uv run --no-project --python 3.12 python scripts/release/registry_state.py \
     --manifest "$RELEASE/files/release-manifest.json" --version 0.1.0 --channel crates.io
   ```

4. `fdu` now resolves `fdu-core` from crates.io rather than from the packaged sibling,
   so reproduce and compare it once more, then publish it.
   The audit must report both crates as `identical`:

   ```shell
   cargo package --locked -p fdu
   (cd target/package && grep ' fdu-0\.1\.0\.crate$' "$RELEASE/files/SHA256SUMS" | shasum -a 256 -c -)
   cargo publish --locked -p fdu
   uv run --no-project --python 3.12 python scripts/release/registry_state.py \
     --manifest "$RELEASE/files/release-manifest.json" --version 0.1.0 --channel crates.io
   ```

5. Install the published crate as a user does, outside the checkout, and check that it
   reports `fdu 0.1.0`:

   ```shell
   unset FDU_RELEASE_TAG
   (cd "$RELEASE" && cargo install fdu --locked --version 0.1.0 --root "$RELEASE/cargo-install")
   "$RELEASE/cargo-install/bin/fdu" --version
   ```

6. Remove the token: `unset CARGO_REGISTRY_TOKEN`, then revoke it in the crates.io
   account settings. Configure crates.io trusted publishing on both crates for later
   releases, as the account table describes.

### Publish the Python Distribution

1. Create a PyPI API token scoped to the account, and read it the same way:

   ```shell
   read -rs UV_PUBLISH_TOKEN && export UV_PUBLISH_TOKEN
   ```

2. Upload the rehearsal’s source distribution and five wheels, never a rebuild.
   `--check-url` lets a rerun skip files PyPI already holds.

   ```shell
   uv publish --trusted-publishing never --check-url https://pypi.org/simple/ \
     "$RELEASE"/files/fdu-0.1.0.tar.gz "$RELEASE"/files/fdu-0.1.0-*.whl
   unset UV_PUBLISH_TOKEN
   ```

3. The audit must report the PyPI release as `identical`, and the published wheel must
   run. `--no-config` sets aside any user-level `exclude-newer` cool-off, which would
   hide a release published minutes ago:

   ```shell
   uv run --no-project --python 3.12 python scripts/release/registry_state.py \
     --manifest "$RELEASE/files/release-manifest.json" --version 0.1.0 --channel pypi
   (cd "$RELEASE" && uv tool run --no-config --from fdu==0.1.0 fdu --version)
   ```

4. Delete the token in the PyPI account settings, and add the trusted publisher for
   later releases to the now-existing `fdu` project.

### Announce the Release

Once every channel verifies, record the final registry state and attach it with the
evidence and artifacts to a GitHub release on the tag.
The notes are the CHANGELOG’s `[0.1.0]` section, saved as `$RELEASE/notes.md`.

```shell
uv run --no-project --python 3.12 python scripts/release/registry_state.py \
  --manifest "$RELEASE/files/release-manifest.json" --version 0.1.0 \
  --output "$RELEASE/registry-state.json"
gh release create v0.1.0 --repo jlevy/fdu --verify-tag --title "fdu 0.1.0" \
  --notes-file "$RELEASE/notes.md" \
  "$RELEASE/registry-state.json" "$RELEASE"/files/release-manifest.json \
  "$RELEASE"/files/SHA256SUMS "$RELEASE"/files/*.crate "$RELEASE"/files/*.whl \
  "$RELEASE"/files/fdu-0.1.0.tar.gz
```

If any step fails partway, follow the recovery rules in
[Publication Invariants](#publication-invariants): verify what reached each registry
with `registry_state.py`, rerun only what is missing, and stop on a conflict.

The implementation audit, Flowmark comparison, deliberate divergences, and proposed
upstream improvements live in the
[release packaging and Python API plan](../specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md).

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
