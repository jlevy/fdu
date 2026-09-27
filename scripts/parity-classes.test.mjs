import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { CLASSES, classify, parseSessions } from "./parity-classes.mjs";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const CORPUS = path.join(ROOT, "tests", "parity", "deviations-python.diff");

//: One session shaped the way `parseSessions` produces them.
function session(removed, added, name = "Some Session") {
  return { name, removed, added };
}

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
