---
type: is
id: is-01m3snr9v8p6xr2mjpwfx3why6
title: Announce on GitHub can miss the draft it just created (list lags gh release create)
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-09-30T17:28:18.024Z
updated_at: 2026-09-30T17:28:18.024Z
---
In the 0.3.0 publishing run (https://github.com/jlevy/fdu/actions/runs/36749866706, attempt 1), `Announce on GitHub` failed with "GitHub did not return the release draft". Both registries had already published.

Cause: `scripts/release/announce.py` `announce()` runs `gh release create --draft` and then immediately calls `release_record()`, which lists `repos/{repo}/releases`. GitHub's list endpoint did not yet include the new draft, so `record` was None and the job raised. The draft existed, with the correct title and body and no assets. Re-run failed jobs (attempt 2) found it, uploaded all eleven assets and published, as the guide's resume path intends.

Fix direction: after the create, retry the listing with a short bounded backoff before failing (for example 5 tries over about 30 s). Alternatively, take the release id or URL that `gh release create` prints and read that record directly, still refusing duplicates. Add a test with a fake host whose first listing after the create returns no match. The failure is benign, since the rerun resumes, but it turns every such race into a failed release run and a manual step.
