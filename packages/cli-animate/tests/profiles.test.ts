import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { h264Level, masterArgs, PROFILES } from '../src/profiles.js';

describe('profiles', () => {
  const info = { width: 2972, height: 1792, fps: 60, scale: 2 };

  it('pins the lowest legal H.264 level', () => {
    assert.equal(h264Level(1920, 1080, 60), '4.2');
    assert.equal(h264Level(1486, 896, 60), '4.2');
    assert.equal(h264Level(2972, 1792, 60), '5.2');
    assert.equal(h264Level(1280, 720, 30), '3.1');
  });

  it('converts and tags the web profile BT.709 with no B-frames', () => {
    const web = PROFILES.web.deriveArgs(info);
    assert.ok(web.join(' ').includes('out_color_matrix=bt709:out_range=tv'));
    assert.equal(web[web.indexOf('-level:v') + 1], '5.2');
    assert.equal(web[web.indexOf('-bf') + 1], '0');
    assert.equal(web[web.indexOf('-colorspace') + 1], 'bt709');
  });

  it('keeps the master lossless RGB and makes the GIF at 1× and 25 fps', () => {
    assert.ok(masterArgs(60).includes('libx264rgb'));
    assert.equal(masterArgs(60)[masterArgs(60).indexOf('-crf') + 1], '0');
    assert.match(PROFILES.gif.deriveArgs(info).join(' '), /fps=25,scale=1486:-1/);
  });
});
