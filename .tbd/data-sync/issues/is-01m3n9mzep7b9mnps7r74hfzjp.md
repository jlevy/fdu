---
type: is
id: is-01m3n9mzep7b9mnps7r74hfzjp
title: "Code analysis: next language tier (Assembly, Perl, Make, then Lua, PowerShell, Scala; Kconfig and devicetree recognised)"
kind: feature
status: open
priority: 3
version: 1
labels: []
dependencies: []
created_at: 2026-09-29T00:39:48.438Z
updated_at: 2026-09-29T00:39:48.438Z
---
fdu counts 15 languages against 333-402 in tokei/scc/cloc (SLOC survey, fdu-61ez). The common gaps on real trees such as linux-v6.12 are Assembly, Perl and Make; Kconfig and .dts/.dtsi are not recognised at all. Each language needs analyzer rules plus chunk-split fixtures like the v3 set.
