---
type: is
id: is-01m0ne12cwet7nwvwhrfgt31nr
title: Record the parity artifact in CI, not on a developer machine
kind: task
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m0n9pjsahc4wk6ek37syjct4
created_at: 2026-08-22T19:09:13.499Z
updated_at: 2026-08-22T19:09:13.499Z
---
make check cannot falsify the artifact's portability, and has now failed to twice.

The artifact is recorded on whatever machine ran make parity-update and verified on ubuntu in CI. Locally the gate compares a macOS recording against a macOS run, so every platform-dependent value agrees by construction. Only the Linux job can catch a leak, which means the local gate reports success on an artifact CI will reject.

Both escapes were the same value -- a cache snapshot encodes to 797 bytes on macOS and 745 on Linux -- and the second got through because closing fdu-1kw3 changed the text it appears in, from 'bytes=797' in a repr to '797 metadata bytes' from the real renderer, so a mask written for the old form silently stopped matching.

Options, roughly in order of preference:

1. Have the parity CI job run --update and fail if the artifact differs, uploading the regenerated file as an artifact so the fix is a download rather than a guess. Recording and verification then happen on the same platform.
2. Keep recording locally but add a portability check to run-parity: refuse to write an artifact containing an absolute path, or a bare integer next to a unit word, unless it is masked. Catches the class rather than the instance.
3. Run the parity job on the macos runner too, so a platform difference fails both ways and is visible immediately.

1 and 2 compose well: CI owns the recording, and the check stops an obviously unportable artifact before it is ever committed.
