import assert from 'node:assert/strict';
import { test } from 'node:test';

import { lineMatcher, restorePatterns } from './golden-restore-patterns.mjs';

const committed = `---
patterns:
  PERF_TIME: '[\\d.]+ (ns|µs|ms|s)'
---
\`\`\`console
$ fdu x
! note: old wording
! perf: took [PERF_TIME] to walk 2 files at [..]
? 0
\`\`\`
`;

test('an expanded pattern line is restored and a reworded line is kept', () => {
  const updated = committed
    .replace('! note: old wording', '! note: new wording')
    .replace('took [PERF_TIME] to walk 2 files at [..]', 'took 1.2 ms to walk 2 files at 9 files/s');
  const { text, restored } = restorePatterns(committed, updated);
  assert.equal(restored, 1);
  assert.match(text, /! note: new wording/);
  assert.match(text, /took \[PERF_TIME\] to walk 2 files at \[\.\.\]/);
});

test('a line whose literal text changed is not restored', () => {
  const updated = committed.replace(
    'took [PERF_TIME] to walk 2 files at [..]',
    'took 1.2 ms to walk 3 files at 9 files/s',
  );
  const { text, restored } = restorePatterns(committed, updated);
  assert.equal(restored, 0);
  assert.match(text, /walk 3 files/);
});

test('front-matter patterns are honored, and elision lines are never paired', () => {
  const matcher = lineMatcher('took [PERF_TIME] total', { PERF_TIME: '[\\d.]+ (ns|µs|ms|s)' });
  assert.ok(matcher.test('took 1.5 ms total'));
  assert.ok(!matcher.test('took soon total'));
  assert.equal(lineMatcher('...', {}), null);
  assert.equal(lineMatcher('no patterns here', {}), null);
});
