# CLI Golden Fixture

The `project/` tree represents a small release-oriented software project.
Its files are valid examples of the formats their names advertise, so the fixture is
understandable when inspected independently of the expected output.

The corpus remains deliberately compact while covering the CLI behaviors that need exact
end-to-end evidence:

- `Makefile` is an extensionless file
- `README.md` and `docs/FAQ.MD` exercise extension aggregation and case folding
- `src/alpha.rs` and `src/omega.rs` are equally sized, exercising the name tie-break
- `dist/acorn-0.1.0.tar.gz` is a valid deterministic archive with a compound extension
- Three directories exercise nested roll-ups, depth limits, and per-directory row limits

The `realistic-project/` tree is the default-output acceptance fixture.
Its sixteen source, documentation, test, benchmark, manifest, and README files create a
concise but recognizable Rust project with three visible directory levels.
One full-output scenario therefore covers the natural overview, ranking, indentation,
alignment, file roll-ups, default depth and limit, compact bars, and the distinction
between a deeper roll-up and an actually omitted ranked sibling.

Fixture bytes are committed rather than generated during a test.
Text files are pinned to LF and the archive is marked binary in `.gitattributes`,
keeping apparent sizes identical across supported platforms.

<!-- This document follows common-doc-guidelines.md.
See github.com/jlevy/practical-prose and review guidelines before editing.
-->
