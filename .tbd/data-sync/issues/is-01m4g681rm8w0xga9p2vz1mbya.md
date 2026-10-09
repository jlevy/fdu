---
type: is
id: is-01m4g681rm8w0xga9p2vz1mbya
title: Refusal of --depth for a flat view prints an empty value
kind: bug
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-10-09T11:19:48.755Z
updated_at: 2026-10-09T11:19:48.755Z
---
fdu docs --depth 1 --format json prints: fdu: invalid --depth "": requires a hierarchical view. The refusal names the flag but quotes an empty value instead of the one given (1). Present in 0.4.0 too. Find where the depth refusal renders its value (RequestError rendering in query_request.rs / cli.rs refused()) and quote the caller's value, with a golden.
