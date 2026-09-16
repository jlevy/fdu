---
type: is
id: is-01m01cj7m8tfwapt8575agzmgn
title: "PR #26 review S2: dispatch release rehearsal to prove full matrix"
kind: task
status: closed
priority: 2
version: 8
labels:
  - release
dependencies:
  - type: blocks
    target: is-01kzg4c6vnh98mqrpkzw7ydne0
parent_id: is-01m01chg3gqm5sjf58mt5ng9zw
child_order_hints:
  - is-01m2nvy6w3zch74wcfbdcgfcbz
created_at: 2026-08-15T00:18:50.120Z
updated_at: 2026-09-16T22:14:53.865Z
closed_at: 2026-09-16T22:14:53.864Z
close_reason: "Release rehearsal run 35156068769 succeeded on main commit 5f2d36d0c9ba60d454112b322f8cb6816342c4ab: plan audit, both crate packages/install smoke, sdist build/install smoke, all five wheel legs (including Windows installed-wheel UTF-8 smoke), immutable artifact inspection, and read-only registry classification. Downloaded all retained artifacts and independently verified all eight payloads against SHA256SUMS; crates.io fdu-core/fdu and PyPI fdu each classify missing as expected. Run: https://github.com/jlevy/fdu/actions/runs/35156068769"
resolution: null
duplicate_of: null
---
Deferred: after R1/R11 land on the PR and merge, dispatch release.yml once so all five wheel legs, evidence, and registry classification actually run before fdu-9cf0.

## Notes

Dispatch is impossible until the workflow reaches main: GitHub only registers workflow_dispatch workflows present on the default branch (verified 2026-08-15 via the Actions workflow list; only ci.yml and performance-environment.yml are registered). Runner-label and deployment-target fixes for the matrix landed in 862190a on claude/fdu-pr-review-g8rsrm. After merge: dispatch release.yml once and confirm all five wheel legs, the evidence job, and registry classification pass before starting fdu-9cf0.

2026-09-16 LOCAL REHEARSAL GREEN at origin/main 16efcd0, macOS arm64, on the 0.1.0 candidate. `make release-test` (32 tests) and `make release-rehearse` both exit 0. Evidence gathered for the dispatch that this bead still owns:
- cargo package --locked -p fdu-core -p fdu: fdu-core 61 files / 2.7 MiB (586.7 KiB compressed, 600751 B, sha256 6fbeee45015eed6acd2cdb581b21055a427b43f213c70478b7a378b3546da08a); fdu 17 files / 263.8 KiB (80.2 KiB compressed, 82174 B, sha256 bc64b340a3f76ff887308412bac5d6794956cd81c43cf7773ee29512800af0b6).
- The packaged fdu pins fdu-core as a registry version, not a path, and its Cargo.lock records fdu-core's checksum as exactly the packaged fdu-core digest above. `readme = "../../README.md"` is copied into the crate byte-identically.
- Repackaging a second time, after every .rs file had its mtime touched, reproduced both digests byte for byte: cargo normalizes tar mtime/uid/gid/mode and zeroes the gzip MTIME with OS=0xff, so the runbook's step-2 digest comparison is sound on this host. The macOS-vs-Linux comparison the release process warns about is still untested and only the dispatch can settle it.
- The unpublished-sibling trap reproduces exactly as documented: `cargo package --locked -p fdu` alone exits 101 with "no matching package named `fdu-core` found / location searched: crates.io index".
- Host wheel is fdu-0.1.0-cp312-abi3-macosx_11_0_arm64.whl, Requires-Python >=3.12, License-Expression MIT, typed facade, py.typed, private _native.abi3.so, CycloneDX SBOM, console script fdu=fdu:_main. inspect_artifacts.py --require-release-matrix correctly refuses a single-host set ("expected exactly five wheels"), so only the dispatch can prove the five-wheel matrix.
- registry_state.py verified against the live registries: fdu-core, fdu and PyPI fdu all classify `missing` (exit 0; exit 3 under --require-identical). All three names are still free (crates.io and PyPI answer 404). The live 200 path was proven separately by driving the shipped code at a real published crate record: the record's checksum equals the published .crate digest, a matching manifest yields `identical` (exit 0) and a wrong digest yields `conflict` (exit 2).
So the local half of the rehearsal needs nothing further; what remains is exactly this bead's dispatch.

2026-09-16 THE DISPATCH BLOCKER IS CLEARED. release.yml is now registered on the default branch (gh workflow list: "Release rehearsal  active  334851160") and has never been run (gh run list --workflow release.yml is empty). CI on main 16efcd0 is green (run 35072033017). So the remaining work on this bead is the dispatch itself, which needs the user's explicit go-ahead.
