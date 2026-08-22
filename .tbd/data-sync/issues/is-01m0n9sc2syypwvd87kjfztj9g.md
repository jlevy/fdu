---
type: is
id: is-01m0n9sc2syypwvd87kjfztj9g
title: Diagnostics name the Python parameter where the CLI names the flag
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0n9pjsahc4wk6ek37syjct4
created_at: 2026-08-22T17:55:06.968Z
updated_at: 2026-08-22T18:26:54.646Z
---
Eight parity sessions differ only in how the offending option is named:

  CLI:    fdu: invalid --analyze "none,code": "none" names the whole axis and cannot be combined
  Python: fdu: invalid analyze "none,code": "none" names the whole axis and cannot be combined

The rule text is shared -- it lives once in content_model.rs and reaches both surfaces intact, which is why it has not drifted. Only the label differs, because the library formats the message with a caller-supplied name and the binding passes the bare axis name.

Arguably correct on both sides: there is no --analyze flag in Python. But it means the shim can never reach parity on any diagnostic, and it hides real drift in the noise -- fdu-gw5b was found only because its wording differed beyond the prefix.

Decide deliberately: either the library takes the label from the caller (and the binding passes what the caller used), or diagnostics name the axis and the CLI adds its own prefix. Both surfaces should then agree on everything after the label.

## Notes

Resolved as a legitimate deviation rather than a defect, and recorded as such in the deviation artifact's header.

The rule text is shared and identical -- the harness proves it matches word for word, including the bound grammar once the shim stopped inventing its own wording. Only the label differs, because the Python API has no --depth or --analyze to name. A Python diagnostic saying '--analyze' would be wrong.

So this is the same class as --version naming the surface: a difference that is correct on both sides. Eight sessions, documented in tests/parity/deviations-python.diff so a reviewer knows why they are allowed rather than guessing.
