#!/usr/bin/env node

// Every directory-entry producer must route its observed kind through the shared
// admission decision. This catches platform-gated fast paths that ordinary host tests
// cannot execute, especially macOS getattrlistbulk loops reviewed from Linux.

import { readdirSync, readFileSync } from "node:fs";
import { dirname, join, relative, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const SCRIPT_DIRECTORY = dirname(fileURLToPath(import.meta.url));
const ROOT = dirname(SCRIPT_DIRECTORY);
const CORE_SOURCE_DIRECTORY = join(ROOT, "crates", "fdu-core", "src");

const PRODUCERS = new Map([
  [
    "crates/fdu-core/src/scan.rs",
    {
      loops: 8,
      routes: [
        "admission::decide(",
        "emission.record_entry(",
        "record_walk_entry(",
        "process_entry(",
      ],
    },
  ],
  [
    "crates/fdu-core/src/opened.rs",
    { loops: 1, routes: ["prepare_walk_entry("] },
  ],
]);

// These directory readers do not admit filesystem entries into the inventory. A new
// source file containing read_dir fails closed until it is classified here or added to
// PRODUCERS and given a per-loop route check.
const NON_INVENTORY_READERS = new Map([
  ["crates/fdu-core/src/cache.rs", "cache-status enumeration"],
  ["crates/fdu-core/src/snapshot.rs", "snapshot temporary-file housekeeping"],
  ["crates/fdu-core/src/opened/golden_support.rs", "test fixture serialization"],
  ["crates/fdu-core/src/scan/macos_bulk.rs", "platform adapter reference tests"],
]);

const WATCH_PATH = "crates/fdu-core/src/watch.rs";
const LISTING_LOOP = /^\s*for\s+[A-Za-z_][A-Za-z0-9_]*\s+in\s+(?:listing|entries)\s*\{\s*$/;

function rustStructure(source) {
  let result = "";
  const state = { kind: "code", blockDepth: 0, rawHashes: 0 };

  for (let index = 0; index < source.length; index += 1) {
    const character = source[index];
    const next = source[index + 1];
    const keepNewline = () => {
      result += character === "\n" ? "\n" : " ";
    };

    if (state.kind === "line-comment") {
      keepNewline();
      if (character === "\n") state.kind = "code";
      continue;
    }
    if (state.kind === "block-comment") {
      keepNewline();
      if (character === "/" && next === "*") {
        state.blockDepth += 1;
        result += " ";
        index += 1;
      } else if (character === "*" && next === "/") {
        state.blockDepth -= 1;
        result += " ";
        index += 1;
        if (state.blockDepth === 0) state.kind = "code";
      }
      continue;
    }
    if (state.kind === "quoted") {
      keepNewline();
      if (character === "\\") {
        if (next !== undefined) {
          result += next === "\n" ? "\n" : " ";
          index += 1;
        }
      } else if (character === '"') {
        state.kind = "code";
      }
      continue;
    }
    if (state.kind === "raw") {
      keepNewline();
      if (character === '"') {
        const suffix = source.slice(index + 1, index + 1 + state.rawHashes);
        if (suffix === "#".repeat(state.rawHashes)) {
          result += " ".repeat(state.rawHashes);
          index += state.rawHashes;
          state.kind = "code";
        }
      }
      continue;
    }

    if (character === "/" && next === "/") {
      result += "  ";
      index += 1;
      state.kind = "line-comment";
      continue;
    }
    if (character === "/" && next === "*") {
      result += "  ";
      index += 1;
      state.kind = "block-comment";
      state.blockDepth = 1;
      continue;
    }
    if (character === '"') {
      result += " ";
      state.kind = "quoted";
      continue;
    }
    if (character === "r") {
      const raw = source.slice(index).match(/^r(#{0,16})"/);
      if (raw) {
        result += " ".repeat(raw[0].length);
        index += raw[0].length - 1;
        state.kind = "raw";
        state.rawHashes = raw[1].length;
        continue;
      }
    }
    result += character;
  }
  return result;
}

function blockBody(source, structuralLines, start) {
  const sourceLines = source.split("\n");
  let depth = 0;
  const collected = [];
  for (let index = start; index < structuralLines.length; index += 1) {
    for (const character of structuralLines[index]) {
      if (character === "{") depth += 1;
      if (character === "}") depth -= 1;
    }
    if (index > start) collected.push(sourceLines[index]);
    if (depth === 0 && index > start) break;
  }
  return collected.join("\n");
}

export function auditAdmissionSources(sources) {
  const problems = [];
  const loopCounts = new Map();

  for (const [path, source] of sources) {
    if (!source.includes("read_dir(")) continue;
    if (!PRODUCERS.has(path) && !NON_INVENTORY_READERS.has(path)) {
      problems.push(
        `${path}: unclassified directory reader; classify it as an inventory producer or a non-inventory reader`,
      );
    }
  }

  for (const [path, policy] of PRODUCERS) {
    const source = sources.get(path);
    if (source === undefined) {
      problems.push(`${path}: missing audited inventory producer`);
      continue;
    }
    const structuralLines = rustStructure(source).split("\n");
    let loops = 0;
    for (const [index, line] of structuralLines.entries()) {
      if (!LISTING_LOOP.test(line)) continue;
      loops += 1;
      const body = blockBody(source, structuralLines, index);
      if (!policy.routes.some((call) => body.includes(call))) {
        problems.push(
          `${path}:${index + 1}: ${line.trim()} bypasses the admission chokepoint`,
        );
      }
    }
    loopCounts.set(path, loops);
    if (loops !== policy.loops) {
      problems.push(
        `${path}: expected ${policy.loops} directory-listing loops, found ${loops}; ` +
          "audit the changed producer topology and update this check",
      );
    }
  }

  const scan = sources.get("crates/fdu-core/src/scan.rs") ?? "";
  for (const chokepoint of [
    "fn record_detached_entry(",
    "fn record_walk_entry(",
    "let process_entry = |",
    "let mut process_entry =",
  ]) {
    if (!scan.includes(chokepoint)) {
      problems.push(`crates/fdu-core/src/scan.rs: missing audited admission chokepoint ${chokepoint}`);
    }
  }
  const scanStructure = rustStructure(scan).split("\n");
  for (const [implementation, route] of [
    ["impl WalkEmission for StreamingEmission", "record_walk_entry("],
    ["impl WalkEmission for DetachedEmission", "record_detached_entry("],
  ]) {
    const start = scanStructure.findIndex((line) => line.includes(implementation));
    if (start < 0) {
      problems.push(`crates/fdu-core/src/scan.rs: missing audited emission ${implementation}`);
      continue;
    }
    if (!blockBody(scan, scanStructure, start).includes(route)) {
      problems.push(
        `crates/fdu-core/src/scan.rs: ${implementation} bypasses the admission chokepoint ${route}`,
      );
    }
  }
  if ((scan.match(/admission::decide\(/g) ?? []).length < 5) {
    problems.push("crates/fdu-core/src/scan.rs: fewer than five shared admission decisions remain");
  }
  if (!scan.includes("crate::admission::should_descend(")) {
    problems.push("crates/fdu-core/src/scan.rs: traversal no longer delegates to shared admission");
  }

  const watch = sources.get(WATCH_PATH) ?? "";
  if (!watch.includes("crate::admission::decide_path(")) {
    problems.push(`${WATCH_PATH}: applying watch verification bypasses admission`);
  }

  return { problems, loopCounts };
}

function collectRustSources(directory) {
  const sources = new Map();
  const visit = (current) => {
    for (const entry of readdirSync(current, { withFileTypes: true })) {
      const path = join(current, entry.name);
      if (entry.isDirectory()) {
        visit(path);
      } else if (entry.isFile() && entry.name.endsWith(".rs")) {
        const repositoryPath = relative(ROOT, path).split(sep).join("/");
        sources.set(repositoryPath, readFileSync(path, "utf8"));
      }
    }
  };
  visit(directory);
  return sources;
}

function main() {
  const { problems, loopCounts } = auditAdmissionSources(collectRustSources(CORE_SOURCE_DIRECTORY));
  if (problems.length > 0) {
    console.error("admission self-check failed:\n");
    for (const problem of problems) console.error(`  ${problem}`);
    process.exitCode = 1;
    return;
  }

  const summary = [...loopCounts.entries()]
    .map(([path, count]) => `${count} in ${path}`)
    .join(", ");
  console.log(`admission self-check passed: ${summary}; watch verification is routed`);
}

if (process.argv[1] && pathToFileURL(process.argv[1]).href === import.meta.url) {
  main();
}
