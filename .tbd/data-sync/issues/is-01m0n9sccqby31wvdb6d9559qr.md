---
type: is
id: is-01m0n9sccqby31wvdb6d9559qr
title: Parity shim cannot serve --docs, and that is the whole skip list
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m0n9pjsahc4wk6ek37syjct4
created_at: 2026-08-22T17:55:07.286Z
updated_at: 2026-08-22T17:55:07.286Z
---
Two sessions are declined rather than compared: the bare invocation plus --docs and --skill (clap's help rendering and two static documents the package does not carry), and --help.

Recorded here so the skip list is visible rather than implicit. Growth in it is a regression worth arguing about; a skip list of three discovery surfaces is the measurement that the Python API is close to complete.

The --version deviation is deliberate and must stay: it is what keeps the artifact non-empty by construction, so an empty diff means the shim never ran.
