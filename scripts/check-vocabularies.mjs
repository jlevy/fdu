#!/usr/bin/env node
// Every vocabulary the engine defines must exist whole on the Python surface.
//
// These enums are the consumer contract's words -- an issue kind, a lifecycle phase, a
// state transition -- and an adapter branches on them. They are declared twice: once in
// `fdu-core`, whose `as_str` gives each member its wire label, and once in the Python
// package as a `StrEnum` that parses those labels back. Two declarations of one vocabulary
// drift, and this one did: `IssueKind::ObservationGap` was added to the engine and not to
// Python, so the first batch carrying it raised `ValueError` from inside `_operation_error`
// rather than being a value a caller could branch on.
//
// The parity harness eventually caught that, but only because a watch capture happened to
// provoke an escalation, and only after a ten-minute gate. This asks the question directly:
// the member sets must be equal, not merely overlapping. A label present on one side only
// is a caller who cannot see a state, or a caller who can name one that never arrives.
//
// Read from source rather than from a running binary on purpose. It costs no build, so it
// can run first, and a mismatch is a fact about the declarations rather than about whatever
// one execution happened to emit.

import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const rustSource = readFileSync(join(root, 'crates/fdu-core/src/engine_contract.rs'), 'utf8');
const pythonSource = readFileSync(join(root, 'crates/fdu-py/python/fdu/_models.py'), 'utf8');

/** The vocabularies that must agree, and where each side declares them. */
const VOCABULARIES = [
  { rust: 'impl IssueKind', python: 'class IssueKind(StrEnum)', name: 'IssueKind' },
  { rust: 'impl Phase', python: 'class Phase(StrEnum)', name: 'Phase' },
];

/** Wire labels from a Rust `as_str` block: `Self::Member => "label",`. */
function rustLabels(marker) {
  const start = rustSource.indexOf(marker);
  if (start < 0) {
    throw new Error(`no ${marker} in engine_contract.rs`);
  }
  // To the end of the impl block, which is the first line that closes at column zero.
  const end = rustSource.indexOf('\n}\n', start);
  const body = rustSource.slice(start, end < 0 ? undefined : end);
  const labels = [...body.matchAll(/Self::\w+ => "([a-z_]+)"/g)].map((match) => match[1]);
  if (labels.length === 0) {
    throw new Error(`${marker} declares no wire labels; has as_str moved?`);
  }
  return new Set(labels);
}

/** Member values from a Python `StrEnum` block: `MEMBER = "label"`. */
function pythonLabels(marker) {
  const start = pythonSource.indexOf(marker);
  if (start < 0) {
    throw new Error(`no ${marker} in _models.py`);
  }
  // To the next top-level declaration, so a member of the following class cannot be read
  // as one of this one.
  const rest = pythonSource.slice(start + marker.length);
  const end = rest.search(/\n(?:@dataclass|class |def )/);
  const body = rest.slice(0, end < 0 ? undefined : end);
  const labels = [...body.matchAll(/^ {4}[A-Z][A-Z_]* = "([a-z_]+)"/gm)].map((match) => match[1]);
  if (labels.length === 0) {
    throw new Error(`${marker} declares no members; has the class moved?`);
  }
  return new Set(labels);
}

const problems = [];
for (const { rust, python, name } of VOCABULARIES) {
  const engine = rustLabels(rust);
  const surface = pythonLabels(python);
  const missing = [...engine].filter((label) => !surface.has(label)).sort();
  const extra = [...surface].filter((label) => !engine.has(label)).sort();
  if (missing.length > 0) {
    problems.push(`${name}: the engine can produce ${missing.join(', ')}; Python cannot parse it`);
  }
  if (extra.length > 0) {
    problems.push(`${name}: Python declares ${extra.join(', ')}; the engine never produces it`);
  }
}

if (problems.length > 0) {
  console.error('vocabulary self-check failed:\n');
  for (const problem of problems) {
    console.error(`  ${problem}`);
  }
  console.error(
    '\nA vocabulary is the consumer contract. Declare the member on both surfaces, or\n' +
      'remove it from both -- a label only one side knows is a state a caller cannot act on.',
  );
  process.exit(1);
}

const counted = VOCABULARIES.map(({ rust, name }) => `${name} (${rustLabels(rust).size})`);
console.log(`vocabulary self-check passed: ${counted.join(', ')} agree across both surfaces`);
