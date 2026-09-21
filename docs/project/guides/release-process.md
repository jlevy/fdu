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
report (`fdu.report/7`, or `fdu.report/8` with content analysis or a metric summary),
the watch stream (`fdu.stream/1`), and cache status (`fdu.cache/2`) each version
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

That install runs inside a throwaway git repository the script creates, which is what
makes the version it then asserts mean anything.
Installed outside any repository, `crates/fdu/build.rs` reports bare semver through its
own fallback, so the assertion held with the `.cargo_vcs_info.json` skip deleted and
pinned nothing (`fdu-tleo`). Installed inside one there is a revision available to
stamp, so bare `fdu 0.1.0` can come only from that skip — which is also the case the
skip exists for, a published crate unpacked under a checkout belonging to somebody else.

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

The first-time walkthrough for each row is
[First-Time Channel Setup](#first-time-channel-setup).

Crates.io publishing is authenticated in both cases.
The bootstrap uses the registry token; steady state exchanges the workflow’s OIDC
identity for a short-lived Cargo credential.
Neither credential belongs in repository files, logs, build artifacts, or reusable
workflows.

Trusted publishers come after `0.1.0`, and only once the `release` environment exists
and is protected: a required reviewer, and a deployment policy that admits only `v*`
tags. GitHub creates an unprotected environment the first time a workflow job names it,
so a publisher registered before the protection would trust any run of `release.yml`
that names `release`, from any ref.

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
A hash conflict therefore ends that version on every channel;
[Recover From a Partial Publication](#recover-from-a-partial-publication) gives the
procedure.

## First-Time Channel Setup

`0.1.0` is the first time the names `fdu` and `fdu-core` appear on either registry.
The accounts and the GitHub environment can be finished days before the tag.
The API tokens cannot: they are created in [Publish the Crates](#publish-the-crates) and
[Publish the Python Distribution](#publish-the-python-distribution), used once, and
revoked in those same sections.

Recheck the three names immediately before the first write; availability is a race.
Use the JSON and crates.io API curls in
[Tag the Release Commit](#tag-the-release-commit) step 2, not the HTML project pages.
`https://pypi.org/project/fdu/` can return HTTP 200 with an anti-bot interstitial for a
name that does not exist.

### Why the First Publish Is by Hand

crates.io will not accept a
[trusted publisher](https://crates.io/docs/trusted-publishing) until the crate exists,
so `fdu-core` and `fdu` must be created with an API token.

PyPI
[can create a project from a pending trusted publisher](https://docs.pypi.org/trusted-publishers/creating-a-project-through-oidc/).
This repository does not use that path for `0.1.0`:
[`release.yml`](../../../.github/workflows/release.yml) is a rehearsal and has no
publish job, and the first release is uploaded by hand from the signed tag.

Do not register a pending PyPI publisher for `fdu` before that upload.
A pending publisher does not reserve the name, and if `release.yml` later grows a
publish job before the `release` environment is protected, the pending record would
trust an unprotected subject.

### crates.io Account

1. Sign in at [crates.io](https://crates.io/) with the GitHub account that owns
   `jlevy/fdu`. crates.io has no other login.
2. On [Account Settings](https://crates.io/settings), confirm the email is verified.
   crates.io has no native 2FA: login is GitHub OAuth.
   Confirm two-factor authentication is enabled on the GitHub account that owns
   `jlevy/fdu` (Settings → Password and authentication).
3. Do not create a token yet, and do not add a trusted publisher: there is no crate to
   attach one to.

The token is created on publish day, at
[New API Token](https://crates.io/settings/tokens/new):

- **Scopes:** `publish-new`, `publish-update`, and `yank`. `yank` is so a conflict can
  be contained without minting a second token.
  Leave `change-owners` off.
- **Crates:** restrict to `fdu-core` and `fdu`. A crate-name restriction applies to
  future crates the account owns, so the names may be listed before they exist.
- **Expiry:** the shortest preset crates.io offers, or a custom date that covers only
  the publish window.

Paste it into `CARGO_REGISTRY_TOKEN` as [Publish the Crates](#publish-the-crates) step 1
describes. Do not run `cargo login`, and do not store the token in
`~/.cargo/credentials.toml`, a GitHub secret, or the `release` environment.

### PyPI Account

1. Sign in with the same PyPI account that publishes Flowmark.
   Do not create a second account for fdu.
2. [Two-factor authentication](https://blog.pypi.org/posts/2024-01-01-2fa-enforced/) is
   required for every management action and every upload.
   Enable it under [Account settings](https://pypi.org/manage/account/) if it is not
   already on.
3. Confirm the account email is verified.

The token is created on publish day, at
[Add API token](https://pypi.org/manage/account/token/):

- **Scope:** Entire account (all projects).
  A project-scoped token cannot be created until `fdu` exists.
- **Name:** something that names this one upload, for example `fdu-0.1.0-bootstrap`.

Paste it into `UV_PUBLISH_TOKEN` as
[Publish the Python Distribution](#publish-the-python-distribution) step 1 describes.
`uv publish` sends it as the password with username `__token__`. Do not write a
`.pypirc`, and do not store the token in a GitHub secret.

### GitHub `release` Environment

`0.1.0` does not use a GitHub environment.
Create the protected `release` environment anyway, before anyone registers a trusted
publisher, because GitHub creates an *unprotected* environment the first time a workflow
job names one.

In the repository: Settings → Environments → New environment, name `release`. Then:

- **Required reviewers:** the maintainer who publishes Flowmark.
  Leave Prevent self-review off: a single-maintainer repository cannot approve its own
  deployment if that is on.
- **Deployment branches and tags:** Selected branches and tags.
  Add a deployment branch or tag rule with Ref type **Tag** and pattern `v*`. Add no
  Branch rules: `v*` as a Branch rule matches names such as `validate-*`. No branch,
  including `main`, should be able to deploy to `release`.
- If **Allow administrators to bypass configured protection rules** is selected,
  deselect it.

Do not add `CARGO_REGISTRY_TOKEN` or `UV_PUBLISH_TOKEN` as environment secrets.
Later publish jobs receive `id-token: write` and exchange OIDC; they hold no long-lived
registry credential.

### After 0.1.0: Trusted Publishers

Only after all three of these are true:

1. `fdu-core` and `fdu` exist on crates.io, and `fdu` exists on PyPI.
2. The `release` environment exists and is protected as above.
3. Every bootstrap token has been revoked.

Then add the publishers.
Use these subjects on every record; they are specific to this repository, not copied
from Flowmark:

| Field | Value |
| --- | --- |
| Owner | `jlevy` |
| Repository | `fdu` |
| Workflow filename | `release.yml` (the top-level file; not a reusable workflow) |
| Environment | `release` |

On crates.io, open each crate’s settings and add a GitHub Actions trusted publisher with
those fields. On PyPI, open
[the `fdu` project’s publishing page](https://pypi.org/manage/project/fdu/settings/publishing/)
and add the same GitHub publisher.
Do not create a pending publisher: the project already exists.

Registering the publishers does not publish anything.
[`release.yml`](../../../.github/workflows/release.yml) still has no publish job; adding
those jobs is later work.
The records exist so that work has a protected subject to name.

### Publication Sequence

The first release is this order.
Channel setup is the only block that can finish before the release commit exists.

1. Finish this section: accounts, 2FA, and the protected `release` environment.
2. Merge everything the release needs onto `main`.
3. [Rehearse the Release Commit](#rehearse-the-release-commit).
4. [Tag the Release Commit](#tag-the-release-commit), including the name recheck.
5. [Publish the Crates](#publish-the-crates): `fdu-core`, then `fdu`. Revoke the
   crates.io token.
6. [Publish the Python Distribution](#publish-the-python-distribution).
   Revoke the PyPI token.
7. [Announce the Release](#announce-the-release).
8. Add [trusted publishers](#after-010-trusted-publishers).

## Publishing 0.1.0 by Hand

A maintainer publishes `0.1.0` from the signed tag in this order: `fdu-core`, then
`fdu`, then the Python distribution.
[First-Time Channel Setup](#first-time-channel-setup) must already be done: the accounts
exist, 2FA is on, and the `release` environment is protected.
Every upload carries bytes the rehearsal validated, and each registry is checked against
the rehearsal’s manifest before the next write.

The commands assume bash or zsh, with `gh`, `uv`, `rustup`, and `curl`: the token
prompts use `read -s`, which a plain POSIX `sh` such as `dash` rejects.
`RELEASE` is an empty scratch directory outside any checkout, and `<run-id>` and
`<release-commit>` are recorded in
[Rehearse the Release Commit](#rehearse-the-release-commit).

### Prerequisites

1. **A tag-signing key.** The tag is signed with an SSH key.
   Set it up once, with `<key>` your key’s file name and `<email>` your Git
   `user.email`, which must be a verified address on your GitHub account for GitHub to
   show the tag as verified:

   ```shell
   git config --global gpg.format ssh
   git config --global user.signingkey ~/.ssh/<key>.pub
   echo "<email> namespaces=\"git\" $(cat ~/.ssh/<key>.pub)" >> ~/.ssh/allowed_signers
   git config --global gpg.ssh.allowedSignersFile ~/.ssh/allowed_signers
   ```

   `git tag -v` needs the allowed-signers file; without it, verification fails even for
   a correctly signed tag.
   Register the same public key on GitHub as a *signing* key, which is separate from an
   authentication key; `gh` needs an extra scope to add one:

   ```shell
   gh auth refresh -h github.com -s write:ssh_signing_key
   gh ssh-key add ~/.ssh/<key>.pub --type signing --title "fdu release signing"
   ```

2. **Private vulnerability reporting.** [SECURITY.md](../../../SECURITY.md) and the
   release notes send reporters to GitHub’s private reporting form, so it must be
   enabled before the release is announced.
   The first command must print `true`; if it prints `false`, the second enables it:

   ```shell
   gh api repos/jlevy/fdu/private-vulnerability-reporting --jq .enabled
   gh api -X PUT repos/jlevy/fdu/private-vulnerability-reporting
   ```

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
   whitespace. `scripts/release/release_body.py` does the strip, the unwrap, and the
   first two checks: exactly one HTML comment in the notes (the guideline footer), and a
   whitespace-only difference between the stripped source and the unwrapped body.
   A comment inside a code span or fence is documentation, not a draft leftover.

   ```shell
   git switch --detach <release-commit>
   uv run --no-project --python 3.12 python scripts/release/release_body.py \
     --notes docs/project/release-notes/0.1.0.md \
     --source "$RELEASE/notes-source.md" \
     --body "$RELEASE/notes.md"
   ```

   Check the rendered body before tagging, because a fix after the tag needs a new
   commit and so a new version.
   The command must print `0`, so GitHub’s renderer finds no line break inside a
   paragraph. Read `$RELEASE/notes.html` as well: it is the body as GitHub will render
   it.

   ```shell
   gh api markdown -f mode=gfm -F text=@"$RELEASE/notes.md" > "$RELEASE/notes.html" &&
     grep -c '<br>' "$RELEASE/notes.html"
   ```

2. Recheck that both crate names and the Python name are still free, before a pushed tag
   commits the version.
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

3. Create the signed tag on the release commit, and push it only if it verifies, with
   the key from [Prerequisites](#prerequisites):

   ```shell
   git tag -s v0.1.0 -m "fdu 0.1.0"
   git tag -v v0.1.0 && git push origin v0.1.0
   ```

4. Clone the pushed tag into a clean directory, and confirm that it names the rehearsed
   commit and the Cargo version:

   ```shell
   git clone --branch v0.1.0 https://github.com/jlevy/fdu "$RELEASE/fdu"
   cd "$RELEASE/fdu"
   uv run --no-project --python 3.12 python scripts/release/resolve_plan.py \
     --mode release --ref refs/tags/v0.1.0 --commit <release-commit> --validate-checkout
   ```

Every remaining command runs in `$RELEASE/fdu`, whose `rust-toolchain.toml` selects the
pinned Rust.

### Publish the Crates

1. Create the crates.io API token as [crates.io Account](#cratesio-account) describes:
   `publish-new`, `publish-update`, and `yank`, limited to `fdu-core` and `fdu`, with
   the shortest expiry crates.io offers.
   Read the token without echoing it or writing it to shell history:

   ```shell
   read -rs CARGO_REGISTRY_TOKEN && export CARGO_REGISTRY_TOKEN
   export FDU_RELEASE_TAG=v0.1.0
   ```

2. Reproduce both crates from the tag and compare them with the rehearsal’s digests.
   A mismatch means crates.io would receive bytes nothing tested: stop, and publish
   nothing until the difference is explained.
   On 2026-09-16 a maintainer’s `cargo package --locked --no-verify -p fdu-core -p fdu`
   on macOS arm64, with the pinned cargo 1.97.1, reproduced both `.crate` digests of
   Linux rehearsal run 35156068769 byte for byte, so a macOS host is not expected to
   differ. If a comparison still fails, and the extracted file trees are identical and
   only the archives differ, reproduce on Linux x86-64 with the pinned toolchain rather
   than relaxing the comparison.

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
   validate the tag as in [Tag the Release Commit](#tag-the-release-commit) step 4, then
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
   account settings. Trusted publishers wait until both crates exist, the token is gone,
   and the `release` environment is protected, as
   [After 0.1.0: Trusted Publishers](#after-010-trusted-publishers) describes.

### Publish the Python Distribution

1. Create the account-scoped PyPI API token as [PyPI Account](#pypi-account) describes,
   and read it the same way:

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
   `--no-build` and an explicit GIL-enabled `--python` make the check test a wheel:
   without them, a free-threaded default interpreter, which cannot install the `abi3`
   wheels, quietly builds the source distribution and the check passes having tested
   none. Run it on 3.12, the `abi3` floor, and again on 3.14. PyPI’s API and index can
   also trail an upload, so if the audit exits 3 for a `missing` release, or the install
   cannot find `fdu==0.1.0`, rerun both as in step 3 of Publish the Crates:

   ```shell
   uv run --no-project --python 3.12 python scripts/release/registry_state.py \
     --manifest "$RELEASE/files/release-manifest.json" --version 0.1.0 --channel pypi \
     --require-identical &&
     (cd "$RELEASE" &&
       uv tool run --no-config --no-build --python 3.12 --from fdu==0.1.0 fdu --version &&
       uv tool run --no-config --no-build --python 3.14 --from fdu==0.1.0 fdu --version)
   ```

4. Delete the token in the PyPI account settings.
   The trusted publisher waits for the protected `release` environment, as
   [After 0.1.0: Trusted Publishers](#after-010-trusted-publishers) describes.

### Announce the Release

Once every channel verifies, record the final registry state and attach it with the
evidence and artifacts to a GitHub release on the tag.
The body is `$RELEASE/notes.md`, derived from the release commit’s notes and checked in
step 1 of [Tag the Release Commit](#tag-the-release-commit); step 4 confirmed that the
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

### After Publishing

Check what users see first:

- [ ] docs.rs built both crates: [fdu-core](https://docs.rs/crate/fdu-core/0.1.0/builds)
  and [fdu](https://docs.rs/crate/fdu/0.1.0/builds) each show a successful build.
- [ ] The crates.io pages for [fdu-core](https://crates.io/crates/fdu-core) and
  [fdu](https://crates.io/crates/fdu) render their READMEs, and their links resolve.
- [ ] The [PyPI page](https://pypi.org/project/fdu/0.1.0/) renders the package README
  and lists the source distribution and five wheels.
- [ ] The [GitHub release](https://github.com/jlevy/fdu/releases/tag/v0.1.0) carries the
  eight artifacts (two crates, the source distribution, and five wheels) and the
  evidence: `registry-state.json`, `release-manifest.json`, and `SHA256SUMS`.
  `gh release view v0.1.0 --repo jlevy/fdu --json assets --jq '.assets | length'` prints
  `11`.

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
| `conflict` whose detail lists only `missing:` files | PyPI holds part of the release, from an interrupted upload or a JSON API that has not caught up. Nothing it holds is wrong. | Wait a few minutes and rerun the audit. If files are still missing, rerun the `uv publish` command from step 2 of Publish the Python Distribution; `--check-url` uploads only what PyPI lacks. |
| `conflict` listing a `hash mismatch:` or an `unexpected:` file | The registry holds bytes nothing tested, under a version that cannot be reused. | Follow the procedure below. |

A `conflict` with a hash mismatch or an unexpected file, on any channel, or any failure
whose fix needs a new commit, ends `0.1.0`: a published file cannot be replaced, the
pushed tag cannot move, and every channel carries one version.
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

   On that pass, [Tag the Release Commit](#tag-the-release-commit) step 2 still requires
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
