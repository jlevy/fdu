#!/usr/bin/env node
// Capture one text repaint of `fdu --watch` so the boundary between repaints can be
// goldened.
//
// The sibling `watch-capture` helper pins the JSONL change stream, where every record is
// self-delimiting. Text has no such framing: a watch run has no final answer and so no
// performance footer, which once left the last row of one repaint and the first row of
// the next as adjacent lines. What this captures is the separator that fixed that, and
// the fact that it appears between repaints rather than above the first one.
//
// Determinism comes from sequencing, not sleeps, exactly as in the sibling helper: the
// change is made only after the initial report has been seen whole, and capture stops at
// the first complete repaint. A step that never arrives fails loudly, because a golden
// recording "nothing happened" would pass forever once repainting broke.
//
// Usage: watch-repaint-capture <tree>

import { spawn } from "node:child_process";
import { writeFileSync } from "node:fs";
import { join } from "node:path";

// Generous enough for a cold scan, a backend round trip, and a loaded CI runner, and
// still inside the harness's own per-command timeout — so a stall reports which step
// stalled instead of a bare "command timed out" that says nothing.
const STEP_TIMEOUT_MS = 20_000;

// The shortest interval the duration grammar accepts, which keeps the repaint well
// inside the step timeout. It throttles rendering only, so it cannot change what the
// repaint says.
const INTERVAL = "1s";

const tree = process.argv[2];
if (!tree) {
  console.error("usage: watch-repaint-capture <tree>");
  process.exit(2);
}

// FDU names the executable outright, extension included, so there is no PATH lookup to
// fall through to a different build and no PATHEXT branch for Windows to get wrong.
const binary = process.env.FDU;
if (!binary) {
  throw new Error("FDU must name the fdu executable under test");
}
const child = spawn(
  binary,
  [
    "--watch",
    "--interval",
    INTERVAL,
    "--view",
    "tree,summary",
    // Apparent bytes are filesystem-independent, which is what makes the rows stable
    // across the platforms this golden runs on.
    "--size",
    "apparent",
    "--color",
    "never",
    tree,
  ],
  { stdio: ["ignore", "pipe", "pipe"] },
);

let pending = "";
const lines = [];
const waiters = [];
let failure = null;
let done = false;

/// Fail every outstanding wait with the same reason.
///
/// Without this a child that dies leaves the awaiting promise unsettled forever, and node
/// exits reporting an unsettled top-level await while saying nothing about the cause.
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
    lines.push(line);
    if (waiters.length > 0 && waiters[0].matches(line)) {
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

/** Wait for a line matching `matches`, or fail loudly. */
function waitFor(matches, description) {
  for (const line of lines) {
    if (matches(line)) return Promise.resolve(line);
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

const isSummaryRow = (line) => line.includes("1 file, 0 directories");

// A change record the engine may legitimately add, which this section does not pin.
//
// Rendered as text it is `"\tinvalidate"` — an empty path, because the root is what was
// invalidated. The reason is `WatchContention`, which the engine emits after its
// optimistic apply loop loses three times: it reads the clock, verifies the sample with
// filesystem I/O outside the index lock, then commits only if the clock has not moved.
// When concurrent commits keep moving it, reconciling the root is the correct answer,
// because the alternatives are doing that I/O under the lock or letting a stale sample
// win.
//
// It is not the operating system dropping events. That is a different reason
// (`WatchOverflow`) and it is not what fires here; the distinction matters because it
// decides whether the fix belongs in the product or in this capture.
//
// So the product is left alone, and the line is dropped here instead: whether the loop
// loses three times depends on how commits interleave, which is not something a golden
// can pin. This section's contract is where the separator falls. The sibling
// `watch-capture` helper pins the change vocabulary, and it needs no filter because it
// matches each record by exact path and operation.
const isInvalidation = (line) => line.endsWith("\tinvalidate");

try {
  // Waiting for the initial report whole is what proves the watcher is bound before the
  // change is made; skipping it would race the scan and lose the event.
  await waitFor(isSummaryRow, "the initial report");
  const beforeChange = lines.length;

  writeFileSync(join(tree, "added.txt"), "hello, again");

  await waitFor((line) => line.startsWith("────"), "the repaint separator");
  const closing = await waitFor(
    (line) => line.includes("2 files, 0 directories"),
    "the repainted summary",
  );
  if (failure) throw new Error(failure);
  done = true;

  // Everything from the initial report through the first complete repaint. Printed as
  // captured so the golden shows the boundary in place rather than a claim about it.
  const end = lines.lastIndexOf(closing) + 1;
  for (const line of lines.slice(0, end)) {
    if (isInvalidation(line)) continue;
    console.log(line);
  }
  // The separator belongs between repaints, never above the first, so where it is not is
  // as much of the contract as where it is.
  if (lines.slice(0, beforeChange).some((line) => line.startsWith("────"))) {
    console.error("the initial report was preceded by a repaint separator");
    child.kill("SIGKILL");
    process.exit(1);
  }
} catch (error) {
  console.error(error.message);
  if (stderr.trim()) console.error(stderr.trim());
  child.kill("SIGKILL");
  process.exit(1);
}

child.kill("SIGKILL");
