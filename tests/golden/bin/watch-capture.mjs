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
// Usage: watch-capture <tree>

import { spawn } from "node:child_process";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";

// Generous: covers a cold scan, a backend round trip, and a loaded CI runner. Only
// reached when something is actually broken.
const STEP_TIMEOUT_MS = 30_000;

const tree = process.argv[2];
if (!tree) {
  console.error("usage: watch-capture <tree>");
  process.exit(2);
}

// Every scripted path stays at the top level of the tree: a nested path would render with
// the platform's separator and split this golden into two platform-specific expectations.
const steps = [
  { label: "create a file", path: "added.txt", act: (p) => writeFileSync(p, "hello") },
  { label: "change its size", path: "added.txt", act: (p) => writeFileSync(p, "hello, again") },
  { label: "remove it", path: "added.txt", act: (p) => rmSync(p) },
  { label: "create a directory", path: "sub", act: (p) => mkdirSync(p) },
];

const child = spawn("fdu", ["--watch", "--view", "files", "--format", "jsonl", tree], {
  stdio: ["ignore", "pipe", "pipe"],
});

let pending = "";
const lines = [];
const waiters = [];
let failure = null;

child.stdout.setEncoding("utf8");
child.stdout.on("data", (chunk) => {
  pending += chunk;
  const parts = pending.split("\n");
  pending = parts.pop() ?? "";
  for (const line of parts) {
    if (!line.trim()) continue;
    lines.push(line);
    // Resolve in order, so a waiter cannot be satisfied by a record that arrived before
    // its own step ran.
    for (let i = 0; i < waiters.length; i += 1) {
      if (waiters[i].matches(line)) {
        waiters.splice(i, 1)[0].resolve(line);
        break;
      }
    }
  }
});

let stderr = "";
child.stderr.setEncoding("utf8");
child.stderr.on("data", (chunk) => {
  stderr += chunk;
});
child.on("error", (error) => {
  failure = `could not start fdu: ${error.message}`;
});
child.on("exit", (code, signal) => {
  if (failure === null && signal === null && code !== 0) {
    failure = `fdu exited early with status ${code}: ${stderr.trim()}`;
  }
});

/** Wait for a line matching `matches`, or fail with `description`. */
function waitFor(matches, description) {
  const existing = lines.find(matches);
  if (existing) return Promise.resolve(existing);
  return new Promise((resolve, reject) => {
    const waiter = { matches, resolve };
    waiters.push(waiter);
    setTimeout(() => {
      const index = waiters.indexOf(waiter);
      if (index !== -1) {
        waiters.splice(index, 1);
        reject(new Error(`timed out waiting for ${description}`));
      }
    }, STEP_TIMEOUT_MS).unref?.();
  });
}

const isChange = (line) => line.includes('"record": "change"');

try {
  // The initial report is a one-shot answer, identical to a run without --watch. Waiting
  // for it is what proves the watcher is bound before any change is made; skipping it
  // would race the scan and lose the first event.
  await waitFor((line) => line.includes('"view": "files"'), "the initial report");

  const captured = [];
  for (const step of steps) {
    if (failure) break;
    step.act(join(tree, step.path));
    const record = await waitFor(
      (line) => isChange(line) && line.includes(`"path": "${step.path}"`),
      `${step.label} (${step.path})`,
    );
    captured.push({ label: step.label, record });
  }

  if (failure) throw new Error(failure);

  for (const { label, record } of captured) {
    console.log(`# ${label}`);
    console.log(record);
  }
} catch (error) {
  console.error(error.message);
  if (stderr.trim()) console.error(stderr.trim());
  child.kill("SIGKILL");
  process.exit(1);
}

child.kill("SIGKILL");
