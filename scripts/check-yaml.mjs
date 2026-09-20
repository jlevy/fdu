#!/usr/bin/env node
// Parse fdu's YAML output with a real YAML parser.
//
// A byte-stable golden proves the output has not *changed*; it never proves the output is
// *valid*. A consistently malformed document passes forever, and this serializer is
// hand-written -- the project avoids serde -- so nothing else would notice. Until this
// script, no YAML parser had ever read fdu's YAML.
//
// The parser is `yaml`, already in the locked dev tree and integrity-pinned, so this
// costs no new dependency.

import { execFileSync } from 'node:child_process';
import { mkdtempSync, writeFileSync, mkdirSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';
import { parse } from 'yaml';

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const fdu = process.env.FDU_BIN ?? join(root, 'target', 'debug', 'fdu');

// A tree with enough shape that every view has rows to render.
const tree = mkdtempSync(join(tmpdir(), 'fdu-yaml-'));
mkdirSync(join(tree, 'src'));
mkdirSync(join(tree, 'docs'));
writeFileSync(join(tree, 'src', 'main.rs'), 'fn main() {\n    // hi\n}\n');
writeFileSync(join(tree, 'src', 'app.py'), '# c\nprint(1)\n');
writeFileSync(join(tree, 'docs', 'guide.md'), '# T\n\nsome prose words here\n');
writeFileSync(join(tree, 'data.json'), '{"k": 1}\n');
writeFileSync(join(tree, 'docs', 'notes.hs'), 'main = putStrLn "hi"\n');

for (const name of ['0x10', '1_000', '0o17', '.1', '._1', '.inf', '.NaN', 'y',
  'braces { remain } [ intact ].txt', 'del\u007fname.txt', 'nel\u0085name.txt',
  'line\u2028separator.txt']) {
  writeFileSync(join(tree, 'docs', name), 'x\n');
}
try {
  writeFileSync(join(tree, 'docs', 'noncharacter\ufffe.txt'), 'x\n');
} catch (error) {
  assert.equal(process.platform, 'darwin', 'only Darwin is expected to reject U+FFFE in a name');
  assert.ok(['EPERM', 'EINVAL'].includes(error.code),
    `unexpected failure creating the platform-optional noncharacter filename: ${error}`);
}
try {
  writeFileSync(Buffer.concat([
    Buffer.from(join(tree, 'docs', 'raw-')), Buffer.from([0xff]), Buffer.from('.bin'),
  ]), 'x\n');
} catch (error) {
  assert.notEqual(process.platform, 'linux', 'Linux must preserve a non-UTF-8 filename');
  assert.ok(['EPERM', 'EINVAL', 'EILSEQ'].includes(error.code),
    `unexpected failure creating the platform-optional raw filename: ${error}`);
}

const views = [
  'tree', 'types', 'extensions', 'families', 'languages', 'documents',
  'files', 'largest', 'recent', 'summary',
];

let checked = 0;
for (const view of views) {
  const args = ['--cache', 'off', '--format', 'yaml', '--view', view, '-n', 'all', '--depth', 'all'];
  // `documents` needs content, and analysing it also exercises the analysis block.
  if (view === 'documents') args.push('--analyze', 'words');
  const out = execFileSync(fdu, [...args, tree], { encoding: 'utf8' });

  const parsed = parse(out, { strict: true, uniqueKeys: true, intAsBigInt: true, version: '1.2' });
  assert.ok(parsed, `${view}: parsed to nothing`);
  assert.ok(typeof parsed.schema === 'string' && parsed.schema.startsWith('fdu.report/'),
    `${view}: no schema in parsed YAML`);
  assert.equal(parsed.status.complete, true, `${view}: complete did not survive the round trip`);
  assert.ok(Array.isArray(parsed.reports) && parsed.reports.length === 1,
    `${view}: expected exactly one report section`);
  assert.equal(parsed.reports[0].view, view, `${view}: section names a different view`);
  checked += 1;
}

// A bounded view must carry its bound through the round trip, since the whole point of
// the field is that a consumer can read it.
const bounded = parse(execFileSync(
  fdu, ['--cache', 'off', '--format', 'yaml', '--view', 'largest', '-n', '1', tree],
  { encoding: 'utf8' },
));
assert.equal(bounded.reports[0].bound.shown, 1, 'bound.shown lost in YAML');
assert.ok(bounded.reports[0].bound.total > 1, 'bound.total lost in YAML');

// And an unbounded one says null rather than omitting the key, so a consumer can branch
// on the value instead of on presence.
const unbounded = parse(execFileSync(
  fdu, ['--cache', 'off', '--format', 'yaml', '--view', 'files', tree],
  { encoding: 'utf8' },
));
assert.ok('bound' in unbounded.reports[0], 'bound key missing when nothing was dropped');
assert.equal(unbounded.reports[0].bound, null, 'an unbounded view must say null');

const exactJson = (source) => JSON.parse(source, (key, value, context) => {
  if (typeof value === 'number' && Number.isInteger(value)) {
    assert.ok(context?.source, `JSON parser did not expose exact source for ${key}`);
    return BigInt(context.source);
  }
  return value;
});

const stripVolatile = (value) => {
  if (Array.isArray(value)) return value.map(stripVolatile);
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.entries(value)
      .filter(([key]) => !['scan_started_at', 'generated_at', 'observed_at_ns'].includes(key))
      .map(([key, inner]) => [key, stripVolatile(inner)]));
  }
  return value;
};

const reassembleJsonl = (source) => {
  const [envelope, ...reports] = source.trimEnd().split('\n').map(exactJson);
  return { ...envelope, reports };
};

// The Rust unit test pins the live sink to this fixture. Parsing that same fixture here
// keeps the language/runtime dependency in the explicit Node-backed gate.
const scalarJson = JSON.parse(readFileSync(
  join(root, 'crates', 'fdu-core', 'src', 'testdata', 'yaml-scalar-corpus.json'), 'utf8',
)).map((entry) => entry.s);
const scalarYaml = readFileSync(
  join(root, 'crates', 'fdu-core', 'src', 'testdata', 'yaml-scalar-corpus.yaml'), 'utf8',
);
for (const version of ['1.1', '1.2']) {
  assert.deepStrictEqual(parse(scalarYaml, {
    strict: true, uniqueKeys: true, intAsBigInt: true, version,
  }), scalarJson, `strict YAML ${version} changed an adversarial scalar`);
}

let compared = 0;
for (const view of [...views, 'full']) {
  for (const analyze of ['none', 'lines', 'code', 'words', 'all']) {
    if (view === 'documents' && !['words', 'all'].includes(analyze)) continue;
    const args = [
      '--cache', 'off', '--view', view, '--analyze', analyze,
      '-n', 'all', '--depth', 'all', tree,
    ];
    const yaml = execFileSync(fdu, [...args, '--format', 'yaml'], { encoding: 'utf8' });
    assert.doesNotMatch(yaml, /[\u007f-\u009f\u2028\u2029\ufeff\ufffe\uffff]/u,
      `${view} --analyze ${analyze}: raw YAML-forbidden character`);
    const json = execFileSync(fdu, [...args, '--format', 'json'], { encoding: 'utf8' });
    const jsonl = execFileSync(fdu, [...args, '--format', 'jsonl'], { encoding: 'utf8' });
    const expected = stripVolatile(exactJson(json));
    const requested = new Set(expected.request.analyze);
    const metricOwners = {
      lines: ['physical_lines', 'blank_lines', 'nonblank_lines', 'raw_words'],
      code: ['code_lines', 'comment_lines', 'code_blank_lines'],
      words: ['logical_words', 'paragraphs', 'visible_words', 'visible_logical_words',
        'document_words'],
    };
    for (const section of expected.reports.filter((item) => item.metrics)) {
      for (const row of [section.metrics.total, ...section.metrics.rows]) {
        assert.deepStrictEqual(Object.keys(row.coverage),
          ['lines', 'code', 'words'].filter((unit) => requested.has(unit)),
          `${view} --analyze ${analyze}: coverage unit presence drifted`);
        for (const [unit, names] of Object.entries(metricOwners)) {
          for (const name of names) {
            assert.equal(name in row.metrics, requested.has(unit),
              `${view} --analyze ${analyze}: ${name} presence drifted`);
          }
        }
        assert.equal('pages' in row, requested.has('words'),
          `${view} --analyze ${analyze}: pages presence drifted`);
      }
    }
    for (const version of ['1.1', '1.2']) {
      const actual = stripVolatile(parse(yaml, {
        strict: true, uniqueKeys: true, intAsBigInt: true, version,
      }));
      assert.deepStrictEqual(actual, expected,
        `${view} --analyze ${analyze}: YAML ${version} differs from JSON`);
    }
    assert.deepStrictEqual(stripVolatile(reassembleJsonl(jsonl)), expected,
      `${view} --analyze ${analyze}: reconstructed JSON Lines differs from JSON`);
    compared += 1;
  }
}

console.log(`yaml self-check passed: ${checked} views parsed, ${compared} cross-format cases`);
