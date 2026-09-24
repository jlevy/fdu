# Release Process

fdu has one product version and three public delivery surfaces: the `fdu-core` and `fdu`
crates, the `fdu` Python distribution, and GitHub release evidence.
The Cargo package version is authoritative.
The release tag, CLI, report generator, Python module, source distribution, wheels, and
release evidence must all identify that same version.

One workflow, [`release.yml`](../../../.github/workflows/release.yml), rehearses and
publishes. It builds both crates, the source distribution, and the five-wheel platform
matrix; smoke-tests every native artifact it can run; inspects metadata, typing,
licenses, and SBOMs; classifies each crate and the Python release on its registry as
missing, identical, or conflicting; and retains a checksum manifest.
Dispatched as it is by default, that is all it does.
Dispatched on the release tag with `publish` set, the same run then uploads exactly
those files, from one job, once the maintainer approves the protected `release`
environment. `0.1.0` is published that way, as
[Publishing a Release](#publishing-a-release) describes;
[Publishing 0.1.0 by Hand](#publishing-010-by-hand) is the fallback.

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
report (`fdu.report/7`), the watch stream (`fdu.stream/2`), and cache status
(`fdu.cache/2`) each version independently, as
[the surface architecture](../architecture/fdu-surface-architecture.md#machine-output-schemas)
lists. The bump rule protects consumers of a released schema.
A schema version that no published release has emitted yet is still a draft: it may
change in place before its first release, without a new number, and the CHANGELOG entry
for the release that first emits it describes its final shape.
Every release also strands the snapshots the previous one wrote, because the engine
fingerprint mixes in the crate version: after an upgrade `--cache-status` reports them
as `stale`, no run reuses them, and `--cache-clear` removes them.
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
Its `publish` input defaults to false, and a run without it never reaches a publishing
job.

## Account and Authentication Model

Use the same maintainer accounts that publish Flowmark, with publisher subjects created
specifically for this repository.
No Flowmark token or publisher record is reused.

| Channel | Required setup |
| --- | --- |
| GitHub Releases | Repository `jlevy/fdu`. The announcement stays a maintainer step, so no workflow job has `contents: write`; if it is automated, only that final job receives it. |
| PyPI, every release | Trusted publisher owner `jlevy`, repository `fdu`, top-level workflow `release.yml`, protected environment `release`. Registered on 2026-09-24 as a *pending* publisher, so the first upload through the workflow creates the project. The publish job receives `id-token: write` and no API token. |
| crates.io first release | The same crates.io owner creates `fdu-core` and `fdu` with a narrowly scoped, short-lived token, because a trusted publisher cannot be attached before the crate exists. The token lives only for the publishing run, as the `CARGO_REGISTRY_TOKEN` secret of the `release` environment; delete the secret and revoke the token afterwards. |
| crates.io later releases | Trusted publisher owner `jlevy`, repository `fdu`, workflow `release.yml`, environment `release`; exchange GitHub OIDC through `rust-lang/crates-io-auth-action` only inside the publish job. |

The first-time walkthrough for each row is
[First-Time Channel Setup](#first-time-channel-setup).

Crates.io publishing is authenticated in both cases.
The bootstrap uses the registry token; steady state exchanges the workflow’s OIDC
identity for a short-lived Cargo credential.
The publish job uses the environment secret when it is present and exchanges OIDC
otherwise, and prints neither.
Neither credential belongs in repository files, logs, build artifacts, or reusable
workflows.

A trusted publisher trusts any run of `release.yml` that names the `release`
environment. GitHub creates an unprotected environment the first time a workflow job
names it, so the environment has to exist and be protected before the first publishing
run: a required reviewer, a deployment policy that admits only `v*` tags, and no
administrator bypass.
The pending PyPI publisher already exists, so this is not optional.
A publishing run checks it before its publish job can start and stops if any of the
three is missing.

## Publication Invariants

Every upload consumes only the validated artifact set.
PyPI receives the tested source distribution and wheels without rebuilding.
Cargo is the narrow exception because `cargo publish` repackages source: reproduce the
validated `.crate`, compare its SHA-256 digest with the retained preview, and abort on
any mismatch before upload.
Every registry write happens in one `publish` job behind the protected `release`
environment, so one approval covers both registries; no build job holds publication
authority. That job compiles nothing: both crates were built and installed by the
rehearsal, and `--no-verify` keeps dependency build scripts and proc macros from running
beside the credentials.

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
The accounts, the PyPI pending publisher, and the GitHub environment can be finished
days before the tag.
The crates.io token cannot: it is created on publish day, stored in the `release`
environment for the one publishing run, and deleted and revoked as soon as that run
ends, as [Publish Through the Workflow](#publish-through-the-workflow) describes.

Recheck the three names immediately before the first write; availability is a race.
Use the JSON and crates.io API curls in
[Tag the Release Commit](#tag-the-release-commit) step 2, not the HTML project pages.
`https://pypi.org/project/fdu/` can return HTTP 200 with an anti-bot interstitial for a
name that does not exist.

### Why crates.io Needs a Token Once

crates.io will not accept a
[trusted publisher](https://crates.io/docs/trusted-publishing) until the crate exists,
so `fdu-core` and `fdu` must be created with an API token.
The publish job reads it from the `release` environment’s `CARGO_REGISTRY_TOKEN` secret
when that secret exists, and otherwise exchanges OIDC, so the same workflow serves the
first release and every later one.

PyPI
[can create a project from a pending trusted publisher](https://docs.pypi.org/trusted-publishers/creating-a-project-through-oidc/),
and this repository uses that path: the pending publisher for `fdu` was registered on
2026-09-24, and the first upload through the workflow creates the project.
A pending publisher does not reserve the name, so the name recheck still applies.
It also trusts whatever job names the `release` environment, which is why that
environment has to be protected before any run can publish.

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

- **Scopes:** `publish-new` and `publish-update`. Leave `yank` and `change-owners` off:
  nobody can read a token back out of a GitHub secret, so a `yank` scope there could
  never be used. Containing a conflict takes a separate `yank`-only token, as
  [Recover From a Partial Publication](#recover-from-a-partial-publication) step 3 says.
- **Crates:** restrict to `fdu-core` and `fdu`. A crate-name restriction applies to
  future crates the account owns, so the names may be listed before they exist.
- **Expiry:** the shortest preset crates.io offers, or a custom date that covers only
  the publish window.

Store it only as the `release` environment’s `CARGO_REGISTRY_TOKEN` secret, as
[Publish Through the Workflow](#publish-through-the-workflow) step 1 describes.
Do not run `cargo login`, and do not store it in `~/.cargo/credentials.toml` or as a
repository secret, which every job of every workflow could read.

### PyPI Account

1. Sign in with the same PyPI account that publishes Flowmark.
   Do not create a second account for fdu.
2. [Two-factor authentication](https://blog.pypi.org/posts/2024-01-01-2fa-enforced/) is
   required for every management action and every upload.
   Enable it under [Account settings](https://pypi.org/manage/account/) if it is not
   already on.
3. Confirm the account email is verified.
4. Confirm the pending publisher is listed on
   [the account’s publishing page](https://pypi.org/manage/account/publishing/) with
   exactly the subjects in
   [After 0.1.0: Trusted Publishers](#after-010-trusted-publishers).

No PyPI token is created for the workflow.
Only [Publishing 0.1.0 by Hand](#publishing-010-by-hand) needs one.

### GitHub `release` Environment

Create the protected `release` environment before the first publishing run.
GitHub creates an *unprotected* environment the first time a workflow job names one, and
the pending PyPI publisher would trust it.

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

The workflow’s `release-environment` job reads these three settings back through the
GitHub API before the publish job can start, and stops the run if any is missing.

The environment holds one secret, and only for the `0.1.0` run: `CARGO_REGISTRY_TOKEN`.
Never add a PyPI token to it.
Later runs exchange OIDC and hold no long-lived registry credential.

### After 0.1.0: Trusted Publishers

PyPI needs nothing more: the first upload turned the pending publisher into the `fdu`
project’s publisher.
Confirm it is listed on
[the project’s publishing page](https://pypi.org/manage/project/fdu/settings/publishing/),
and add no second one.

crates.io needs one publisher per crate, and only after all three of these are true:

1. `fdu-core` and `fdu` exist on crates.io.
2. The `release` environment exists and is protected as above.
3. The `CARGO_REGISTRY_TOKEN` secret is deleted and the token revoked.

Use these subjects on every record; they are specific to this repository, not copied
from Flowmark:

| Field | Value |
| --- | --- |
| Owner | `jlevy` |
| Repository | `fdu` |
| Workflow filename | `release.yml` (the top-level file; not a reusable workflow) |
| Environment | `release` |

On crates.io, open each crate’s settings and add a GitHub Actions trusted publisher with
those fields. Registering a publisher does not publish anything.
From then on the publish job finds no secret and exchanges OIDC through
`rust-lang/crates-io-auth-action`; a run that finds neither a secret nor a publisher
fails before its first upload.

### Publication Sequence

The first release is this order.
Channel setup is the only block that can finish before the release commit exists.

1. Finish this section: accounts, 2FA, the pending PyPI publisher, and the protected
   `release` environment.
2. Merge everything the release needs onto `main`.
3. [Rehearse the Release Commit](#rehearse-the-release-commit).
4. [Tag the Release Commit](#tag-the-release-commit), including the name recheck.
5. [Publish Through the Workflow](#publish-through-the-workflow): store the crates.io
   token, dispatch on the tag with `publish` set, approve, then delete the secret and
   revoke the token.
6. [Announce the Release](#announce-the-release).
7. Work through [After Publishing](#after-publishing).
8. Add the crates.io [trusted publishers](#after-010-trusted-publishers).

Later releases skip steps 1 and 8, and step 5 has no token.
If the workflow cannot publish, [Publishing 0.1.0 by Hand](#publishing-010-by-hand)
replaces step 5.

## Publishing a Release

A maintainer publishes from the signed tag: `fdu-core`, then `fdu`, then the Python
distribution.
[First-Time Channel Setup](#first-time-channel-setup) must already be done:
the accounts exist, 2FA is on, and the `release` environment is protected.
Every upload carries bytes the rehearsal validated, and each registry is checked against
the rehearsal’s manifest before the next write.

The commands assume bash or zsh, with `gh`, `uv`, `rustup`, and `curl`: the by-hand
token prompts use `read -s`, which a plain POSIX `sh` such as `dash` rejects.
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
   announced: it is what [Publishing 0.1.0 by Hand](#publishing-010-by-hand) uploads if
   the workflow cannot publish at all.

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
   crates.io also refuses a new crate whose name differs from an existing one only by
   `-` against `_`, so `fdu_core` is checked as well.
   Each command must print `404`:

   ```shell
   curl -sS -o /dev/null -w '%{http_code}\n' -A 'fdu-release (https://github.com/jlevy/fdu)' \
     https://crates.io/api/v1/crates/fdu-core
   curl -sS -o /dev/null -w '%{http_code}\n' -A 'fdu-release (https://github.com/jlevy/fdu)' \
     https://crates.io/api/v1/crates/fdu_core
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

### Publish Through the Workflow

The publishing run rebuilds, smoke-tests, and inspects every artifact from the tag, then
uploads exactly those files from one job once you approve the `release` environment.
Before each upload the job proves what it is about to send:

| Before | The job checks |
| --- | --- |
| Anything | Its own checkout is the tag and the commit the plan resolved, the environment check passed, and the downloaded crates, source distribution, and five wheels are exactly the files in the run’s manifest, inspected again, each with the recorded size and SHA-256, matching `SHA256SUMS` too. |
| The first write | Both registries are audited. An `identical` version is skipped, a `missing` one is published, and any conflict on either registry stops the job, so a PyPI conflict stops crates.io from being written first. |
| Each crate | `cargo package --locked --no-verify` reproduces it from the tag, and its digest must equal the manifest’s. `fdu` is reproduced again against the published `fdu-core`. |
| `fdu` and PyPI | For up to ten minutes, the job waits for crates.io to serve the manifest’s digest for the crate just published, in both the API record and the sparse index Cargo resolves from. Another digest in either stops the job. |
| The end | PyPI lists exactly the manifest’s files and digests, and `registry_state.py --require-identical` passes for every registry. |

1. **`0.1.0` only: store the crates.io token.** Create it as
   [crates.io Account](#cratesio-account) describes, and add it as the `release`
   environment’s `CARGO_REGISTRY_TOKEN` secret: Settings → Environments → `release` →
   Add environment secret.
   Or run the command below and paste the token when `gh` prompts for it, so it never
   reaches shell history.
   Later releases skip this step: with no secret, the job exchanges OIDC.

   ```shell
   gh secret set CARGO_REGISTRY_TOKEN --env release --repo jlevy/fdu
   ```

2. **Dispatch the publishing run on the tag**, and record its ID as `<publish-run-id>`.
   The plan job fails at once unless the ref is `refs/tags/v0.1.0`, that tag names the
   checked-out commit, and the Cargo version is `0.1.0`.

   ```shell
   gh workflow run release.yml --repo jlevy/fdu --ref v0.1.0 -f publish=true
   gh run list --repo jlevy/fdu --workflow release.yml --limit 1
   ```

3. **Check the run, then approve it once.** When the `Publish to crates.io and PyPI` job
   is *Waiting*, the builds, smoke tests, inspection, and the `release-environment`
   check have passed. Confirm the run is the tag and the release commit: the command must
   print `workflow_dispatch`, `v0.1.0`, and `<release-commit>`.

   ```shell
   gh run view <publish-run-id> --repo jlevy/fdu --json event,headBranch,headSha \
     --jq '[.event, .headBranch, .headSha] | join(" ")'
   ```

   Then open the run on GitHub, choose Review deployments, select `release`, and
   approve. That one approval covers both registries; nothing is uploaded before it.

4. **Watch the job to the end:**

   ```shell
   gh run watch <publish-run-id> --repo jlevy/fdu --exit-status
   ```

   If it fails, go to
   [Recover From a Partial Publication](#recover-from-a-partial-publication) before
   anything else.

5. **`0.1.0` only: remove the token.** Once the job has passed
   `Wait until crates.io serves the rehearsed fdu`, both crates are published and no
   rerun needs the token again: a rerun finds them `identical` and skips the credential
   step. Delete the secret, then revoke the token in the
   [crates.io token settings](https://crates.io/settings/tokens):

   ```shell
   gh secret delete CARGO_REGISTRY_TOKEN --env release --repo jlevy/fdu
   ```

6. **Keep the published files.** Download the publishing run’s artifacts, as in
   [Rehearse the Release Commit](#rehearse-the-release-commit) step 3, into
   `$RELEASE/published`. They are what the registries hold, and what the announcement
   attaches. Each run builds its own wheels and source distribution, so these need not
   match the rehearsal’s in `$RELEASE/files`.

   ```shell
   gh run download <publish-run-id> --repo jlevy/fdu --dir "$RELEASE/published-download"
   mkdir "$RELEASE/published"
   find "$RELEASE/published-download" -type f -exec cp {} "$RELEASE/published/" \;
   (cd "$RELEASE/published" && shasum -a 256 -c SHA256SUMS)
   ```

### Announce the Release

Once every channel verifies, record the final registry state and attach it with the
evidence and artifacts to a GitHub release on the tag.
This stays a maintainer step; the workflow never writes to the repository.
The files are `$RELEASE/published`, the set the registries hold.
The body is `$RELEASE/notes.md`, derived from the release commit’s notes and checked in
step 1 of [Tag the Release Commit](#tag-the-release-commit); step 4 confirmed that the
tag names that commit, so the body is the tagged text.
The release is created only if the audit exits 0, which with `--require-identical` means
every channel holds exactly the rehearsal’s files:

```shell
uv run --no-project --python 3.12 python scripts/release/registry_state.py \
  --manifest "$RELEASE/published/release-manifest.json" --version 0.1.0 \
  --require-identical --output "$RELEASE/registry-state.json" &&
  gh release create v0.1.0 --repo jlevy/fdu --verify-tag --title "fdu 0.1.0" \
    --notes-file "$RELEASE/notes.md" \
    "$RELEASE/registry-state.json" "$RELEASE"/published/release-manifest.json \
    "$RELEASE"/published/SHA256SUMS "$RELEASE"/published/*.crate \
    "$RELEASE"/published/*.whl "$RELEASE"/published/fdu-0.1.0.tar.gz
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

Then install it the ways a user does, from outside any checkout.
Each command must print `fdu 0.1.0`. If your uv configuration sets an `exclude-newer`
cool-off, add `--no-config` to the `uv` commands, or the cool-off hides a release
published minutes ago.

- [ ] `uvx` runs the newest release: `uvx fdu@latest --version`.
- [ ] `uv tool install fdu` installs it:
  `uv tool install fdu && "$(uv tool dir --bin)/fdu" --version`, then
  `uv tool uninstall fdu`.
- [ ] `cargo install --locked fdu` builds it from crates.io:
  `cargo install --locked fdu --root "$RELEASE/cargo-user" && "$RELEASE/cargo-user/bin/fdu" --version`.
- [ ] `fdu --install-skill`, run from one of those installs, installs the agent skill.
  The installer lands in its own pull request; skip this line if that did not merge
  before the tag.

### Recover From a Partial Publication

Neither crates.io nor PyPI lets a version’s files be replaced, even after a yank or a
deletion. So when any step fails, first audit the registries against the manifest of the
files being published, and let the verdict decide what comes next.
The publish job’s log already holds one: its audit steps print each registry’s state,
and a guard that stops prints `conflict:` or `timeout:` with the digests it saw.
Confirm it from `$RELEASE/fdu`, with `$RELEASE/published` from
[Publish Through the Workflow](#publish-through-the-workflow) step 6 (download it now if
the run failed before that step):

```shell
uv run --no-project --python 3.12 python scripts/release/registry_state.py \
  --manifest "$RELEASE/published/release-manifest.json" --version 0.1.0
```

An audit that cannot read a registry stops with an error naming the URL and exits 1.
That is no verdict: rerun the audit, and never read it as `missing`.

| Audit reports | Meaning | Next step |
| --- | --- | --- |
| `missing`, after the lag retry in step 3 of Publish the Crates | Nothing reached the registry. | Fix the cause, then rerun: the failed publish job, or by hand the failed step from its start, so a crate is compared again before it is published. |
| `identical` | The upload landed, though the command reported a failure. | Rerun the failed publish job, which skips it; by hand, continue with the next step. |
| `conflict` whose detail lists only `missing:` files | PyPI holds part of the release, from an interrupted upload or a JSON API that has not caught up. Nothing it holds is wrong. | Wait a few minutes and rerun the audit. If files are still missing, rerun the failed publish job, or by hand the `uv publish` command from step 2 of Publish the Python Distribution; either way `--check-url` uploads only what PyPI lacks. |
| `conflict` listing a `hash mismatch:` or an `unexpected:` file | The registry holds bytes nothing tested, under a version that cannot be reused. | Follow the procedure below. |

Rerun a failed publish job with Re-run failed jobs on the same workflow run, and approve
it again.
The rerun uploads the files that run inspected, so it can finish what the first
attempt started. Never dispatch a new publishing run once PyPI holds any file of the
version: a new run builds its own wheels and source distribution, which need not match
the uploaded ones, and its evidence job stops on a partially published PyPI release in
any case. Artifacts expire after 90 days; past that, finish by hand from
`$RELEASE/published`. A failure whose fix changes `release.yml` cannot be rerun either,
because the tag runs the workflow it names; finish that release by hand as well.

A `conflict` with a hash mismatch or an unexpected file, on any channel, or any failure
whose fix needs a new commit, ends `0.1.0`: a published file cannot be replaced, the
pushed tag cannot move, and every channel carries one version.
Nothing this process writes can conflict before `fdu-core` is on crates.io.

1. **Publish nothing more under `0.1.0`.** Run no later step.
   Above all, never publish `fdu` against an `fdu-core` that conflicts.

2. **Record what happened while the evidence is fresh**, in a bead or a GitHub issue:

   - the audit’s output;
   - the published digest of each conflicting crate beside the manifest’s, from
     `curl -fsSL -A 'fdu-release (https://github.com/jlevy/fdu)' https://crates.io/api/v1/crates/<crate>/0.1.0/download | shasum -a 256`;
   - where it was published: the publishing run’s ID and its log, or by hand the host
     and toolchain (`uname -a`, `cargo -V`) and the digests step 2 of Publish the Crates
     printed;
   - the release commit, the rehearsal’s run ID, and every version yanked.

   Keep `$RELEASE`, including a hand publication’s `target/package`, which holds the
   archives `cargo publish` built and uploaded.

3. **Yank each version whose published bytes are wrong, and only those.**
   `cargo yank fdu-core@0.1.0` (or `fdu@0.1.0`) needs a token with the `yank` scope: by
   hand, the one from step 1 of Publish the Crates; after a workflow publication, a new
   token limited to `yank` on `fdu-core` and `fdu` with the shortest expiry, revoked
   afterwards. On PyPI, yank the release from the project’s settings.
   A version the audit reports as `identical` holds the tested bytes and stays.
   A yank stops new resolution; it deletes nothing and does not break an existing
   lockfile.

4. **Never retag.** `v0.1.0` keeps naming the release commit, and nothing is published
   again under `0.1.0` on any channel.

5. **Release `0.1.1` instead.** Bump the version in a pull request, including every
   manifest and the expected version in `tests/release/test_metadata.py`, and give the
   CHANGELOG a `0.1.1` entry that says which `0.1.0` artifacts exist and which were
   yanked. Once it merges, repeat [Publishing a Release](#publishing-a-release) from
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

Whatever the outcome, delete the `CARGO_REGISTRY_TOKEN` environment secret if it still
exists, and unset and revoke every token as the steps above describe.

## Publishing 0.1.0 by Hand

This is the fallback for when the workflow cannot publish: the `release` environment
cannot be set up in time, or the publish job fails in a way a rerun cannot fix, such as
a fix that changes `release.yml`, which the tag cannot pick up without a new commit and
so a new version. It holds registry tokens in a shell instead of the environment.

It uploads the files any registry may already hold, so that nothing published twice can
differ. If a publishing run uploaded anything, those are its files in
`$RELEASE/published`, from [Publish Through the Workflow](#publish-through-the-workflow)
step 6; a crate it published is then `identical` and skipped.
Otherwise they are the rehearsal’s files from
[Rehearse the Release Commit](#rehearse-the-release-commit) step 3:

```shell
[ -d "$RELEASE/published" ] || cp -R "$RELEASE/files" "$RELEASE/published"
```

Every command below runs in `$RELEASE/fdu`, from
[Tag the Release Commit](#tag-the-release-commit) step 4. Afterwards, continue with
[Announce the Release](#announce-the-release).

### Publish the Crates

1. Create a crates.io API token as [crates.io Account](#cratesio-account) describes,
   adding the `yank` scope so a conflict can be contained without minting a second
   token: `publish-new`, `publish-update`, and `yank`, limited to `fdu-core` and `fdu`,
   with the shortest expiry crates.io offers.
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
   (cd target/package && grep '\.crate$' "$RELEASE/published/SHA256SUMS" | shasum -a 256 -c -)
   ```

   A match is evidence about this host and this tree only, and Cargo cannot upload
   anything else: `cargo publish` takes no archive argument and repackages from the
   checkout every time.
   So the host whose digests matched is the host that publishes.
   If only a Linux reproduction matches, do all of Publish the Crates on that Linux
   host: download and check the files being published there, as in
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
     --manifest "$RELEASE/published/release-manifest.json" --version 0.1.0 --channel crates.io
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
   (cd target/package && grep ' fdu-0\.1\.0\.crate$' "$RELEASE/published/SHA256SUMS" | shasum -a 256 -c -)
   cargo publish --locked -p fdu
   uv run --no-project --python 3.12 python scripts/release/registry_state.py \
     --manifest "$RELEASE/published/release-manifest.json" --version 0.1.0 --channel crates.io \
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

1. Create a PyPI API token at [Add API token](https://pypi.org/manage/account/token/),
   named for this one upload, for example `fdu-0.1.0-bootstrap`. Scope it to the `fdu`
   project if the project exists; otherwise it has to be Entire account, because a
   project-scoped token needs an existing project.
   `uv publish` sends it as the password with username `__token__`. Do not write a
   `.pypirc`, and do not store it in a GitHub secret.
   Read it the same way:

   ```shell
   read -rs UV_PUBLISH_TOKEN && export UV_PUBLISH_TOKEN
   ```

2. Upload the rehearsal’s source distribution and five wheels, never a rebuild.
   `--check-url` lets a rerun skip files PyPI already holds.

   ```shell
   uv publish --trusted-publishing never --check-url https://pypi.org/simple/ \
     "$RELEASE"/published/fdu-0.1.0.tar.gz "$RELEASE"/published/fdu-0.1.0-*.whl
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
     --manifest "$RELEASE/published/release-manifest.json" --version 0.1.0 --channel pypi \
     --require-identical &&
     (cd "$RELEASE" &&
       uv tool run --no-config --no-build --python 3.12 --from fdu==0.1.0 fdu --version &&
       uv tool run --no-config --no-build --python 3.14 --from fdu==0.1.0 fdu --version)
   ```

4. Delete the token in the PyPI account settings.
   Then open
   [the project’s publishing page](https://pypi.org/manage/project/fdu/settings/publishing/):
   if a hand upload created the project, the pending publisher may not have carried
   over. If it is not listed, add it there with the subjects in
   [After 0.1.0: Trusted Publishers](#after-010-trusted-publishers).

The implementation audit, Flowmark comparison, deliberate divergences, and proposed
upstream improvements live in the
[release packaging and Python API plan](../specs/active/plan-2026-08-14-fdu-release-packaging-python-api-polish.md).

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
