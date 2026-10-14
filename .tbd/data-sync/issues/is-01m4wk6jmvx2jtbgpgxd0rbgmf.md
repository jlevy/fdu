---
type: is
id: is-01m4wk6jmvx2jtbgpgxd0rbgmf
title: Expose Python report rendering without eager model conversion
kind: feature
status: open
priority: 2
version: 1
labels: []
dependencies: []
created_at: 2026-10-14T06:57:05.176Z
updated_at: 2026-10-14T06:57:05.176Z
---
Astra review A1: public reports eagerly render JSON, decode dictionaries, build typed models, retain _wire plus native renderer; render-only users pay all stages. Design supported immutable native handle/render-only API with optional/lazy models and wire copy, preserving shared schema and snapshot parity. Move opened owned Report instead of cloning it. Keep complete Files semantics; measure memory and teardown before choosing representation. See Python/Rust architecture review dated 2026-10-13.
