---
type: is
id: is-01m0prhcmhj09n41s1zae35yhm
title: "Session integration shape: mid-walk progress, async form, session-to-watch clock handoff"
kind: task
status: open
priority: 1
version: 5
spec_path: docs/project/specs/active/plan-2026-08-23-fdu-interactive-client-implementation.md
labels: []
dependencies:
  - type: blocks
    target: is-01m0qs0msk75k8r89b44vqqjnz
parent_id: is-01m0prgbradma67z3j1wfyh8r7
created_at: 2026-08-23T07:32:08.465Z
updated_at: 2026-08-24T00:53:42.172Z
---
Three requirements that land with the progressive-results session, not after it: progress readable mid-walk (entries applied, clock, completeness) for crawl-status UIs; the async shape shipping with the sync one (same adapter policy as watch); and the walk-complete clock being the clock a watch resumes from, tested for the no-gap property.

## Notes

NAMING CONSTRAINT FROM THE RECONCILIATION, recorded here because it binds this bead's
types and nothing else's: "Distinguish the fdu progressive and watch lifecycle type names
before exposing them beside Metabrowser's InventoryHandle."

The session and watch lifecycle types this bead introduces will sit next to metabrowser's
own handle type in one Python namespace. Two things called some variant of "handle" or
"state", meaning different things on either side of a binding, is a defect that only ever
shows up in a reader's head. Pick names that survive being imported next to theirs.

Blocked by fdu-e86o and fdu-a0j0, which land the session core in the progressive-results
epic. Nothing in this bead can be built against a session that does not exist, and
inventing one in the command line is what the surface architecture forbids.
