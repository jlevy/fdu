import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { extractMarkers, formatCast, markerSequence, markerTimes, parseCast } from '../src/cast.js';

const header = { version: 3 as const, term: { cols: 80, rows: 24 } };

describe('asciicast', () => {
  it('reads v3 intervals as absolute times and writes them back', () => {
    const text = '{"version":3,"term":{"cols":80,"rows":24}}\n[0.5,"o","a"]\n[0.25,"o","b"]\n';
    const cast = parseCast(text);
    assert.deepEqual(cast.events.map((e) => e.time), [0.5, 0.75]);
    assert.equal(formatCast(cast), text);
  });

  it('reads v2 absolute times and v2 sizes', () => {
    const cast = parseCast('{"version":2,"width":100,"height":30}\n[1.5,"o","x"]\n');
    assert.deepEqual(cast.header.term, { cols: 100, rows: 30 });
    assert.equal(cast.events[0]!.time, 1.5);
  });

  it('turns in-band markers into m events at the same instant', () => {
    const data = `before${markerSequence('first')}middle${markerSequence('second')}`;
    const events = extractMarkers([{ time: 2, code: 'o', data }]);
    assert.deepEqual(events, [
      { time: 2, code: 'o', data: 'before' },
      { time: 2, code: 'm', data: 'first' },
      { time: 2, code: 'o', data: 'middle' },
      { time: 2, code: 'm', data: 'second' },
    ]);
    assert.equal(markerTimes(events).get('second'), 2);
  });

  it('keeps marker labels free of control characters', () => {
    assert.ok(!/[\u0000-\u0006\u0008-\u001a\u001c-\u001f]/.test(markerSequence('a\u0007b\nc').slice(1, -1)));
  });

  it('round-trips a cast with markers', () => {
    const cast = { header, events: extractMarkers([{ time: 1, code: 'o' as const, data: `x${markerSequence('m')}y` }]) };
    assert.deepEqual(parseCast(formatCast(cast)).events, cast.events);
  });
});
