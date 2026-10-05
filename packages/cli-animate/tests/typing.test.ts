import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { baseInterval, DEFAULT_PROFILE, digraphFactor, keyIntervals, keyOf, REFERENCE_TEXT, Rng } from '../src/typing.js';

const mean = (xs: number[]): number => xs.reduce((a, b) => a + b, 0) / xs.length;

describe('keyboard map', () => {
  it('assigns standard touch-typing fingers', () => {
    assert.deepEqual(keyOf('f'), { hand: 'L', finger: 1, row: 2, shifted: false });
    assert.deepEqual(keyOf('l'), { hand: 'R', finger: 3, row: 2, shifted: false });
    assert.deepEqual(keyOf(';'), { hand: 'R', finger: 4, row: 2, shifted: false });
    assert.deepEqual(keyOf('0'), { hand: 'R', finger: 4, row: 0, shifted: false });
    assert.deepEqual(keyOf('P'), { hand: 'R', finger: 4, row: 1, shifted: true });
    assert.deepEqual(keyOf('~'), { hand: 'L', finger: 4, row: 0, shifted: true });
  });
});

describe('digraph factors', () => {
  it('orders alternation < same hand < repeated key < same finger', () => {
    const alternate = digraphFactor('f', 'j');
    const sameHand = digraphFactor('f', 's');
    const repeat = digraphFactor('k', 'k');
    const sameFinger = digraphFactor('f', 'r');
    assert.ok(alternate < sameHand && sameHand < repeat && repeat < sameFinger);
  });

  it('types frequent digraphs faster than their geometric twins', () => {
    assert.ok(digraphFactor('t', 'h') < digraphFactor('t', 'j'));
  });

  it('slows the first key of a word and of a path segment', () => {
    assert.equal(digraphFactor(' ', 'j'), DEFAULT_PROFILE.wordInitial);
    assert.ok(digraphFactor('/', 'j') > digraphFactor('f', 'j'));
  });

  it('charges Shift on the key and after it, and digits for the reach', () => {
    assert.equal(digraphFactor('f', 'J'), DEFAULT_PROFILE.shift);
    assert.equal(digraphFactor('J', 'f'), DEFAULT_PROFILE.afterShift);
    assert.equal(digraphFactor('f', '8'), DEFAULT_PROFILE.digit);
  });
});

describe('keyIntervals', () => {
  it('reproduces a take from its seed and differs across seeds', () => {
    const text = 'fdu ~/src --analyze code';
    assert.deepEqual(keyIntervals(text, new Rng(7)), keyIntervals(text, new Rng(7)));
    assert.notDeepEqual(keyIntervals(text, new Rng(7)), keyIntervals(text, new Rng(8)));
  });

  it('returns one floored interval per character', () => {
    const text = 'git status && cargo build';
    const intervals = keyIntervals(text, new Rng(3));
    assert.equal(intervals.length, [...text].length);
    assert.ok(intervals.every((s) => s >= DEFAULT_PROFILE.floorS));
  });

  it('types the reference text near the target speed', () => {
    for (const wpm of [90, 120, 160, 200]) {
      const profile = { ...DEFAULT_PROFILE, wpm };
      const runs = Array.from({ length: 40 }, (_, seed) => mean(keyIntervals(REFERENCE_TEXT, new Rng(seed), profile)));
      const achieved = 60 / (mean(runs) * 5);
      assert.ok(Math.abs(achieved / wpm - 1) < 0.06, `target ${wpm} WPM, achieved ${achieved.toFixed(1)}`);
    }
  });

  it('scales the base inversely with speed', () => {
    assert.ok(baseInterval({ ...DEFAULT_PROFILE, wpm: 60 }) > baseInterval({ ...DEFAULT_PROFILE, wpm: 120 }));
  });
});
