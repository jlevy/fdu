---
type: is
id: is-01m3ke9g4x5phd2ssj6vhbdnv2
title: Python CachePolicy should let the engine name a retired policy's replacement
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-09-28T07:22:26.332Z
updated_at: 2026-09-28T07:22:26.332Z
---
REG-4 from the stack 141 regression review. fdu.CachePolicy('only') raises the enum's own ValueError before the engine's parse_cache_policy runs, so tests/parity/py/parity_cli.py:67-83 restates the three retired-value messages and the 'Retired Policies Name Their Replacement' parity sessions test the shim, not the package. Pass the raw string through fdu.report(cache=value) so the engine names the replacement, or expose the retired map from _native.contract().
