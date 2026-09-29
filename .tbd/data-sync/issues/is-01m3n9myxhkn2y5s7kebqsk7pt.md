---
type: is
id: is-01m3n9myxhkn2y5s7kebqsk7pt
title: C header probe labels C headers as C++ on substrings like 'pid_namespace ' and comment text
kind: bug
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-09-29T00:39:47.888Z
updated_at: 2026-09-29T00:39:47.888Z
---
crates/fdu-core/src/classify/file_type_detection.rs resolve_c_header returns cpp when the first 16 KiB contains any of 'namespace ', 'template<', 'template <', 'std::', 'constexpr ' as a plain substring. Kernel identifiers such as 'struct pid_namespace *' and comment text match. On linux-v6.12 the same predicate matches 168 of 25,308 .h files (0.66%). Present in 0.2.0; not a regression. Fix: require a token boundary before the keyword (start of line after optional whitespace for 'namespace'/'template'), skip // and /* */ comments, and add fixtures for kernel-style headers and real C++ headers. Changes the language breakdown only, not line counts. Found by the SLOC tool survey (fdu-61ez).
