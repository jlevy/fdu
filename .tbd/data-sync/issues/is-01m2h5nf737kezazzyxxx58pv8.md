---
type: is
id: is-01m2h5nf737kezazzyxxx58pv8
title: Record the .gitignore observation default and its reasoning in fdu-design-principles.md
kind: task
status: open
priority: 2
version: 1
labels:
  - stack-followup
dependencies: []
created_at: 2026-09-14T23:57:30.719Z
updated_at: 2026-09-14T23:57:30.719Z
---
From PR #57 guideline review finding 1, option (c) (https://github.com/jlevy/fdu/pull/57#pullrequestreview-5203155952), noted as not done by delta review 5204082880. The read_controls default has gone on, then off, then on during 2026-09-14. rust-rules warns about this when Default makes a decision the caller should own. The user wants project decisions in project docs, not memory. Add a short principle stating: .gitignore handling is built in; it is observed by default on every surface; each request opts out with read_controls or --no-gitignore; an opted-out index answers with a typed 'not observed' and never with 'not ignored'. Include why: one default snapshot scope, and usable ignore roll-ups everywhere. Land it with the default-on PR (fdu-elnn), so the principle and the code agree. Beads: fdu-elnn, fdu-agb6, fdu-x7yb.
