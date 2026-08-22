---
type: is
id: is-01m0m3fa15x11zc5vcsz0naw4g
title: Python API cannot render cache status or watch records the way the CLI does
kind: feature
status: closed
priority: 2
version: 2
labels: []
dependencies: []
created_at: 2026-08-22T06:45:31.300Z
updated_at: 2026-08-22T18:26:53.885Z
closed_at: 2026-08-22T18:26:53.884Z
close_reason: |-
  Cache status renders through the library now. The human layout moved from cli.rs into report_format::render_cache_status beside every other human layout -- its own comment used to say 'Text is rendered by the CLI, which owns the human layout', which is exactly why no other caller could print what fdu prints. The CLI now delegates every format, including text, and all 129 goldens pass unchanged.

  fdu.render_cache_status(statuses, format) exposes it. Seven parity sessions closed.

  Watch record rendering is NOT closed by this and is split out as its own item -- a Change has no renderer, so the two watch sessions still differ.
---
Found by the parity shim: 16 of 38 recorded deviations are this one gap.

fdu.cache_status/list_caches return CacheStatus values and Index.watch yields Change values, but nothing turns either into the CLI's bytes. Report.render exists now; these have no equivalent, so a Python caller wanting fdu's cache-status or watch output must reimplement the formatting or shell out.

The shim currently prints repr(), which is why those sessions deviate:
  +CacheStatus(path=PosixPath('.cache/fdu/[HASH].fdu'), bytes=797, ...)
against the CLI's formatted line.

This is the same gap Report.render closed for reports, in the two surfaces that were already public but still unprintable. Closing it shrinks the deviation file, which is the measurement.
