---
type: is
id: is-01m3n9myxhkn2y5s7kebqsk7pt
title: C header probe labels C headers as C++ on substrings like 'pid_namespace ' and comment text
kind: bug
status: closed
priority: 2
version: 3
labels: []
dependencies: []
parent_id: is-01m3r273jb24qc4hp7ak005jfm
created_at: 2026-09-29T00:39:47.888Z
updated_at: 2026-09-30T03:48:52.886Z
closed_at: 2026-09-30T03:48:52.886Z
close_reason: "Fixed in 2ed3af9f: resolve_c_header scans the 16 KiB prefix once, skipping // and /* */ comments and string/char literals, and counts namespace/template only opening a line and std::/constexpr only as whole tokens. Test c_headers_are_not_cpp_on_identifier_comment_or_string_substrings (kernel lines from the bead) failed before. Measured on linux-v6.12 with --analyze lines --view files: 168 of 25,292 .h files were cpp before, 7 after, all scripts/gcc-plugins/*.h which are C++. Line counts unchanged."
resolution: null
duplicate_of: null
---
crates/fdu-core/src/classify/file_type_detection.rs resolve_c_header returns cpp when the first 16 KiB contains any of 'namespace ', 'template<', 'template <', 'std::', 'constexpr ' as a plain substring. Kernel identifiers such as 'struct pid_namespace *' and comment text match. On linux-v6.12 the same predicate matches 168 of 25,308 .h files (0.66%). Present in 0.2.0; not a regression. Fix: require a token boundary before the keyword (start of line after optional whitespace for 'namespace'/'template'), skip // and /* */ comments, and add fixtures for kernel-style headers and real C++ headers. Changes the language breakdown only, not line counts. Found by the SLOC tool survey (fdu-61ez).
