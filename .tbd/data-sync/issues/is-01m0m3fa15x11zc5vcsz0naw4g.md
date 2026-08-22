---
type: is
id: is-01m0m3fa15x11zc5vcsz0naw4g
title: Python API cannot render cache status or watch records the way the CLI does
kind: feature
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-08-22T06:45:31.300Z
updated_at: 2026-08-22T06:45:31.300Z
---
Found by the parity shim: 16 of 38 recorded deviations are this one gap.

fdu.cache_status/list_caches return CacheStatus values and Index.watch yields Change values, but nothing turns either into the CLI's bytes. Report.render exists now; these have no equivalent, so a Python caller wanting fdu's cache-status or watch output must reimplement the formatting or shell out.

The shim currently prints repr(), which is why those sessions deviate:
  +CacheStatus(path=PosixPath('.cache/fdu/[HASH].fdu'), bytes=797, ...)
against the CLI's formatted line.

This is the same gap Report.render closed for reports, in the two surfaces that were already public but still unprintable. Closing it shrinks the deviation file, which is the measurement.
