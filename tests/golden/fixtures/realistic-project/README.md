# Acorn

Acorn inventories a source tree, keeps a compact local index, and renders reports for
people and scripts. The fixture resembles a small Rust command-line project so the
default report has enough hierarchy to expose alignment, ranking, depth, and bar-width
regressions in one readable golden.

Run `acorn scan PATH` to build an index and `acorn report PATH` to inspect it.
