#!/usr/bin/env node
// Render an asciicast to video by stepping the web stage on the recording's own clock.
//
// The frame at time t is the page's drawing after seeking to t, captured by a screenshot,
// so the video is the same pixels the web embed shows and no frame depends on how fast
// this machine happens to render: a slow screenshot delays the encode, never the picture.
// Frames whose interval holds no terminal output reuse the previous screenshot, which is
// what makes a mostly static terminal cheap to capture at 60 fps.
//
//   node render.mjs out/realtime.cast out/realtime.mp4 [--engine xterm|asciinema]
//        [--fps 60] [--scale 1|2] [--width 1920 --height 1080] [--font 22]
//        [--hold-end 2] [--codec h264|vp9]

import { createServer } from "node:http";
import { readFile, stat } from "node:fs/promises";
import { spawn, execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright-core";
import { writeFileAtomicSync } from "../../scripts/atomic-write.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const argv = process.argv.slice(2);
const opt = (name, dflt) => {
  const i = argv.indexOf(`--${name}`);
  return i >= 0 ? argv[i + 1] : dflt;
};
const [castArg, outArg] = argv.filter((a, i) => !a.startsWith("--") && !argv[i - 1]?.startsWith("--"));
const engine = opt("engine", "xterm");
const fps = +opt("fps", 60);
const scale = +opt("scale", 1);
const W = +opt("width", 1920), H = +opt("height", 1080);
const font = +opt("font", 22);
const holdEnd = +opt("hold-end", 2);
const codec = opt("codec", outArg.endsWith(".webm") ? "vp9" : "h264");
// Chromium: CHROME_PATH, else whatever playwright-core resolves (it never downloads one).
const chromePath = process.env.CHROME_PATH || undefined;

// Serve the harness directory so the page can fetch the cast and the node_modules assets.
const types = { ".html": "text/html", ".js": "text/javascript", ".css": "text/css",
  ".woff2": "font/woff2", ".woff": "font/woff", ".cast": "text/plain", ".json": "application/json" };
const server = createServer(async (req, res) => {
  const file = path.join(here, decodeURIComponent(new URL(req.url, "http://x").pathname));
  try {
    await stat(file);
    res.writeHead(200, { "content-type": types[path.extname(file)] || "application/octet-stream" });
    res.end(await readFile(file));
  } catch { res.writeHead(404); res.end(); }
}).listen(0, "127.0.0.1");
await new Promise((r) => server.once("listening", r));
const base = `http://127.0.0.1:${server.address().port}`;

const browser = await chromium.launch({ executablePath: chromePath,
  args: ["--font-render-hinting=none", "--disable-background-networking", "--disable-component-update"] });
const page = await browser.newPage({ viewport: { width: W, height: H }, deviceScaleFactor: scale });
const castRel = path.relative(here, path.resolve(castArg));
await page.goto(`${base}/web/stage.html?mode=capture&engine=${engine}&w=${W}&h=${H}&font=${font}&cast=../${castRel}`);
await page.waitForFunction(() => window.stageReady === true, null, { timeout: 30000 });
const { duration, eventTimes } = await page.evaluate(() => ({
  duration: window.stage.duration, eventTimes: window.stage.eventTimes }));

const total = Math.ceil((duration + holdEnd) * fps);
// `lossless` is for verification: verify_timing.py compares consecutive frames exactly,
// which a lossy encode's keyframe refreshes would confound.
const encoder = codec === "lossless"
  ? ["-c:v", "libx264rgb", "-crf", "0", "-preset", "ultrafast"]
  : codec === "vp9"
  ? ["-c:v", "libvpx-vp9", "-b:v", "0", "-crf", "24", "-row-mt", "1", "-pix_fmt", "yuv420p"]
  : ["-c:v", "libx264", "-preset", "slow", "-crf", "16", "-tune", "animation",
     "-profile:v", "high", "-pix_fmt", "yuv420p", "-movflags", "+faststart"];
const ffArgs = ["-y", "-loglevel", "error", "-f", "image2pipe", "-framerate", String(fps), "-i", "-",
  ...(scale !== 1 && opt("downscale") ? ["-vf", `scale=${W}:${H}:flags=lanczos`] : []),
  ...encoder, "-r", String(fps), outArg];
const ff = spawn("ffmpeg", ffArgs, { stdio: ["pipe", "inherit", "inherit"] });
const ffDone = new Promise((r, j) => ff.on("close", (c) => (c === 0 ? r() : j(new Error(`ffmpeg ${c}`)))));
const write = (buf) => new Promise((r) => (ff.stdin.write(buf) ? r() : ff.stdin.once("drain", r)));

const started = Date.now();
let frame = null, cursor = 0, shots = 0;
for (let i = 0; i < total; i++) {
  const t = Math.min(i / fps, duration);
  let changed = frame === null;
  while (cursor < eventTimes.length && eventTimes[cursor] <= t) { cursor++; changed = true; }
  if (changed) {
    await page.evaluate((tt) => window.stage.seek(tt), t);
    frame = await page.screenshot({ type: "png" });
    shots++;
  }
  await write(frame);
}
ff.stdin.end();
await ffDone;
const chromiumVersion = browser.version();
await browser.close();
server.close();

const elapsed = (Date.now() - started) / 1000;
const ffv = execFileSync("ffmpeg", ["-version"]).toString().split("\n")[0];
const receipt = {
  video: path.basename(outArg), cast: path.basename(castArg), engine, fps, frames: total,
  screenshots: shots, size: `${W * scale}x${H * scale}`, codec, encoder_args: encoder,
  cast_duration_seconds: duration, video_duration_seconds: total / fps,
  capture_seconds: elapsed, chromium: chromiumVersion, ffmpeg: ffv,
};
writeFileAtomicSync(outArg.replace(/\.[^./]+$/, "") + ".video.json", JSON.stringify(receipt, null, 2) + "\n");
console.log(JSON.stringify(receipt));
