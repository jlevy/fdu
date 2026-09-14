#!/usr/bin/env node
// Capture a bounded slice of `fdu --watch` output so the change stream can be goldened.
//
// A golden compares one command's completed output, and a watch process never exits, so
// there is nothing to compare until watching is expressed as a command that terminates.
// This is that command: it starts a watcher, applies a scripted sequence of changes,
// collects the `fdu.stream/1` records each one produces, and stops.
//
// Determinism comes from the sequencing, not from sleeps. Each step waits for its own
// record to arrive before the next step runs, so the output order is fixed by causality
// rather than by how fast the machine or the backend happens to be. A step that never
// arrives fails loudly instead of silently shortening the stream, because a golden that
// records "nothing happened" would pass forever once watching broke.
//
// Usage: watch-capture [--min-size] <tree>
//
// `--min-size` watches under the selection `--min-size 100 --size apparent` instead, and
// steps through a change the selection excludes, one it admits, and the removal of the
// excluded file. Selection filters what the stream reports, not what is watched, so the
// scripted steps are written against that one bound rather than accepting arbitrary flags.
// The bound is on apparent bytes because the default metric is allocated bytes, which
// gives a four-byte file a whole filesystem block and differs between filesystems.

import { spawn } from "node:child_process";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";

// Generous: covers a cold scan, a backend round trip, and a loaded CI runner. Only
// reached when something is actually broken.
const STEP_TIMEOUT_MS = 30_000;

const argv = process.argv.slice(2);
const sized = argv[0] === "--min-size";
const tree = sized ? argv[1] : argv[0];
if (!tree || argv.length !== (sized ? 2 : 1)) {
  console.error("usage: watch-capture [--min-size] <tree>");
  process.exit(2);
}

// Every scripted path stays at the top level of the tree: a nested path would render with
// the platform's separator and split this golden into two platform-specific expectations.
const streamSteps = [
  { label: "create a file", path: "added.txt", op: "upsert", act: (p) => writeFileSync(p, "hello") },
  {
    label: "change its size",
    path: "added.txt",
    op: "upsert",
    act: (p) => writeFileSync(p, "hello, again"),
  },
  { label: "remove it", path: "added.txt", op: "remove", act: (p) => rmSync(p) },
  { label: "create a directory", path: "sub", op: "upsert", act: (p) => mkdirSync(p) },
];

// A step without an `op` is one the selection excludes, so there is no record to wait for.
// What the stream said about its path up to the next awaited record is printed under its
// label instead: a record that leaks through the selection becomes a golden diff rather
// than something the helper quietly skipped. The next awaited step bounds it, because
// the excluded change happened first, and the watcher applies changes in the order they
// happened and, within one batch, in path order -- which is why the excluded name sorts
// first.
const sizedSteps = [
  { label: "create a file under the bound", path: "a-small.txt", act: (p) => writeFileSync(p, "tiny") },
  {
    label: "create a file over the bound",
    path: "b-large.txt",
    op: "upsert",
    act: (p) => writeFileSync(p, "x".repeat(200)),
  },
  // A removal carries no size to filter on, and hiding it would hide the disappearance
  // of something the caller was watching.
  { label: "remove the file under the bound", path: "a-small.txt", op: "remove", act: (p) => rmSync(p) },
];

const steps = sized ? sizedSteps : streamSteps;
const selection = sized ? ["--min-size", "100", "--size", "apparent"] : [];

// FDU names the executable outright, extension included, so there is no PATH lookup to
// fall through to a different build and no PATHEXT branch for Windows to get wrong.
const binary = process.env.FDU;
if (!binary) {
  throw new Error("FDU must name the fdu executable under test");
}
const child = spawn(binary, ["--watch", "--view", "files", "--format", "jsonl", ...selection, tree], {
  stdio: ["ignore", "pipe", "pipe"],
});

let pending = "";
const lines = [];
// Records at an index below the cursor have already satisfied an earlier step. Without
// this, a step's matcher can be satisfied by a record from a *previous* step -- the first
// version of this script reported the create record three times for three different
// actions, which is exactly the kind of silent wrongness a golden must not encode.
let cursor = 0;
const waiters = [];
let failure = null;

/// Fail every outstanding wait with the same reason.
///
/// Without this, a child that dies leaves the awaiting promise unsettled forever. Node
/// then exits 13 reporting an "unsettled top-level await" and says nothing about the
/// cause -- which is exactly how this script first failed on Windows CI, hiding whatever
/// the child had actually done.
function fail(reason) {
  failure ??= reason;
  while (waiters.length > 0) waiters.shift().reject(new Error(reason));
}

child.stdout.setEncoding("utf8");
child.stdout.on("data", (chunk) => {
  pending += chunk;
  const parts = pending.split("\n");
  pending = parts.pop() ?? "";
  for (const line of parts) {
    if (!line.trim()) continue;
    lines.push(line);
    // Only the oldest waiter may match: steps are strictly sequential, and a record can
    // satisfy at most the step that caused it.
    if (waiters.length > 0 && waiters[0].matches(line)) {
      cursor = lines.length;
      const waiter = waiters.shift();
      waiter.done?.();
      waiter.resolve(line);
    }
  }
});

let stderr = "";
child.stderr.setEncoding("utf8");
child.stderr.on("data", (chunk) => {
  stderr += chunk;
});
child.on("error", (error) => {
  fail(`could not start ${binary}: ${error.message}`);
});
child.on("exit", (code, signal) => {
  // Any exit is a failure while steps remain: the watcher is supposed to outlive them.
  if (waiters.length > 0 || !done) {
    fail(`${binary} exited early with status ${code} (signal ${signal})`);
  }
});

/** Wait for a line matching `matches` that arrived after the previous step, or fail. */
function waitFor(matches, description) {
  for (let i = cursor; i < lines.length; i += 1) {
    if (matches(lines[i])) {
      cursor = i + 1;
      return Promise.resolve(lines[i]);
    }
  }
  return new Promise((resolve, reject) => {
    const waiter = { matches, resolve, reject };
    waiters.push(waiter);
    // Deliberately not unref'd: this timer is what keeps the event loop alive while
    // waiting, so a stalled step reports a timeout instead of letting the process fall
    // out from under an unsettled await.
    const timer = setTimeout(() => {
      const index = waiters.indexOf(waiter);
      if (index !== -1) {
        waiters.splice(index, 1);
        reject(new Error(`timed out waiting for ${description}`));
      }
    }, STEP_TIMEOUT_MS);
    waiter.done = () => clearTimeout(timer);
  });
}

const isChange = (line) => line.includes('"record": "change"');

// Set once every step has been captured, so the exit handler can tell a normal shutdown
// from a child that died mid-capture.
let done = false;

try {
  // The initial report is a one-shot answer, identical to a run without --watch. Waiting
  // for it is what proves the watcher is bound before any change is made; skipping it
  // would race the scan and lose the first event.
  await waitFor((line) => line.includes('"view": "files"'), "the initial report");

  const captured = [];
  // Excluded steps acted on since the last awaited record, each with where it began.
  let excluded = [];
  for (const step of steps) {
    if (failure) break;
    const start = lines.length;
    step.act(join(tree, step.path));
    if (!step.op) {
      const entry = { label: step.label, path: step.path, start, records: [] };
      captured.push(entry);
      excluded.push(entry);
      continue;
    }
    const record = await waitFor(
      (line) =>
        isChange(line) &&
        line.includes(`"path": "${step.path}"`) &&
        line.includes(`"op": "${step.op}"`),
      `${step.label} (${step.op} ${step.path})`,
    );
    for (const entry of excluded) {
      entry.records = lines
        .slice(entry.start, cursor)
        .filter((line) => isChange(line) && line.includes(`"path": "${entry.path}"`));
    }
    excluded = [];
    captured.push({ label: step.label, records: [record] });
  }

  if (failure) throw new Error(failure);
  if (excluded.length > 0) {
    throw new Error("an excluded step needs an awaited step after it to bound what it captured");
  }
  done = true;

  for (const { label, records } of captured) {
    console.log(`# ${label}`);
    for (const record of records) console.log(record);
  }
} catch (error) {
  console.error(error.message);
  if (stderr.trim()) console.error(stderr.trim());
  child.kill("SIGKILL");
  process.exit(1);
}

child.kill("SIGKILL");
