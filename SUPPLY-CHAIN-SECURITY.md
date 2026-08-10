# Supply-chain security

This repository delays new executable dependencies for at least 14 days, installs from
committed lockfiles, disables npm lifecycle scripts, pins GitHub Actions to reviewed
commits, and treats missing provenance as a hard failure.

Run `make supply-chain` before installing or updating dependencies.
The zero-dependency validator checks Cargo checksums and crate publication dates, npm
tarballs and integrities, PyPI artifact hashes and upload times, action commit
signatures and dates, toolchain manifests, runtime checksum manifests, and explicit
bootstrap assets against their authoritative services.
It also checks that pull-request jobs use read-only permissions, do not persist checkout
credentials, do not write reusable caches, and do not execute project dependencies
before the provenance job succeeds.
GitHub API checks use `GITHUB_TOKEN` or `GH_TOKEN` when explicitly provided and
otherwise reuse the local GitHub CLI credential without invoking a shell or printing it.
CI passes only its read-only workflow token to the provenance step, avoiding the shared
unauthenticated API quota without broadening repository permissions.

Dependency changes must be narrow and intentional:

1. Confirm the dependency is necessary and inspect its source and ownership.
2. Select a version that has been public for at least 14 full days.
3. Regenerate only the relevant lockfile and review the source and lock diffs.
4. Run `make supply-chain`, the ecosystem audit, and `make check`.
5. Commit every changed lockfile with the manifest change.

Exceptions require a specific version, reason, prior maintainer approval, expiration,
and follow-up in `supply-chain-policy.json`. An exception can waive release age only; it
can never waive missing or mismatched checksums, integrity, timestamps, or source
provenance. Agents do not approve new exceptions.

The GitHub CLI bootstrap is never run by session startup.
In a disposable environment, invoke it explicitly with
`FDU_BOOTSTRAP_GH_CLI=1 scripts/bootstrap-gh-cli.sh`; the script refuses unsupported
platforms and verifies the downloaded asset before extraction.

The governing policy is the tbd `supply-chain-hardening` guideline and the linked
[Supply Chain Hardening guidebook](https://github.com/jlevy/supply-chain-hardening).

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
