import assert from "node:assert/strict";
import test from "node:test";

import { auditAdmissionSources } from "./check-admission-sites.mjs";

function baseline(overrides = new Map()) {
  return new Map([
    [
      "crates/fdu-core/src/scan.rs",
      [
        "fn record_detached_entry(",
        "fn record_walk_entry(",
        "let process_entry = |",
        "let mut process_entry =",
        "crate::admission::should_descend(",
        ...Array.from({ length: 5 }, () => "admission::decide("),
        "impl WalkEmission for StreamingEmission {",
        "  fn record_entry() { record_walk_entry(); }",
        "}",
        "impl WalkEmission for DetachedEmission {",
        "  fn record_entry() { record_detached_entry(); }",
        "}",
        ...Array.from({ length: 8 }, () => "for item in listing {\n  process_entry();\n}"),
      ].join("\n"),
    ],
    [
      "crates/fdu-core/src/opened.rs",
      "read_dir(root);\nfor item in listing {\n  prepare_walk_entry();\n}",
    ],
    ["crates/fdu-core/src/watch.rs", "crate::admission::decide_path("],
    ...overrides,
  ]);
}

test("audits every declared inventory producer", () => {
  assert.deepEqual(auditAdmissionSources(baseline()).problems, []);
});

test("rejects a directory reader outside the explicit classification", () => {
  const sources = baseline(
    new Map([["crates/fdu-core/src/new_walker.rs", "let listing = read_dir(root);"]]),
  );
  assert.match(auditAdmissionSources(sources).problems[0], /unclassified directory reader/);
});

test("ignores braces in Rust comments and strings while checking a loop", () => {
  const opened = [
    "read_dir(root);",
    "for item in listing {",
    '  let message = "}";',
    "  // } does not close the loop",
    "  prepare_walk_entry();",
    "}",
  ].join("\n");
  const result = auditAdmissionSources(
    baseline(new Map([["crates/fdu-core/src/opened.rs", opened]])),
  );
  assert.deepEqual(result.problems, []);
});

test("rejects a producer loop that bypasses admission", () => {
  const opened = "read_dir(root);\nfor item in listing {\n  retain(item);\n}";
  const result = auditAdmissionSources(
    baseline(new Map([["crates/fdu-core/src/opened.rs", opened]])),
  );
  assert.match(result.problems[0], /bypasses the admission chokepoint/);
});

test("rejects a generic emission implementation that bypasses admission", () => {
  const scan = baseline().get("crates/fdu-core/src/scan.rs").replace(
    "fn record_entry() { record_detached_entry(); }",
    "fn record_entry() { retain(entry); }",
  );
  const result = auditAdmissionSources(
    baseline(new Map([["crates/fdu-core/src/scan.rs", scan]])),
  );
  assert.match(result.problems[0], /DetachedEmission bypasses the admission chokepoint/);
});
