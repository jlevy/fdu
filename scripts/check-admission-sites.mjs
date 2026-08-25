#!/usr/bin/env node
// Every loop that turns a directory listing into observations must ask about admission.
//
// `HiddenPolicy::admits` decides what is inside the scan scope, and it has to be asked
// from every place the walker enumerates a directory -- the serial scan, the parallel
// scan, both reconciliation paths, and the macOS bulk reader inside three of those. That
// is seven sites for one rule, which is six chances to add an eighth and forget.
//
// It has already happened once. The portable loops were wired and the macOS bulk reads
// were not, so `--hidden prune` worked on Linux and silently kept `.git` on macOS. Nothing
// local caught it: the bulk paths are behind `cfg(target_os = "macos")`, so a Linux test
// run does not compile them and a Linux clippy run does not see them. CI found it, four
// tests and two wheel smokes at once, on a platform this repository cannot execute.
//
// A runtime test cannot cover a path that does not exist on the host running it. This can:
// the loops are visible in the source whatever platform reads it, and the rule -- "a
// listing loop asks about admission" -- is a property of the text. It costs no build, so
// it runs before anything is compiled.
//
// The bulk reader is the case worth naming. It does not look like a listing loop: it hands
// over name, kind and attrs together, so there is no `read_dir` item to hang the check on
// and it reads as a fast path rather than as another place the rule has to hold.

import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const path = 'crates/fdu-core/src/scan.rs';
const source = readFileSync(join(root, path), 'utf8');
const lines = source.split('\n');

/** Loop headers that enumerate one directory's entries, by the binding they walk. */
const LISTING_LOOPS = /^\s*for\s+(item|entry)\s+in\s+(listing|entries)\s*\{\s*$/;

/** The call every one of them must contain. */
const ADMISSION = 'admits(';

/** The body of a block whose opening brace ends `lines[start]`, brace-matched. */
function body(start) {
  let depth = 0;
  const collected = [];
  for (let index = start; index < lines.length; index += 1) {
    const line = lines[index];
    for (const character of line) {
      if (character === '{') depth += 1;
      if (character === '}') depth -= 1;
    }
    if (index > start) collected.push(line);
    if (depth === 0 && index > start) break;
  }
  return collected.join('\n');
}

const problems = [];
let found = 0;
for (const [index, line] of lines.entries()) {
  if (!LISTING_LOOPS.test(line)) continue;
  found += 1;
  if (!body(index).includes(ADMISSION)) {
    problems.push(`${path}:${index + 1}: ${line.trim()} never asks admits()`);
  }
}

// A rule nobody matches is a rule nobody enforces, and a rename of one binding would
// silently empty this check while it went on reporting success.
if (found < 7) {
  console.error(
    `admission self-check: matched only ${found} listing loops in ${path}.\n` +
      'The engine has seven. Either a walk was removed, or the loop shape changed and this\n' +
      'check stopped seeing the sites it exists to police -- which looks exactly like passing.',
  );
  process.exit(1);
}

if (problems.length > 0) {
  console.error('admission self-check failed:\n');
  for (const problem of problems) {
    console.error(`  ${problem}`);
  }
  console.error(
    '\nA listing loop decides what enters the index. One that does not ask `admits` puts\n' +
      'pruned entries back, and if it is behind a platform gate no local run will say so.',
  );
  process.exit(1);
}

console.log(`admission self-check passed: ${found} listing loops all ask admits()`);
