/**
 * Delivery profiles. `render` captures once, to a lossless RGB master, and derives every
 * other delivery from the master, so `verify` and every delivery see the same frames.
 *
 * - The master keeps the screenshots exactly (RGB, lossless) and is tagged sRGB.
 * - `web` converts with an explicit BT.709 matrix to limited range and tags the stream:
 *   untagged video of 720 lines or more is read as BT.709, so a BT.601 conversion (ffmpeg's
 *   default) shifts saturated terminal colours. The H.264 level is pinned to the lowest
 *   the frame size and rate allow; left alone, `-tune animation` raises the reference
 *   frames and x264 tags levels many hardware decoders refuse. B-frames are off, since
 *   some players freeze on the first frame with them.
 * - `gif` is for READMEs: 25 fps (browsers treat GIF delays of 10 ms or less as 100 ms, so
 *   the rate must divide 100), a 256-colour palette without dithering, at 1× width.
 */

export const PROFILE_NAMES = ['web', 'master', 'gif'] as const;
export type ProfileName = (typeof PROFILE_NAMES)[number];

export interface Profile {
  name: ProfileName;
  extension: '.mp4' | '.gif';
  description: string;
  /** ffmpeg output arguments to make this profile from the master. */
  deriveArgs(info: { width: number; height: number; fps: number; scale: number }): string[];
}

/** Arguments that encode piped PNG frames into the lossless RGB master. */
export function masterArgs(fps: number): string[] {
  return [
    '-c:v', 'libx264rgb', '-preset', 'medium', '-crf', '0', '-g', String(fps * 10),
    '-color_primaries', 'bt709', '-color_trc', 'iec61966-2-1', '-colorspace', 'rgb',
    '-movflags', '+faststart',
  ];
}

// H.264 Annex A limits: [level, max macroblocks per second, max frame size in macroblocks].
const H264_LEVELS: [string, number, number][] = [
  ['3.1', 108_000, 3_600],
  ['3.2', 216_000, 5_120],
  ['4.0', 245_760, 8_192],
  ['4.1', 245_760, 8_192],
  ['4.2', 522_240, 8_704],
  ['5.0', 589_824, 22_080],
  ['5.1', 983_040, 36_864],
  ['5.2', 2_073_600, 36_864],
  ['6.0', 4_177_920, 139_264],
  ['6.1', 8_355_840, 139_264],
  ['6.2', 16_711_680, 139_264],
];

/** The lowest H.264 level whose frame-size and macroblock-rate limits admit this video. */
export function h264Level(width: number, height: number, fps: number): string {
  const frame = Math.ceil(width / 16) * Math.ceil(height / 16);
  const found = H264_LEVELS.find(([, mbps, fs]) => frame <= fs && frame * fps <= mbps);
  if (!found) throw new Error(`no H.264 level admits ${width}x${height} at ${fps} fps`);
  return found[0];
}

export const PROFILES: Record<ProfileName, Profile> = {
  web: {
    name: 'web',
    extension: '.mp4',
    description: 'H.264 High 4:2:0, CRF 16, pinned level, no B-frames, BT.709; plays everywhere',
    deriveArgs: ({ width, height, fps }) => [
      '-vf', 'scale=out_color_matrix=bt709:out_range=tv,format=yuv420p',
      '-c:v', 'libx264', '-preset', 'slow', '-crf', '16', '-tune', 'animation',
      '-profile:v', 'high', '-level:v', h264Level(width, height, fps), '-bf', '0', '-g', String(fps * 5),
      '-colorspace', 'bt709', '-color_primaries', 'bt709', '-color_trc', 'bt709', '-color_range', 'tv',
      '-an', '-movflags', '+faststart',
    ],
  },
  master: {
    name: 'master',
    extension: '.mp4',
    description: 'lossless RGB H.264, sRGB; the capture itself, used by verify',
    deriveArgs: () => ['-c', 'copy'],
  },
  gif: {
    name: 'gif',
    extension: '.gif',
    description: '25 fps, 256 colours, no dithering, 1× width; for READMEs',
    deriveArgs: ({ width, scale }) => [
      '-vf',
      `fps=25,scale=${Math.round(width / scale)}:-1:flags=lanczos,split[a][b];` +
        '[a]palettegen=max_colors=256:stats_mode=full[p];[b][p]paletteuse=dither=none:diff_mode=rectangle',
      '-loop', '0',
    ],
  },
};

export function profile(name: string): Profile {
  if (!(PROFILE_NAMES as readonly string[]).includes(name)) {
    throw new Error(`unknown profile "${name}"; choose one of ${PROFILE_NAMES.join(', ')}`);
  }
  return PROFILES[name as ProfileName];
}
