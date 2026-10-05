/**
 * `verify`: check that a rendered video changes exactly where its cast does.
 *
 * Each output event must change the picture on the first frame at or after it, and the
 * picture must not change on any other frame. Run it on a lossless render: a lossy
 * encode refreshes the picture at each keyframe, which reads as change.
 */

import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { parseCast } from './cast.js';
import { CliError } from './errors.js';
import { findProgram } from './tools.js';

export interface VerifyResult {
  ok: boolean;
  expected_change_frames: number;
  observed_change_frames: number;
  missing: number[];
  unexpected: number[];
}

/**
 * The frame an event appears in: the first i with event <= i / fps. This repeats the
 * renderer's own float comparison; a tolerance would disagree with it for events that sum,
 * through float noise, to just past a frame boundary.
 */
export function firstFrameAtOrAfter(event: number, fps: number): number {
  let i = Math.ceil(event * fps);
  while (i > 0 && event <= (i - 1) / fps) i--;
  while (event > i / fps) i++;
  return i;
}

export function expectedChangeFrames(eventTimes: number[], fps: number): Set<number> {
  const frames = new Set(eventTimes.map((t) => firstFrameAtOrAfter(t, fps)));
  frames.delete(0);
  return frames;
}

/** Parse ffmpeg's per-frame YMAX of consecutive-frame differences; tblend frame k compares k and k+1. */
export function parseChangedFrames(metadata: string): Set<number> {
  const changed = new Set<number>();
  let frame: number | undefined;
  for (const line of metadata.split('\n')) {
    const header = /^frame:(\d+)/.exec(line);
    if (header) frame = Number(header[1]);
    const ymax = /YMAX=([\d.]+)/.exec(line);
    if (ymax && frame !== undefined && Number(ymax[1]) > 0) changed.add(frame + 1);
  }
  return changed;
}

export function verify(castPath: string, videoPath: string, fps = 60): VerifyResult {
  // Inputs first, so a wrong path is reported as one whatever the host has installed.
  const cast = parseCast(readFileSync(castPath, 'utf8'));
  if (!existsSync(videoPath)) throw new CliError(`video not found: ${videoPath}`);
  const ffmpeg = findProgram('ffmpeg');
  const expected = expectedChangeFrames(
    cast.events.filter((e) => e.code === 'o').map((e) => e.time),
    fps,
  );
  const result = spawnSync(
    ffmpeg,
    ['-loglevel', 'error', '-i', videoPath, '-vf', 'format=gray,tblend=all_mode=difference,signalstats,metadata=print:key=lavfi.signalstats.YMAX:file=-', '-f', 'null', '-'],
    { encoding: 'utf8', maxBuffer: 256 * 1024 * 1024 },
  );
  if (result.status !== 0) throw new CliError(`ffmpeg could not read ${videoPath}: ${result.stderr.trim()}`);
  const observed = parseChangedFrames(result.stdout);
  const missing = [...expected].filter((f) => !observed.has(f)).sort((a, b) => a - b);
  const unexpected = [...observed].filter((f) => !expected.has(f)).sort((a, b) => a - b);
  return {
    ok: missing.length === 0 && unexpected.length === 0,
    expected_change_frames: expected.size,
    observed_change_frames: observed.size,
    missing: missing.slice(0, 20),
    unexpected: unexpected.slice(0, 20),
  };
}
