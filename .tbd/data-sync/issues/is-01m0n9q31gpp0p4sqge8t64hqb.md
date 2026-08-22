---
type: is
id: is-01m0n9q31gpp0p4sqge8t64hqb
title: Report.render omits the CLI's display notes
kind: bug
status: open
priority: 1
version: 2
labels: []
dependencies: []
parent_id: is-01m0n9pjsahc4wk6ek37syjct4
created_at: 2026-08-22T17:53:52.175Z
updated_at: 2026-08-22T18:55:23.456Z
---
Three parity sessions differ only by a missing note line, e.g.

  note: --analyze lines,code,words read 176 B; no selected view displays content metrics — try --view families, languages, or all

The CLI prints these; Report.render does not. They are not decoration: this one tells a user their analysis work produced nothing visible, which is the exact defect the content axis was introduced to prevent. A Python caller rendering a report gets no such warning.

Either the notes belong in the rendered output (so render() is genuinely what the CLI writes) or they belong on the Report as data the caller can inspect. Probably both -- data on the value, included when rendering text.

## Notes

Design settled, and split by what the note actually depends on.

TWO NOTES, NOT ONE:

1. 'note: omitted documents -- requires content analysis: add --analyze ...'
   Depends only on the resolved views and the analysis set. Both are things the library
   already computes -- ViewSpec::resolve returns (selected, omitted) -- so this belongs on
   the Report and inside render(). One parity session ('Every View One Walk Can Answer')
   differs solely because of it, and fixing it also clears a confusing block from the
   deviation artifact, because a failed block prints its expectations with patterns
   unexpanded and that looks like a [SEP] mismatch that is not real.

2. 'note: --analyze lines,code,words read 176 B; no selected view displays content
   metrics -- try --view families, ...'
   Needs bytes_read, which is walk telemetry. Verified the report envelope carries no such
   field: keys are schema, generator, root, scan_started_at, generated_at, source,
   freshness, complete, errors, analysis, reports. That exclusion is deliberate and is the
   same recorded gap as the performance footer, so this note stays with the footer and
   with the CLI. Three sessions, and they are not defects.

AN ORDERING FINDING WORTH FIXING WITH IT:

The note currently prints AFTER the performance footer:

  <report body>
  Performance: walked 1 file / 3 B; ...
  note: omitted documents -- requires content analysis: ...

which reads oddly -- the footer looks like a terminator and then something follows it.
Body, then notes, then footer is the better order, and it falls out naturally if render()
owns note 1 and the CLI keeps appending the footer last.

PLAN:
  - Query carries omitted_views, set by ViewSpec::resolve (the CLI's ResolvedViews.omitted
    is already exactly this and can go).
  - report() copies it onto Report. NOT serialised -- keeping it out of the envelope means
    no schema version bump.
  - render() emits note 1 for Format::Text.
  - cli.rs display_notes keeps only note 2.
  - Regenerate the goldens for the ordering change; no backward compatibility is owed.

Not started: it touches Query construction in several places, and it deserves its own gate
cycle rather than being rushed in behind unrelated work.
