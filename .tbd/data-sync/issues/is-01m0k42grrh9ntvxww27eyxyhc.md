---
type: is
id: is-01m0k42grrh9ntvxww27eyxyhc
title: The extensions view has no share column while its three sibling views do
kind: task
status: open
priority: 3
version: 1
labels: []
dependencies: []
parent_id: is-01m0k41ks4s0nxzfxj3v141nx8
created_at: 2026-08-21T21:36:46.359Z
updated_at: 2026-08-21T21:36:46.359Z
---
`families`, `types`, and `languages` each print a share column; `extensions` does not, so
one grouped view is shaped unlike its three siblings.

    FAMILIES     1.0 GiB   43.0%  unknown   22543 files
    EXTENSIONS   673 MiB  (none)            6454 files

`TypeRow` carries bytes, allocated, and files but no denominator, and the section is a
bare `Vec<TypeRow>` with no total. Summing the shown rows is wrong once `--limit` truncates
them, so the total has to be plumbed through the section -- which changes the machine
shape and therefore needs a schema bump.

Separable from the alignment fix, and larger than it looks.
