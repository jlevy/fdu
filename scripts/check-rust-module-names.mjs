#!/usr/bin/env node

import { lstat, readdir } from "node:fs/promises";
import { basename, dirname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const CRATE_ROOT_BASENAMES = new Set(["build.rs", "lib.rs", "main.rs"]);
const FORBIDDEN_BASENAMES = new Set([
  "basic.rs",
  "code.rs",
  "common.rs",
  "detect.rs",
  "helper.rs",
  "helpers.rs",
  "parse.rs",
  "types.rs",
  "util.rs",
  "utils.rs",
]);
const RUST_SURFACES = ["examples", "src", "tests"];

function compareText(left, right) {
  if (left < right) return -1;
  if (left > right) return 1;
  return 0;
}

async function walkRust(directory, paths, excludeFixtures) {
  let entries;
  try {
    entries = await readdir(directory, { withFileTypes: true });
  } catch (error) {
    if (error?.code === "ENOENT") return;
    throw error;
  }

  entries.sort((left, right) => compareText(left.name, right.name));
  for (const entry of entries) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      if (excludeFixtures && entry.name === "fixtures") continue;
      await walkRust(path, paths, excludeFixtures);
    } else if (entry.isFile() && entry.name.endsWith(".rs")) {
      paths.push(path);
    }
  }
}

async function addBuildScript(crateDirectory, paths) {
  const path = join(crateDirectory, "build.rs");
  try {
    if ((await lstat(path)).isFile()) paths.push(path);
  } catch (error) {
    if (error?.code !== "ENOENT") throw error;
  }
}

/** Collect human-authored Rust files from crate source, test, and example surfaces. */
export async function collectRustFiles(repositoryRoot) {
  const cratesDirectory = join(repositoryRoot, "crates");
  let crates;
  try {
    crates = await readdir(cratesDirectory, { withFileTypes: true });
  } catch (error) {
    if (error?.code === "ENOENT") return [];
    throw error;
  }

  const paths = [];
  crates.sort((left, right) => compareText(left.name, right.name));
  for (const crate of crates) {
    if (!crate.isDirectory()) continue;
    const crateDirectory = join(cratesDirectory, crate.name);
    await addBuildScript(crateDirectory, paths);
    for (const surface of RUST_SURFACES) {
      await walkRust(
        join(crateDirectory, surface),
        paths,
        surface === "examples" || surface === "tests",
      );
    }
  }
  return paths.sort(compareText);
}

function displayPath(repositoryRoot, path) {
  return relative(repositoryRoot, path).split(sep).join("/");
}

function isStandardCrateRoot(repositoryRoot, path) {
  const parts = relative(repositoryRoot, path).split(sep);
  if (parts.length === 3) {
    return parts[0] === "crates" && parts[2] === "build.rs";
  }
  return (
    parts.length === 4 &&
    parts[0] === "crates" &&
    parts[2] === "src" &&
    (parts[3] === "lib.rs" || parts[3] === "main.rs")
  );
}

function violation(kind, fileBasename, paths) {
  return { kind, basename: fileBasename, paths: [...paths].sort(compareText) };
}

/** Audit the objective parts of fdu's Rust filename convention. */
export function auditRustModuleNames(paths, repositoryRoot) {
  const violations = [];
  const duplicateCandidates = new Map();

  for (const path of [...paths].sort(compareText)) {
    const fileBasename = basename(path);
    const shown = displayPath(repositoryRoot, path);
    if (fileBasename === "mod.rs") {
      violations.push(violation("legacy-module-root", fileBasename, [shown]));
      continue;
    }
    if (CRATE_ROOT_BASENAMES.has(fileBasename)) {
      if (!isStandardCrateRoot(repositoryRoot, path)) {
        violations.push(violation("misplaced-crate-root", fileBasename, [shown]));
      }
      continue;
    }
    if (FORBIDDEN_BASENAMES.has(fileBasename)) {
      violations.push(violation("forbidden-basename", fileBasename, [shown]));
      continue;
    }
    const matches = duplicateCandidates.get(fileBasename) ?? [];
    matches.push(shown);
    duplicateCandidates.set(fileBasename, matches);
  }

  for (const [fileBasename, matches] of duplicateCandidates) {
    if (matches.length > 1) {
      violations.push(violation("duplicate-basename", fileBasename, matches));
    }
  }

  return violations.sort(
    (left, right) =>
      compareText(left.basename, right.basename) || compareText(left.kind, right.kind),
  );
}

function violationHeading({ kind, basename: fileBasename }) {
  switch (kind) {
    case "duplicate-basename":
      return `duplicate basename "${fileBasename}"; use subsystem-specific responsibility names:`;
    case "forbidden-basename":
      return `context-dependent basename "${fileBasename}"; name the owned responsibility:`;
    case "legacy-module-root":
      return `legacy module root "${fileBasename}"; use foo.rs with children in foo/:`;
    case "misplaced-crate-root":
      return `crate-root basename "${fileBasename}" outside its standard Cargo location:`;
    default:
      throw new Error(`unknown Rust filename violation kind: ${kind}`);
  }
}

/** Format violations for stable local and CI diagnostics. */
export function formatViolations(violations) {
  if (violations.length === 0) return "";
  const lines = ["Rust module filename check failed:"];
  for (const current of violations) {
    lines.push(`- ${violationHeading(current)}`);
    for (const path of current.paths) lines.push(`  - ${path}`);
  }
  return lines.join("\n");
}

export async function main() {
  const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
  const paths = await collectRustFiles(repositoryRoot);
  const violations = auditRustModuleNames(paths, repositoryRoot);
  if (violations.length > 0) {
    console.error(formatViolations(violations));
    process.exitCode = 1;
    return;
  }
  console.log(`Rust module filenames: checked ${paths.length} files`);
}

const invokedPath = process.argv[1] && resolve(process.argv[1]);
if (invokedPath === fileURLToPath(import.meta.url)) {
  try {
    await main();
  } catch (error) {
    console.error(`Rust module filename check failed: ${error?.message ?? error}`);
    process.exitCode = 1;
  }
}
