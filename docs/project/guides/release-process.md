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
A machine-output field change requires a version bump of the schema that carries it: the
report (`fdu.report/5`, or `fdu.report/6` with content analysis or a metric summary),
the watch stream (`fdu.stream/1`), and cache status (`fdu.cache/1`) each version
independently, as
[the surface architecture](../architecture/fdu-surface-architecture.md#machine-output-schemas)
lists. Every release also strands the snapshots the previous one wrote, because the
engine fingerprint mixes in the crate version: after an upgrade `--cache-status` reports
them as `stale`, no run reuses them, and `--cache-clear` removes them.
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
A conflict therefore ends that version on every channel;
[Recover From a Partial Publication](#recover-from-a-partial-publication) gives the
procedure.

## Publishing 0.1.0 by Hand

A maintainer publishes `0.1.0` from the signed tag in this order: `fdu-core`, then
`fdu`, then the Python distribution.
Every upload carries bytes the rehearsal validated, and each registry is checked against
the rehearsal’s manifest before the next write.

The commands assume bash or zsh, with `gh`, `uv`, `rustup`, and `curl`: the token
prompts use `read -s`, which a plain POSIX `sh` such as `dash` rejects.
`RELEASE` is an empty scratch directory outside any checkout, and `<run-id>` and
`<release-commit>` are recorded in the first step.

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

1. Tag from your own clone of `jlevy/fdu`, whose `origin` is GitHub, not from
   `$RELEASE`. Fetch, and confirm that the working tree is clean and the release commit
   is on `main`: the status command must print nothing, and the ancestry check must
   print `on main`.

   ```shell
   git fetch origin
   git status --porcelain
   git merge-base --is-ancestor <release-commit> origin/main && echo "on main"
   ```

   Then check out exactly the release commit, and derive the GitHub release body from
   its notes, [`docs/project/release-notes/0.1.0.md`](../release-notes/0.1.0.md).
   GitHub renders a single newline in a release body as a line break, so the
   flowmark-wrapped file would show every source line break.
   The body is the file with its HTML comments removed and each paragraph and list item
   joined onto one line by the Makefile’s pinned flowmark, which changes nothing but
   whitespace:

   ```shell
   git switch --detach <release-commit>
   uv run --no-project --python 3.12 python -c \
     'import re, sys; sys.stdout.write(re.sub(r"<!--.*?-->\n*", "", sys.stdin.read(), flags=re.S))' \
     < docs/project/release-notes/0.1.0.md > "$RELEASE/notes-source.md"
   uv run --project explorations/benchmarks --frozen --only-group docs \
     flowmark --width 0 --output "$RELEASE/notes.md" "$RELEASE/notes-source.md"
   ```

   Check the body before tagging, because a fix after the tag needs a new commit and so
   a new version. The first command must print `1`, the notes’ standard footer, so no
   unfilled draft comment was stripped silently; the second must print nothing, so the
   body differs from the notes only in whitespace; and the third must print `0`, so
   GitHub’s renderer finds no line break inside a paragraph.
   Read `$RELEASE/notes.html` as well: it is the body as GitHub will render it.

   ```shell
   grep -c '<!--' docs/project/release-notes/0.1.0.md
   cmp <(tr -s '[:space:]' ' ' < "$RELEASE/notes-source.md") \
     <(tr -s '[:space:]' ' ' < "$RELEASE/notes.md")
   gh api markdown -f mode=gfm -F text=@"$RELEASE/notes.md" > "$RELEASE/notes.html" &&
     grep -c '<br>' "$RELEASE/notes.html"
   ```

   Then create, verify, and push the signed tag on the release commit:

   ```shell
   git tag -s v0.1.0 -m "fdu 0.1.0"
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

   On the `0.1.1` path, a name `0.1.0` reached prints `200`; step 5 of
   [Recover From a Partial Publication](#recover-from-a-partial-publication) gives the
   check that replaces this one for that name.

Every remaining command runs in `$RELEASE/fdu`, whose `rust-toolchain.toml` selects the
pinned Rust.

### Publish the Crates

1. Create a crates.io API token with the `publish-new`, `publish-update`, and `yank`
   scopes, limited to the crates `fdu-core` and `fdu`, with the shortest expiry
   crates.io offers. `yank` is there so that a conflict can be contained at once, without
   creating a second token mid-incident.
   Read the token without echoing it or writing it to shell history:

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

   A match is evidence about this host and this tree only, and Cargo cannot upload
   anything else: `cargo publish` takes no archive argument and repackages from the
   checkout every time.
   So the host whose digests matched is the host that publishes.
   If only a Linux reproduction matches, do all of Publish the Crates on that Linux
   host: download and check the rehearsal’s files there as in
   [Rehearse the Release Commit](#rehearse-the-release-commit) step 3, clone and
   validate the tag as in [Tag the Release Commit](#tag-the-release-commit) step 2, then
   run steps 1 to 6 of this section in that clone.
   Publishing from the host whose digests differed uploads the bytes that failed the
   comparison.

3. Publish `fdu-core`. Cargo waits until the index carries it.
   The audit must then report `fdu-core` as `identical` and `fdu` as `missing`. This is
   the first write nobody can undo: from here on, any failure goes through
   [Recover From a Partial Publication](#recover-from-a-partial-publication) before
   anything else runs.

   ```shell
   cargo publish --locked -p fdu-core
   uv run --no-project --python 3.12 python scripts/release/registry_state.py \
     --manifest "$RELEASE/files/release-manifest.json" --version 0.1.0 --channel crates.io
   ```

   If `cargo publish` succeeded but the audit still reports `fdu-core` as `missing`, the
   crates.io version record the audit reads may be trailing the index Cargo waited for.
   Rerun the audit once a minute for up to ten minutes.
   A `missing` that outlasts that is a failed upload.

4. `fdu` now resolves `fdu-core` from crates.io rather than from the packaged sibling,
   so reproduce and compare it once more, then publish it.
   The audit must report both crates as `identical`; `--require-identical` makes it exit
   3 while either is `missing`, which right after the publish may be the same lag, so
   rerun it as in step 3:

   ```shell
   cargo package --locked -p fdu
   (cd target/package && grep ' fdu-0\.1\.0\.crate$' "$RELEASE/files/SHA256SUMS" | shasum -a 256 -c -)
   cargo publish --locked -p fdu
   uv run --no-project --python 3.12 python scripts/release/registry_state.py \
     --manifest "$RELEASE/files/release-manifest.json" --version 0.1.0 --channel crates.io \
     --require-identical
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
   hide a release published minutes ago.
   PyPI’s API and index can also trail an upload, so if the audit exits 3 for a
   `missing` release, or the install cannot find `fdu==0.1.0`, rerun both as in step 3
   of Publish the Crates:

   ```shell
   uv run --no-project --python 3.12 python scripts/release/registry_state.py \
     --manifest "$RELEASE/files/release-manifest.json" --version 0.1.0 --channel pypi \
     --require-identical &&
     (cd "$RELEASE" && uv tool run --no-config --from fdu==0.1.0 fdu --version)
   ```

4. Delete the token in the PyPI account settings, and add the trusted publisher for
   later releases to the now-existing `fdu` project.

### Announce the Release

Once every channel verifies, record the final registry state and attach it with the
evidence and artifacts to a GitHub release on the tag.
The body is `$RELEASE/notes.md`, derived from the release commit’s notes and checked in
step 1 of [Tag the Release Commit](#tag-the-release-commit); step 2 confirmed that the
tag names that commit, so the body is the tagged text.
The release is created only if the audit exits 0, which with `--require-identical` means
every channel holds exactly the rehearsal’s files:

```shell
uv run --no-project --python 3.12 python scripts/release/registry_state.py \
  --manifest "$RELEASE/files/release-manifest.json" --version 0.1.0 \
  --require-identical --output "$RELEASE/registry-state.json" &&
  gh release create v0.1.0 --repo jlevy/fdu --verify-tag --title "fdu 0.1.0" \
    --notes-file "$RELEASE/notes.md" \
    "$RELEASE/registry-state.json" "$RELEASE"/files/release-manifest.json \
    "$RELEASE"/files/SHA256SUMS "$RELEASE"/files/*.crate "$RELEASE"/files/*.whl \
    "$RELEASE"/files/fdu-0.1.0.tar.gz
```

### Recover From a Partial Publication

Neither crates.io nor PyPI lets a version’s files be replaced, even after a yank or a
deletion.
So when any step fails, first run that channel’s audit, the `registry_state.py`
command from the step that failed, and let its verdict decide what comes next.
An audit that cannot read a registry stops with an error naming the URL and exits 1.
That is no verdict: rerun the audit, and never read it as `missing`.

| Audit reports | Meaning | Next step |
| --- | --- | --- |
| `missing`, after the lag retry in step 3 of Publish the Crates | Nothing reached the registry. | Fix the cause, then rerun the failed step from its start, so a crate is compared again before it is published. |
| `identical` | The upload landed, though the command reported a failure. | Continue with the next step. |
| `conflict` | The registry holds bytes nothing tested, under a version that cannot be reused. | Follow the procedure below. |

A `conflict` on any channel, or any failure whose fix needs a new commit, ends `0.1.0`:
a published file cannot be replaced, the pushed tag cannot move, and every channel
carries one version.
The first case can only arise from step 3 of Publish the Crates onward, once `fdu-core`
is on crates.io.

1. **Publish nothing more under `0.1.0`.** Run no later step.
   Above all, never publish `fdu` against an `fdu-core` that conflicts.

2. **Record what happened while the evidence is fresh**, in a bead or a GitHub issue:

   - the audit’s output;
   - the published digest of each conflicting crate beside the manifest’s, from
     `curl -fsSL -A 'fdu-release (https://github.com/jlevy/fdu)' https://crates.io/api/v1/crates/<crate>/0.1.0/download | shasum -a 256`;
   - the host and toolchain that published (`uname -a`, `cargo -V`) and the digests step
     2 of Publish the Crates printed;
   - the release commit, the rehearsal’s run ID, and every version yanked.

   Keep `$RELEASE`, including the clone’s `target/package`, which holds the archives
   `cargo publish` built and uploaded.

3. **Yank each version whose published bytes are wrong, and only those.**
   `cargo yank fdu-core@0.1.0` (or `fdu@0.1.0`) uses the token from step 1 of Publish
   the Crates; on PyPI, yank the release from the project’s settings.
   A version the audit reports as `identical` holds the tested bytes and stays.
   A yank stops new resolution; it deletes nothing and does not break an existing
   lockfile.

4. **Never retag.** `v0.1.0` keeps naming the release commit, and nothing is published
   again under `0.1.0` on any channel.

5. **Release `0.1.1` instead.** Bump the version in a pull request, including every
   manifest and the expected version in `tests/release/test_metadata.py`, and give the
   CHANGELOG a `0.1.1` entry that says which `0.1.0` artifacts exist and which were
   yanked. Once it merges, repeat this section from
   [Rehearse the Release Commit](#rehearse-the-release-commit) with `0.1.1` in place of
   `0.1.0`. Both crates are published at `0.1.1`, `fdu-core` included, even if its
   source did not change.

   On that pass, [Tag the Release Commit](#tag-the-release-commit) step 3 still requires
   `404` for a name `0.1.0` never reached.
   A name it did reach now prints `200`, so check that name’s new version instead; for
   each such name, its command here must print `404`:

   ```shell
   curl -sS -o /dev/null -w '%{http_code}\n' -A 'fdu-release (https://github.com/jlevy/fdu)' \
     https://crates.io/api/v1/crates/fdu-core/0.1.1
   curl -sS -o /dev/null -w '%{http_code}\n' -A 'fdu-release (https://github.com/jlevy/fdu)' \
     https://crates.io/api/v1/crates/fdu/0.1.1
   curl -sS -o /dev/null -w '%{http_code}\n' https://pypi.org/pypi/fdu/0.1.1/json
   ```

Whatever the outcome, unset and revoke every token as the steps above describe.

The implementation audit, Flowmark comparison, deliberate divergences, and proposed
upstream improvements live in the
[release packaging and Python API plan](../specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md).

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
