---
type: is
id: is-01m0n9sbn9ek4bgz8m8fcw767t
title: Python API cannot express the CLI's one-shot cache contract
kind: feature
status: open
priority: 2
version: 1
labels: []
dependencies: []
parent_id: is-01m0n9pjsahc4wk6ek37syjct4
created_at: 2026-08-22T17:55:06.528Z
updated_at: 2026-08-22T17:55:06.528Z
---
Six parity sessions differ only in which tier answered: the CLI reports cold_scan where the Python surface reports warm_revalidate.

This is not a bug. The corpus documents it: 'A one-shot report never loads the snapshot for a metadata query: revalidating one stats every entry regardless, so the load would be added to the walk, never instead of it... Sessions opened through the library hold their index and do amortise the load; this is the one-shot contract only.'

The gap is that a Python caller has no way to ASK for the one-shot contract. fdu.open() always takes the session path. A tool that runs fdu once per invocation -- which is most of them -- would want the CLI's behaviour and cannot request it.

Either fdu.scan() should document itself as the one-shot contract and the shim use it, or open() should take the choice as a parameter. Whichever, the six deviations then either vanish or become explicitly justified rather than incidental.
