import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { CLASSES, classify, normalisePortableValues, parseSessions } from "./parity-classes.mjs";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const CORPUS = path.join(ROOT, "tests", "parity", "deviations-python.diff");

//: One session shaped the way `parseSessions` produces them.
function session(removed, added, name = "Some Session", file = "") {
  return { name, file, removed, added };
}

test("portable golden paths require the exact fixture root and unchanged other fields", () => {
  const cache = "[ROOT]/tests/parity/.corpus/cli-cache.tryscript.md";
  const content = "[ROOT]/tests/parity/.corpus/cli-content.tryscript.md";
  const json = (root, files) =>
    `{"schema": "fdu.report/10", "root": "${root}", "files": ${files}}`;
  assert.equal(
    classify(session([json("[SCAN_PATH]", 7)], [json("[SANDBOX]/project", 7)], "Cache", cache))?.id,
    "portable-golden-pattern",
  );
  assert.equal(
    classify(session([json("[SCAN_PATH]", 7)], [json("[SANDBOX]/content-project", 7)], "Content", content))?.id,
    "portable-golden-pattern",
  );
  const envelope = (root, age, observed, allocated, files = 7, schema = "fdu.report/10") =>
    `{"schema": "${schema}", "root": "${root}", "age_reference_ns": ${age}, "observed_at_ns": ${observed}, "allocated": ${allocated}, "files": ${files}}`;
  const golden = envelope("[SCAN_PATH]", "[AGE_NS]", "[MTIME_NS]", "[ALLOCATED]");
  const concrete = (age, allocated, files = 7, schema = "fdu.report/10", observed = "456") =>
    envelope("[SANDBOX]/project", age, observed, allocated, files, schema);
  assert.equal(
    classify(session([golden], [concrete("-123", "4096")], "JSONL", cache))?.id,
    "portable-golden-pattern",
    "whole JSONL lines retain the golden's typed numeric patterns",
  );
  for (const actual of [
    concrete("NaN", "4096"),
    concrete("-123", "unknown"),
    concrete("-123", "4096", 7, "fdu.report/10", "unknown"),
    concrete("-123", "4096", 8),
    concrete("-123", "4096", 7, "fdu.report/8"),
  ]) {
    assert.equal(classify(session([golden], [actual], "JSONL", cache)), null, actual);
  }
  assert.equal(
    classify(session(["80 B  assets[SEP]logo.png"], ["80 B  assets/logo.png"]))?.id,
    "portable-golden-pattern",
  );
  assert.equal(
    classify(session(['{"path": "dist[JSON_SEP]a.tar.gz"}'], ['{"path": "dist/a.tar.gz"}']))?.id,
    "portable-golden-pattern",
    "the JSON-string separator pattern is the same separator",
  );
  assert.equal(
    classify(session(['{"path": "dist[JSON_SEP]a.tar.gz"}'], ['{"path": "dist/b.tar.gz"}'])),
    null,
    "the rest of a JSON path must still match exactly",
  );
  for (const [actual, fixture] of [
    ["[SANDBOX]/other", cache],
    ["[SANDBOX]/content-project", cache],
    ["[SANDBOX]/project/child", cache],
    ["/unrelated/project", cache],
  ]) {
    assert.equal(classify(session([json("[SCAN_PATH]", 7)], [json(actual, 7)], "Cache", fixture)), null);
  }
  assert.equal(
    classify(session([json("[SCAN_PATH]", 7)], [json("[SANDBOX]/project", 8)], "Cache", cache)),
    null,
    "a changed file count cannot ride along with a portable root",
  );
  assert.equal(
    classify(session(['"other": "[SCAN_PATH]"'], ['"other": "[SANDBOX]/project"'], "Cache", cache)),
    null,
    "only the report root field has this portable pattern",
  );
  assert.equal(
    classify(session(["otherroot: [SCAN_PATH]"], ["otherroot: [SANDBOX]/project"], "Cache", cache)),
    null,
    "a YAML field whose name only ends in root must not match",
  );
});

test("portable numeric masking touches observed values only and rejects literal drift", () => {
  const observed = normalisePortableValues(
    '-"allocated": 123\n-"age_reference_ns": [AGE_NS]\n-"observed_at_ns": [MTIME_NS]\n-"allocated": [ALLOCATED]\n+"allocated": 999\n+"age_reference_ns": -456\n+"observed_at_ns": 789\n+"allocated": NaN\n',
  );
  assert.equal(
    observed,
    '-"allocated": 123\n-"age_reference_ns": [AGE_NS]\n-"observed_at_ns": [MTIME_NS]\n-"allocated": [ALLOCATED]\n+"allocated": 999\n+"age_reference_ns": [AGE_NS_VALUE]\n+"observed_at_ns": [MTIME_NS_VALUE]\n+"allocated": NaN\n',
  );
  const golden = '-"age_reference_ns": [AGE_NS], "observed_at_ns": [MTIME_NS], "allocated": [ALLOCATED]\n';
  assert.equal(
    normalisePortableValues(golden + '+"age_reference_ns": 1, "observed_at_ns": 2, "allocated": 4096\n'),
    normalisePortableValues(golden + '+"age_reference_ns": 9, "observed_at_ns": 8, "allocated": 8192\n'),
    'two observations of the same typed fields serialize to one stable artifact line',
  );
  assert.equal(
    classify(session(['"allocated": 123'], ['"allocated": [ALLOCATED_VALUE]'])),
    null,
    'a literal golden allocated value is never a portable wildcard',
  );
  assert.equal(
    classify(session(['"allocated": [ALLOCATED]'], ['"allocated": NaN'])),
    null,
    'malformed numeric output remains visible',
  );
});

test("every class carries the fields the report prints", () => {
  for (const cls of CLASSES) {
    assert.ok(cls.id, "a class needs an id");
    assert.ok(cls.title, `${cls.id} needs a title`);
    assert.ok(Array.isArray(cls.why) && cls.why.length > 0, `${cls.id} needs a reason`);
    assert.equal(typeof cls.matches, "function", `${cls.id} needs a matcher`);
  }
});

test("class ids are unique", () => {
  const ids = CLASSES.map((cls) => cls.id);
  assert.deepEqual(ids, [...new Set(ids)]);
});

test("facts and suggestions cannot be dismissed as run telemetry", () => {
  for (const line of ["note: an observed fact", "tip: --view families", "warn: a partial result"]) {
    assert.equal(classify(session([line], [])), null, line);
  }
});

test("bound tips accept only exact CLI-to-Python setter names and values", () => {
  const matches = (removed, added) =>
    classify(session([removed, "note: same fact"], [added, "note: same fact"]))?.id ===
    "bound-tip-vocabulary";
  for (const [cli, api] of [
    ["tip: show smaller entries: --min-share=0%", "tip: show smaller entries: min_share=0%"],
    ["tip: expand deeper: --depth=all", "tip: expand deeper: depth=all"],
    ["tip: show more children: --breadth=all", "tip: show more children: breadth=all"],
    ["tip: show more rows: --limit=all", "tip: show more rows: limit=all"],
  ]) {
    assert.ok(matches(cli, api), `${cli} / ${api}`);
    assert.ok(matches(`! ${cli}`, `! ${api}`), `stderr: ${cli}`);
    assert.ok(!matches(cli, `${api} now`), "extra text is a real difference");
    assert.ok(!matches(cli, api.replace("=all", "=3").replace("=0%", "=1%")), "value changed");
  }
  assert.ok(!matches("tip: show smaller entries: --min-share=0%", "tip: expand deeper: depth=all"));
});

test("the stale-answer warning accepts only its exact option translation", () => {
  const cache = "[ROOT]/tests/parity/.corpus/cli-cache.tryscript.md";
  const warning = (option) =>
    `warn: stale answer: served from the snapshot without filesystem verification; drop ${option} for a fresh answer`;
  const root = (value) => `{"root": "${value}"}`;
  const classified = (cli, api) =>
    classify(
      session([root("[SCAN_PATH]"), cli], [root("[SANDBOX]/project"), api], "Stale", cache),
    )?.id;
  assert.equal(classified(warning("--stale-ok"), warning("stale_ok")), "portable-golden-pattern");
  assert.equal(
    classified(`! ${warning("--stale-ok")}`, `! ${warning("stale_ok")}`),
    "portable-golden-pattern",
    "stderr marker",
  );
  for (const api of [
    warning("--stale-ok").replace("stale answer", "cached answer"),
    warning("refresh()"),
    `${warning("stale_ok")} now`,
    warning("stale_ok").replace("warn:", "note:"),
  ]) {
    assert.equal(classified(warning("--stale-ok"), api), undefined, api);
  }
  assert.equal(
    classify(session([warning("--stale-ok")], [warning("stale_ok")]))?.id,
    "surface-label",
    "alone, the option label is the whole difference",
  );
});

// A class that cannot fail is worse than no class: the summary then reports a clean
// surface while a real difference goes unread. Each class gets a fixture it would
// otherwise match, polluted with one genuinely changed line.
//
// This must be checked per class with an appropriate fixture. An earlier version used a
// single fixture for all four, which `surface-label` and `surface-vocabulary` rejected on
// length alone and `discovery-surface` rejected on its name — so three of the four passed
// for a reason unrelated to the property, which is the defect this file exists to catch.
test("no class absorbs an extra changed line", () => {
  const polluted = {
    "portable-golden-pattern": session(
      ['"root": "[SCAN_PATH]"', "total 100"],
      ['"root": "[SANDBOX]/project"', "total 999"],
      "Cache",
      "[ROOT]/tests/parity/.corpus/cli-cache.tryscript.md",
    ),
    // Same shape these two explain (equal-length hunks, one knob renamed), plus one line
    // whose two sides genuinely disagree.
    "surface-label": session(
      ["error: invalid --scan-depth", "total 100"],
      ["error: invalid max_depth", "total 999"],
    ),
    "surface-vocabulary": session(
      ["error: invalid --modified-since", "total 100"],
      ["error: invalid modified_since", "total 999"],
    ),
    "bound-tip-vocabulary": session(
      ["tip: expand deeper: --depth=all", "total 100"],
      ["tip: expand deeper: depth=all", "total 999"],
    ),
  };
  for (const cls of CLASSES) {
    const fixture = polluted[cls.id];
    if (!fixture) continue;
    assert.ok(
      !cls.matches(fixture),
      `${cls.id} explains a session whose answer differs on a line it says nothing about`,
    );
  }

  // `discovery-surface` is deliberately excluded and pinned here rather than left to look
  // covered. It matches on the session NAME alone, so it does absorb a changed line by
  // design — the package genuinely carries no `--docs`, and the whole session is the
  // difference. Recorded so that a future change making it content-sensitive is a visible
  // decision rather than a silent one.
  const byName = CLASSES.find((cls) => cls.id === "discovery-surface");
  assert.ok(
    byName.matches({ name: "Version Is Exact", removed: ["fdu 0.1.0"], added: ["fdu 9.9.9 (python)"] }),
    "discovery-surface is name-keyed; if this now fails, it became content-sensitive",
  );
});

// The recorded corpus is the thing these matchers exist for, so a change that orphans a
// recorded session must fail here rather than at the next `make parity-check`.
test("every recorded deviation still classifies", () => {
  const sessions = parseSessions(readFileSync(CORPUS, "utf8"));
  assert.ok(sessions.length > 0, "the recorded corpus parsed to nothing");

  const unexplained = sessions.filter((item) => classify(item) === null);
  assert.deepEqual(
    unexplained.map((item) => item.name),
    [],
    "recorded sessions no longer explained by any class",
  );

  // Every class must still earn its place: one that explains nothing would quietly
  // absorb a real regression later.
  for (const cls of CLASSES) {
    const count = sessions.filter((item) => classify(item)?.id === cls.id).length;
    assert.ok(count > 0, `${cls.id} explains no recorded session and should be removed`);
  }
});
