---
type: is
id: is-01m3qfdxw5sxhvpthzh1z40dmw
title: "0.3.0: mark counters::Counts #[non_exhaustive] so new counters stop being breaking changes"
kind: task
status: open
priority: 3
version: 2
labels: []
dependencies: []
parent_id: is-01m3pc4384h1zyw0hdjyzp3p7h
created_at: 2026-09-29T20:59:17.765Z
updated_at: 2026-09-29T22:00:17.715Z
---
Review finding R161-1 (#161): H171 added three public fields to counters::Counts, which has only public fields and no #[non_exhaustive], so every new counter is a semver-breaking change. The maintainer accepted the break (the release becomes 0.3.0 under the 0.2.2 plan's gate). Since 0.3.0 breaks Counts anyway, marking it #[non_exhaustive] in the same release costs nothing further and makes later counter additions compatible; external code keeps Counts::default() and field reads. Check that nothing outside fdu-core (the fdu CLI tests, perf_probe example, fdu-py) builds Counts with a struct literal. Maintainer decision.
