---
type: is
id: is-01m2yhtwjg6j0vjyjf1v9sehd5
title: Preserve native path identity in cache status output
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-20T04:40:18.759Z
updated_at: 2026-09-20T04:40:18.759Z
---
Cache status currently renders cache paths and roots through lossy strings without raw companions. Add raw path identity to fdu.cache/2 rows through the shared emission policy and test non-Unicode cache/root paths.
