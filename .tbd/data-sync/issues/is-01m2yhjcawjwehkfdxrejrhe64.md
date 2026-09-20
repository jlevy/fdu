---
type: is
id: is-01m2yhjcawjwehkfdxrejrhe64
title: Python watch interval accepts nonfinite values and can panic in native conversion
kind: bug
status: open
priority: 1
version: 1
labels: []
dependencies: []
parent_id: is-01m2yh8kc79nw7bn6k6xw8g3bp
created_at: 2026-09-20T04:35:39.995Z
updated_at: 2026-09-20T04:35:39.995Z
---
WatchOptions only checks interval<=0; NaN/infinity pass, then native lib.rs Duration::from_secs_f64 can panic. Reject nonfinite and out-of-range values with ordinary argument errors, including direct native boundary calls, and add focused tests.
