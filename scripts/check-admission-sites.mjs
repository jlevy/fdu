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

// -- The second rule, and the second shape it needs ---------------------------------
//
// `admits` is asked by name, inside the listing loop, so the loop shape is a good anchor.
// `retains` cannot be: a kind is only known after the `stat`, so most sites ask it from a
// closure or helper the loop calls rather than from the loop itself. Anchoring on the loop
// would report success for a rule the loop delegates and the delegate forgot.
//
// So this half pins the *set of functions that ask it*, by name. A new producer of rows
// has to appear here, which is a decision rather than an omission -- and the list is the
// documentation of where the rule lives.
//
// It has already happened once, in the change that introduced the rule: the guard went
// into `revalidate`, whose listing loop looks exactly like the reconcilers' and is not on
// their path. A scan excluded a socket and the first refresh put it back. Tests caught it;
// this would have caught it before the tests were written.
const KIND_ADMISSION = {
  'crates/fdu-core/src/scan.rs': {
    call: 'retains(',
    expected: [
      'scan_internal', // the serial walk
      'record_walk_entry', // the parallel walk
      'revalidate', // the warm-start listing sweep
      'reconcile_target_inner', // the serial reconcile, and the single-path refresh
      'reconcile_wave_worker', // the parallel reconcile, via `process_entry`
      'retains', // the predicate's own definition
    ],
  },
  'crates/fdu-core/src/watch.rs': {
    call: 'scan::retains(',
    expected: ['admitted'], // the watcher's apply funnel
  },
};

/** `text` up to its test module, which is not production code and not a producer. */
function production(text) {
  const rows = text.split('\n');
  const start = rows.findIndex(
    (row, index) => row === '#[cfg(test)]' && /^mod tests\b/.test(rows[index + 1] ?? ''),
  );
  return start === -1 ? text : rows.slice(0, start).join('\n');
}

/** Names of top-level `fn`s in `text` whose body contains `call`. */
function callersOf(text, call) {
  const rows = text.split('\n');
  const names = new Set();
  let current = null;
  let depth = 0;
  for (const row of rows) {
    const declared = /^(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:async\s+)?fn\s+([A-Za-z0-9_]+)/.exec(row);
    if (depth === 0 && declared) current = declared[1];
    if (current && row.includes(call)) names.add(current);
    for (const character of row) {
      if (character === '{') depth += 1;
      if (character === '}') depth -= 1;
    }
    if (depth <= 0) depth = 0;
  }
  return names;
}

const kindProblems = [];
for (const [file, rule] of Object.entries(KIND_ADMISSION)) {
  // Read up to the test module: a test asks the predicate too, and a test is not a
  // producer of rows. Cut by the module boundary rather than by a naming convention --
  // the first draft exempted names beginning `a_`, which is how this repository names its
  // tests and would also have exempted a production function that happened to start that
  // way. A rule with a hole shaped like its own subject is worse than no rule.
  const text = production(readFileSync(join(root, file), 'utf8'));
  const actual = callersOf(text, rule.call);
  const expected = new Set(rule.expected);
  for (const name of actual) {
    if (!expected.has(name)) {
      kindProblems.push(
        `${file}: ${name}() asks ${rule.call} and is not a recorded admission site`,
      );
    }
  }
  for (const name of expected) {
    if (!actual.has(name)) {
      kindProblems.push(`${file}: ${name}() no longer asks ${rule.call}`);
    }
  }
}

if (kindProblems.length > 0) {
  console.error('kind-admission self-check failed:\n');
  for (const problem of kindProblems) {
    console.error(`  ${problem}`);
  }
  console.error(
    '\nEvery producer of index rows has to hold the same kind rule. A new one that does\n' +
      'not is an index whose contents depend on which producer last touched a path; a site\n' +
      'that stopped asking is the same defect arriving quietly.',
  );
  process.exit(1);
}

const sites = Object.values(KIND_ADMISSION).reduce((total, rule) => total + rule.expected.length, 0);
console.log(
  `admission self-check passed: ${found} listing loops all ask admits(), ` +
    `and ${sites} recorded sites ask the kind rule`,
);
