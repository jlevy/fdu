/**
 * `render`: turn a cast into video by stepping the stage page on the recording's clock.
 *
 * Capture happens once, into a lossless RGB master. The frame at time t is the page's
 * drawing after seeking to t, so no frame depends on how fast this machine renders: a
 * slow screenshot delays the encode, never the picture. A frame whose interval holds no
 * terminal output reuses the previous screenshot. The viewport is the stage's own size
 * (the window plus a margin), so the video is cropped to the terminal.
 *
 * Every delivery (web MP4, README GIF) is then derived from the master by ffmpeg, so
 * `verify`, which reads the master, checks the same frames every delivery shows.
 */

import { type ChildProcess, spawn, spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, rmSync } from 'node:fs';
import { dirname } from 'node:path';
import { chromium } from 'playwright-core';
import { CliError } from './errors.js';
import { missingFaces } from './fonts.js';
import { publish, temporarySibling, writeJsonAtomic } from './fsutil.js';
import { masterArgs, type ProfileName, profile as getProfile } from './profiles.js';
import { startStageServer } from './server.js';
import { findProgram, programVersion, whichProgram } from './tools.js';

export interface RenderOptions {
  profile?: string;
  fps?: number;
  /** Device pixels per CSS pixel; 2 keeps glyph edges sharp. */
  scale?: number;
  /** Seconds to hold the last frame. */
  holdEnd?: number;
  margin?: number;
  fontSize?: number;
  chromePath?: string;
  /** Keep the lossless master here; otherwise it is a temporary. */
  masterPath?: string;
  onProgress?: (done: number, total: number) => void;
}

export interface Capture {
  master: string;
  fps: number;
  scale: number;
  width: number;
  height: number;
  frames: number;
  screenshots: number;
  cast_duration_seconds: number;
  capture_seconds: number;
  chromium: string;
}

export interface StreamInfo {
  codec?: string;
  profile?: string;
  level?: string;
  pix_fmt?: string;
  color_space?: string;
  color_transfer?: string;
  color_primaries?: string;
}

export interface RenderReceipt {
  kind: 'cli-animate.render';
  video: string;
  cast: string;
  profile: ProfileName;
  capture: Omit<Capture, 'master'> & { master: string | null };
  encoder_args: string[];
  stream: StreamInfo;
  tools: Record<string, string>;
}

interface StageInfo {
  size: { width: number; height: number };
  duration: number;
  eventTimes: number[];
}

/** The receipt beside a video: `demo.mp4` → `demo.mp4.json`, so each delivery has its own. */
export function videoReceiptPath(videoPath: string): string {
  return `${videoPath}.json`;
}

function exited(child: ChildProcess, what: string): Promise<void> {
  return new Promise((resolve, reject) => {
    child.on('error', reject);
    child.on('close', (code) => (code === 0 ? resolve() : reject(new CliError(`${what} exited with status ${String(code)}`, 1))));
  });
}

/** Capture a cast into a lossless RGB master at `masterPath`. */
export async function capture(castPath: string, masterPath: string, options: RenderOptions = {}): Promise<Capture> {
  if (!existsSync(castPath)) throw new CliError(`cast not found: ${castPath}`);
  if (missingFaces().length > 0) throw new CliError('fonts are missing or unverified: run `cli-animate fonts` first');
  const fps = options.fps ?? 60;
  const scale = options.scale ?? 2;
  const ffmpeg = findProgram('ffmpeg');
  const executablePath = options.chromePath ?? process.env.CHROME_PATH;

  const server = await startStageServer(castPath);
  const browser = await chromium
    .launch({
      ...(executablePath ? { executablePath } : {}),
      args: [
        '--force-color-profile=srgb',
        '--font-render-hinting=none',
        '--disable-background-networking',
        '--disable-component-update',
        '--disable-background-timer-throttling',
        '--disable-renderer-backgrounding',
      ],
    })
    .catch(async (error: Error) => {
      await server.close();
      throw new CliError(`cannot launch Chromium (set CHROME_PATH or pass --chrome): ${error.message.split('\n')[0]}`);
    });
  const started = performance.now();
  mkdirSync(dirname(masterPath), { recursive: true });
  const temporary = temporarySibling(masterPath);
  try {
    // Lay out in a viewport larger than any window, then shrink it to the stage.
    const page = await browser.newPage({ viewport: { width: 4000, height: 3000 }, deviceScaleFactor: scale });
    const query = new URLSearchParams({ mode: 'capture', margin: String(options.margin ?? 32), font: String(options.fontSize ?? 22) });
    await page.goto(`${server.url}/stage.html?${query.toString()}`);
    await page.waitForFunction(() => 'stageReady' in window || 'stageError' in window, null, { timeout: 30_000 });
    const failure = await page.evaluate(() => (window as unknown as { stageError?: string }).stageError);
    if (failure) throw new CliError(`stage failed: ${failure}`);
    const stage = await page.evaluate(() => {
      const s = (window as unknown as { stage: StageInfo }).stage;
      return { size: s.size, duration: s.duration, eventTimes: s.eventTimes };
    });
    await page.setViewportSize(stage.size);

    const total = Math.ceil((stage.duration + (options.holdEnd ?? 2)) * fps);
    const child = spawn(
      ffmpeg,
      ['-y', '-loglevel', 'error', '-f', 'image2pipe', '-framerate', String(fps), '-i', '-', ...masterArgs(fps), '-r', String(fps), temporary],
      { stdio: ['pipe', 'inherit', 'inherit'] },
    );
    // If ffmpeg dies, report its exit status rather than a broken pipe.
    let failed: Error | undefined;
    const done = exited(child, 'ffmpeg').catch((error: Error) => {
      failed = error;
    });
    const stdin = child.stdin!;
    stdin.on('error', () => undefined);
    const send = (frame: Buffer): Promise<void> => {
      if (failed) return Promise.reject(failed);
      return new Promise((resolve) => {
        if (stdin.write(frame)) return resolve();
        const settle = (): void => {
          stdin.off('drain', settle);
          stdin.off('close', settle);
          resolve();
        };
        stdin.on('drain', settle);
        stdin.on('close', settle);
      });
    };

    let frame: Buffer | undefined;
    let cursor = 0;
    let screenshots = 0;
    for (let i = 0; i < total; i++) {
      const t = Math.min(i / fps, stage.duration);
      let changed = frame === undefined;
      while (cursor < stage.eventTimes.length && stage.eventTimes[cursor]! <= t) {
        cursor++;
        changed = true;
      }
      if (changed) {
        await page.evaluate((tt) => (window as unknown as { stage: { seek(t: number): Promise<void> } }).stage.seek(tt), t);
        frame = await page.screenshot({ type: 'png' });
        screenshots++;
      }
      await send(frame!);
      if (i % fps === 0) options.onProgress?.(i, total);
    }
    stdin.end();
    await done;
    if (failed) throw failed;
    publish(temporary, masterPath);
    return {
      master: masterPath,
      fps,
      scale,
      width: stage.size.width * scale,
      height: stage.size.height * scale,
      frames: total,
      screenshots,
      cast_duration_seconds: stage.duration,
      capture_seconds: Math.round(performance.now() - started) / 1000,
      chromium: browser.version(),
    };
  } catch (error) {
    rmSync(temporary, { force: true });
    throw error;
  } finally {
    await browser.close();
    await server.close();
  }
}

/** Codec, level, and colour tags of the first video stream, as ffprobe reports them. */
export function probe(videoPath: string): StreamInfo {
  const ffprobe = whichProgram('ffprobe');
  if (!ffprobe) return {};
  const result = spawnSync(
    ffprobe,
    ['-v', 'error', '-select_streams', 'v:0', '-show_entries', 'stream=codec_name,profile,level,pix_fmt,color_space,color_transfer,color_primaries', '-of', 'json', videoPath],
    { encoding: 'utf8' },
  );
  const stream = (JSON.parse(result.stdout || '{}') as { streams?: Record<string, unknown>[] }).streams?.[0] ?? {};
  const level = typeof stream.level === 'number' && stream.level > 0 ? (stream.level / 10).toFixed(1) : undefined;
  const pick = (key: string): string | undefined => (typeof stream[key] === 'string' ? (stream[key] as string) : undefined);
  const info: StreamInfo = {};
  for (const [key, value] of [
    ['codec', pick('codec_name')],
    ['profile', pick('profile')],
    ['level', level],
    ['pix_fmt', pick('pix_fmt')],
    ['color_space', pick('color_space')],
    ['color_transfer', pick('color_transfer')],
    ['color_primaries', pick('color_primaries')],
  ] as const) {
    if (value !== undefined) (info as Record<string, string>)[key] = value;
  }
  return info;
}

/** Describe an existing master from its stream, for deriving deliveries from it later. */
export function masterInfo(masterPath: string, scale: number): Capture {
  if (!existsSync(masterPath)) throw new CliError(`master not found: ${masterPath}`);
  const ffprobe = findProgram('ffprobe');
  const result = spawnSync(
    ffprobe,
    ['-v', 'error', '-select_streams', 'v:0', '-count_packets', '-show_entries', 'stream=width,height,r_frame_rate,nb_read_packets', '-of', 'json', masterPath],
    { encoding: 'utf8' },
  );
  const stream = (JSON.parse(result.stdout || '{}') as { streams?: Record<string, string | number>[] }).streams?.[0];
  if (!stream) throw new CliError(`cannot read a video stream from ${masterPath}`);
  const [num, den] = String(stream.r_frame_rate).split('/').map(Number);
  return {
    master: masterPath,
    fps: Math.round((num ?? 60) / (den || 1)),
    scale,
    width: Number(stream.width),
    height: Number(stream.height),
    frames: Number(stream.nb_read_packets),
    screenshots: 0,
    cast_duration_seconds: 0,
    capture_seconds: 0,
    chromium: 'unknown (derived from an existing master)',
  };
}

/** Derive a delivery from the master with ffmpeg. */
export async function deliver(captured: Capture, videoPath: string, profileName: ProfileName): Promise<string[]> {
  const ffmpeg = findProgram('ffmpeg');
  const args = getProfile(profileName).deriveArgs({ width: captured.width, height: captured.height, fps: captured.fps, scale: captured.scale });
  mkdirSync(dirname(videoPath), { recursive: true });
  const temporary = temporarySibling(videoPath);
  try {
    const child = spawn(ffmpeg, ['-y', '-loglevel', 'error', '-i', captured.master, ...args, temporary], { stdio: ['ignore', 'inherit', 'inherit'] });
    await exited(child, 'ffmpeg');
    publish(temporary, videoPath);
  } catch (error) {
    rmSync(temporary, { force: true });
    throw error;
  }
  return args;
}

/** Write the receipt beside a delivered video and return it. */
export function writeRenderReceipt(
  castPath: string,
  videoPath: string,
  profileName: ProfileName,
  captured: Capture,
  encoderArgs: string[],
  masterKept: boolean,
): RenderReceipt {
  const ffmpeg = findProgram('ffmpeg');
  const { master, ...captureFields } = captured;
  const receipt: RenderReceipt = {
    kind: 'cli-animate.render',
    video: videoPath,
    cast: castPath,
    profile: profileName,
    capture: { ...captureFields, master: masterKept ? master : null },
    encoder_args: encoderArgs,
    stream: probe(videoPath),
    tools: { chromium: captured.chromium, ffmpeg: programVersion(ffmpeg, ['-version']), node: process.version },
  };
  writeJsonAtomic(videoReceiptPath(videoPath), receipt);
  return receipt;
}

/** Capture a cast and deliver it in one profile, writing the video and its receipt. */
export async function render(castPath: string, videoPath: string, options: RenderOptions = {}): Promise<RenderReceipt> {
  const chosen = getProfile(options.profile ?? 'web');
  const keepMaster = chosen.name === 'master' ? videoPath : options.masterPath;
  const masterPath = keepMaster ?? temporarySibling(`${videoPath}.master.mp4`);
  try {
    const captured = await capture(castPath, masterPath, options);
    const args = chosen.name === 'master' ? masterArgs(captured.fps) : await deliver(captured, videoPath, chosen.name);
    return writeRenderReceipt(castPath, videoPath, chosen.name, captured, args, keepMaster !== undefined);
  } finally {
    if (!keepMaster) rmSync(masterPath, { force: true });
  }
}
