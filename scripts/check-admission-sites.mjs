#!/usr/bin/env node

// Every directory-listing loop must route its observed kind through the shared
// admission decision. This catches platform-gated fast paths that ordinary host tests
// cannot execute, especially macOS getattrlistbulk loops reviewed from Linux.

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const repositoryRoot = dirname(dirname(fileURLToPath(import.meta.url)));
const scanPath = "crates/fdu-core/src/scan.rs";
const watchPath = "crates/fdu-core/src/watch.rs";
const scan = readFileSync(join(repositoryRoot, scanPath), "utf8");
const watch = readFileSync(join(repositoryRoot, watchPath), "utf8");
const lines = scan.split("\n");
const listingLoop = /^\s*for\s+(item|entry)\s+in\s+(listing|entries)\s*\{\s*$/;
const routedCalls = ["admission::decide(", "record_walk_entry(", "process_entry("];

function blockBody(start) {
  let depth = 0;
  const collected = [];
  for (let index = start; index < lines.length; index += 1) {
    const line = lines[index];
    for (const character of line) {
      if (character === "{") depth += 1;
      if (character === "}") depth -= 1;
    }
    if (index > start) collected.push(line);
    if (depth === 0 && index > start) break;
  }
  return collected.join("\n");
}

const problems = [];
let listingLoops = 0;
for (const [index, line] of lines.entries()) {
  if (!listingLoop.test(line)) continue;
  listingLoops += 1;
  const body = blockBody(index);
  if (!routedCalls.some((call) => body.includes(call))) {
    problems.push(
      `${scanPath}:${index + 1}: ${line.trim()} bypasses the admission chokepoint`,
    );
  }
}

// A source-shape check that silently stops matching is worse than no check. Update this
// count deliberately whenever a producer loop is added, removed, or rewritten.
if (listingLoops !== 8) {
  problems.push(
    `${scanPath}: expected 8 directory-listing loops, found ${listingLoops}; ` +
      "audit the changed producer topology and update this check",
  );
}

for (const chokepoint of [
  "fn record_walk_entry(",
  "let process_entry = |",
  "let mut process_entry =",
]) {
  if (!scan.includes(chokepoint)) {
    problems.push(`${scanPath}: missing audited admission chokepoint ${chokepoint}`);
  }
}
if ((scan.match(/admission::decide\(/g) ?? []).length < 5) {
  problems.push(`${scanPath}: fewer than five shared admission decisions remain`);
}
if (!scan.includes("crate::admission::should_descend(")) {
  problems.push(`${scanPath}: traversal no longer delegates to shared admission`);
}
if (!watch.includes("crate::admission::decide_path(")) {
  problems.push(`${watchPath}: applying watch verification bypasses admission`);
}

if (problems.length > 0) {
  console.error("admission self-check failed:\n");
  for (const problem of problems) console.error(`  ${problem}`);
  process.exit(1);
}

console.log(
  `admission self-check passed: ${listingLoops} listing loops and watch verification are routed`,
);
