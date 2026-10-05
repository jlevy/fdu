import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { expectedChangeFrames, firstFrameAtOrAfter, parseChangedFrames } from '../src/verify.js';

describe('frame mapping', () => {
  it('puts an event in the first frame at or after it, by the renderer’s comparison', () => {
    assert.equal(firstFrameAtOrAfter(1.95, 60), 117);
    assert.equal(firstFrameAtOrAfter(1.9500000000000002, 60), 118);
    assert.equal(firstFrameAtOrAfter(0, 60), 0);
    assert.equal(firstFrameAtOrAfter(0.001, 60), 1);
  });

  it('ignores frame 0, which has no predecessor to differ from', () => {
    assert.deepEqual([...expectedChangeFrames([0, 0.5], 60)], [30]);
  });

  it('reads tblend metadata as the later frame of each changed pair', () => {
    const metadata = 'frame:0    pts:0\nlavfi.signalstats.YMAX=0\nframe:1    pts:1\nlavfi.signalstats.YMAX=212\n';
    assert.deepEqual([...parseChangedFrames(metadata)], [2]);
  });
});
