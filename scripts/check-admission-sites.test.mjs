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

test("does not accept a route named only in a comment or a string", () => {
  for (const mention of [
    "  // prepare_walk_entry();",
    "  /// prepare_walk_entry(",
    "  /* prepare_walk_entry() */",
    '  let route = "prepare_walk_entry(";',
  ]) {
    const opened = ["read_dir(root);", "for item in listing {", mention, "  retain(item);", "}"].join(
      "\n",
    );
    const result = auditAdmissionSources(
      baseline(new Map([["crates/fdu-core/src/opened.rs", opened]])),
    );
    assert.match(result.problems[0] ?? "", /bypasses the admission chokepoint/, mention);
  }
});

test("does not accept an emission route named only in a comment", () => {
  const scan = baseline().get("crates/fdu-core/src/scan.rs").replace(
    "fn record_entry() { record_detached_entry(); }",
    "fn record_entry() { retain(entry); } // record_detached_entry();",
  );
  const result = auditAdmissionSources(
    baseline(new Map([["crates/fdu-core/src/scan.rs", scan]])),
  );
  assert.match(result.problems[0] ?? "", /DetachedEmission bypasses the admission chokepoint/);
});

test("rejects an emission implementation the audit does not name", () => {
  const scan = [
    baseline().get("crates/fdu-core/src/scan.rs"),
    "impl WalkEmission for ShadowEmission {",
    "  fn record_entry() { record_walk_entry(); }",
    "}",
  ].join("\n");
  const result = auditAdmissionSources(
    baseline(new Map([["crates/fdu-core/src/scan.rs", scan]])),
  );
  assert.match(result.problems[0] ?? "", /unaudited emission implementation ShadowEmission/);
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

test("audits an emission implementation that follows a quote character literal", () => {
  // A `'"'` used to open string state and blank every later line up to the next double
  // quote, so an implementation in that span was neither audited nor reported.
  const scan = [
    baseline().get("crates/fdu-core/src/scan.rs"),
    "fn is_quote(c: char) -> bool { c == '\"' }",
    "impl WalkEmission for ShadowEmission {",
    "  fn record_entry() { record_walk_entry(); }",
    "}",
  ].join("\n");
  const result = auditAdmissionSources(
    baseline(new Map([["crates/fdu-core/src/scan.rs", scan]])),
  );
  assert.match(result.problems[0] ?? "", /unaudited emission implementation ShadowEmission/);
});

test("ignores quotes and braces in character literals while checking a loop", () => {
  for (const literal of [
    "'\"'",
    "'}'",
    "'{'",
    "b'}'",
    "'\\''",
    "'\\\"'",
    "'\\\\'",
    "'\\x7d'",
    "'\\u{7d}'",
    "'\\u{10FFFF}'",
    "'\u{1F600}'",
  ]) {
    const opened = [
      "read_dir(root);",
      "for item in listing {",
      `  let literal = ${literal};`,
      "  prepare_walk_entry();",
      "}",
    ].join("\n");
    const result = auditAdmissionSources(
      baseline(new Map([["crates/fdu-core/src/opened.rs", opened]])),
    );
    assert.deepEqual(result.problems, [], literal);
  }
});

test("keeps lifetimes and loop labels as code rather than character literals", () => {
  // A lifetime never closes with a quote. Reading `'a>(item: &'` as a literal would blank
  // real code, so these must leave the route visible and the braces counted.
  const opened = [
    "read_dir(root);",
    "for item in listing {",
    "  fn keep<'a>(item: &'a Item) -> &'a Item { item }",
    "  fn name(_: &'static str, _: &'_ str) {}",
    "  'next: loop { break 'next; }",
    "  prepare_walk_entry();",
    "}",
  ].join("\n");
  const result = auditAdmissionSources(
    baseline(new Map([["crates/fdu-core/src/opened.rs", opened]])),
  );
  assert.deepEqual(result.problems, []);

  const scan = [
    baseline().get("crates/fdu-core/src/scan.rs"),
    "impl<'a> WalkEmission for ShadowEmission<'a> {",
    "  fn record_entry(&'a self) { record_walk_entry(); }",
    "}",
  ].join("\n");
  const shadow = auditAdmissionSources(
    baseline(new Map([["crates/fdu-core/src/scan.rs", scan]])),
  );
  assert.match(shadow.problems[0] ?? "", /unaudited emission implementation ShadowEmission/);
});
