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

// This gate is what backs the claim that every surface gives the same answer, so the
// interesting question about a class is never "does it match what it was written for"
// but "what else does it now excuse". `run-telemetry` matched on the *count* of the
// non-telemetry remainder, so one removed line was explained by one added line whatever
// the two said: a session that lost a `note:` line could also report a different tally
// and still classify. The remainder has to be equal, not merely equinumerous.
test("run-telemetry explains only the telemetry lines themselves", () => {
  const matches = (item) => classify(item)?.id === "run-telemetry";

  // Genuine: the command line prints telemetry the report envelope cannot carry.
  assert.ok(matches(session(["note: analysis read 5 bytes"], [])));
  assert.ok(matches(session(["Performance: 3ms"], [])));
  // Genuine: telemetry removed, everything else identical.
  assert.ok(matches(session(["note: x", "256 B  7 files"], ["256 B  7 files"])));

  // A changed tally riding along with a removed note is a different answer, not telemetry.
  assert.ok(
    !matches(session(["note: x", "256 B  7 files, 4 directories"], ["512 B  9 files, 4 directories"])),
  );
  assert.ok(!matches(session(["Performance: x", "total 100"], ["total 999"])));
  // Order matters too: a remainder that matches as a set but not in sequence is a diff.
  assert.ok(!matches(session(["note: x", "a", "b"], ["b", "a"])));

  // Without a telemetry line there is nothing for this class to explain.
  assert.ok(!matches(session(["total 100"], ["total 999"])));
});

// A golden writes the separator as the named pattern and the package prints the literal,
// so comparing raw text would make any hunk containing a path read as a changed answer.
// That is why `sameSeparator` exists; leaving it out of this class made a legitimate
// telemetry-only session unexplained, and the predictable response to that would have
// been to loosen the equality rule again.
test("run-telemetry compares paths through the separator pattern", () => {
  const matches = (item) => classify(item)?.id === "run-telemetry";
  assert.ok(matches(session(["80 B  assets[SEP]logo.png", "note: x"], ["80 B  assets/logo.png"])));
  // The separator is the only thing it may fold: a different file is still a diff.
  assert.ok(!matches(session(["80 B  assets[SEP]logo.png", "note: x"], ["80 B  assets/other.png"])));
  assert.ok(!matches(session(["80 B  assets[SEP]logo.png", "note: x"], ["99 B  assets/logo.png"])));
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
    "run-telemetry": session(["note: x", "total 100"], ["total 999"]),
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
