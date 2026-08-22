---
type: is
id: is-01m0n9q3by1dk5kcqdzyb7n4cw
title: Python filesystem errors do not carry the CLI's message shape
kind: bug
status: closed
priority: 2
version: 2
labels: []
dependencies: []
parent_id: is-01m0n9pjsahc4wk6ek37syjct4
created_at: 2026-08-22T17:53:52.509Z
updated_at: 2026-08-22T18:10:34.127Z
closed_at: 2026-08-22T18:10:34.127Z
close_reason: |-
  FilesystemError now renders like the CLI -- 'I/O error at {path}: {reason}' -- and exposes .path. OSError's own __str__ led with '[Errno None]' for anything raised from the engine, stating an error number that does not exist.

  Two further duplications removed while confirming it. The engine's strerror already carries the operating system's detail, so an errno suffix produced 'No such file or directory (os error 2) (errno 2)'. And the CLI printed its cause chain unconditionally, so 'I/O error at {path}: {source}' -- which embeds its source -- was followed by 'caused by: {source}', the same sentence twice, on the most common failure there is. The chain now skips a cause the headline already contains.

  Both surfaces produce byte-identical output for a missing root and for a non-directory root.
---
Two parity sessions differ in how a filesystem failure reads:

  CLI:    fdu: I/O error at missing: No such file or directory (os error 2)
            caused by: No such file or directory (os error 2)
  Python: fdu: [Errno None] scan root is not a directory: '/.../plain-file'

The CLI names the operation and the path and then the cause; the Python exception leads with an Errno that is None. A caller printing str(error) gets something less useful than the CLI for the same failure, and 'Errno None' is actively misleading.

FilesystemError should expose path, kind, message and os_error as fields (it already does) AND render like the CLI in __str__.
