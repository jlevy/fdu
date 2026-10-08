---
type: is
id: is-01m4efe7zqcg5ttphf74yv33pj
title: Exercise the announcement's demo-video read in a release rehearsal
kind: bug
status: open
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m47z5n33np3b15yena1k3hqt
created_at: 2026-10-08T19:22:00.054Z
updated_at: 2026-10-08T19:22:10.615Z
---
Deferred from PR #186 review A6 (https://github.com/jlevy/fdu/pull/186#issuecomment-6067330629). Rehearsals skip the release.yml announce job (if: needs.plan.outputs.publish == 'true'), so the CI-side read of docs/media/fdu-demo.mp4 at GITHUB_SHA (maintainer.stage_demo via scripts/release/announce.py) and the twelfth upload first run at publication. release-engineering-rules: a release path first exercised by a real tag is an untested production change. Proposal: a read-only rehearsal step, or a stop-before-GitHub mode on announce.py, that runs verify_kept + stage_demo + expected_assets on the downloaded artifacts and checks the asset count (11 or 12) without contents: write. Waits on: PR #186 merging (stage_demo exists only there). Residual risk is low: preflight's demo video line reads the same blob before tagging.
