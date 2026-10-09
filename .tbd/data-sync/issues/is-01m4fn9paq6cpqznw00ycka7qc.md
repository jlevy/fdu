---
type: is
id: is-01m4fn9paq6cpqznw00ycka7qc
title: require_draft's notes-mismatch message points at the wrong notes
kind: bug
status: open
priority: 2
version: 1
labels:
  - release
dependencies: []
parent_id: is-01m47z5n33np3b15yena1k3hqt
created_at: 2026-10-09T06:23:36.790Z
updated_at: 2026-10-09T06:23:36.790Z
---
Review B B2 on #189 (https://github.com/jlevy/fdu/pull/189#issuecomment-6075575482): maintainer.py:1499-1522 tells the maintainer to edit the draft to $RELEASE/notes.md, which makes the job's rerun fail on identity when the workflow created the draft; point it at the workflow's announcement-notes-vX artifact, as the guide's procedure (release-process.md:330-340) says.
