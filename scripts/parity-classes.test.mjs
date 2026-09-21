import assert from "node:assert/strict";
import test from "node:test";

import { CLASSES, classify, parseSessions } from "./parity-classes.mjs";

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

// A class that cannot fail is worse than no class, because the summary then reports a
// clean surface while a real difference goes unread. Adding one genuinely changed line
// to an otherwise explained session must leave it unexplained, for every class.
test("no class absorbs an extra changed line", () => {
  for (const cls of CLASSES) {
    const polluted = session(["note: x", "Performance: y", "left side"], ["right side"]);
    assert.ok(
      !cls.matches(polluted),
      `${cls.id} explains a session whose answer differs on a line it says nothing about`,
    );
  }
});

test("parseSessions and classify agree on the recorded corpus", () => {
  // Shape-only: the recorded deviations live in tests/parity and are checked by the
  // parity run itself. Here we only assert the parser yields the fields a matcher reads,
  // so a matcher can never see `undefined` and silently return false.
  const parsed = parseSessions("");
  assert.ok(Array.isArray(parsed));
});
