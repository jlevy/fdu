/**
 * The terminal font, fetched at build time and verified by hash.
 *
 * Planetaire Mono Text (github.com/jlevy/planetaire, SIL OFL 1.1) is the face for both
 * outputs: the web replay and the video render the same stage page, so they share one
 * set of files. ExtraBold also answers weight 700, because terminals ask for "bold" and
 * Planetaire recommends ExtraBold as terminal bold.
 */

import { createHash } from 'node:crypto';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { CliError } from './errors.js';
import { writeFileAtomic } from './fsutil.js';
import { FONTS_DIR } from './paths.js';

export const FONT_FAMILY = 'Planetaire Mono Text';
const VERSION = 'v0.2.0';
const BASE_URL = `https://raw.githubusercontent.com/jlevy/planetaire/${VERSION}/fonts/web`;

interface Face {
  file: string;
  style: 'normal' | 'italic';
  weight: string;
  sha256: string;
}

export const FACES: readonly Face[] = [
  { file: 'PlanetaireMonoText-Regular.woff2', style: 'normal', weight: '400', sha256: 'f412b36c96e0b92dcb0d9d476e572375706d449616d341c7429af02eac7b408f' },
  { file: 'PlanetaireMonoText-Italic.woff2', style: 'italic', weight: '400', sha256: '348c6582e8ec6822a3d5d3fda89196f04320b5ed0b873ab1fb7404ede0ac7d7b' },
  { file: 'PlanetaireMonoText-ExtraBold.woff2', style: 'normal', weight: '700 800', sha256: '51da9327a03ca057ccf156636086929d784ebd7037ce02f9d3763841802b7bdc' },
  { file: 'PlanetaireMonoText-ExtraBoldItalic.woff2', style: 'italic', weight: '700 800', sha256: '98b3ab1df9df24afded11586de2235a723ac5d28a45c118db684a5961f828a00' },
];

const sha256 = (data: Uint8Array): string => createHash('sha256').update(data).digest('hex');

export function stylesheet(): string {
  const rules = FACES.map(
    (face) =>
      `@font-face {\n  font-family: '${FONT_FAMILY}';\n  font-style: ${face.style};\n  font-weight: ${face.weight};\n` +
      `  font-display: block;\n  src: url('${face.file}') format('woff2');\n}`,
  );
  return `/* ${FONT_FAMILY} ${VERSION}, fetched by \`cli-animate fonts\`. SIL OFL 1.1. */\n\n${rules.join('\n\n')}\n`;
}

/** Faces missing or not matching their pinned hash. */
export function missingFaces(dir: string = FONTS_DIR): Face[] {
  return FACES.filter((face) => {
    const path = join(dir, face.file);
    return !existsSync(path) || sha256(readFileSync(path)) !== face.sha256;
  });
}

/** Download every face that is missing or wrong, verify it, and write fonts.css. */
export async function fetchFonts(dir: string = FONTS_DIR): Promise<{ fetched: string[]; dir: string }> {
  const fetched: string[] = [];
  for (const face of missingFaces(dir)) {
    const url = `${BASE_URL}/${face.file}`;
    const response = await fetch(url);
    if (!response.ok) throw new CliError(`font download failed: ${url} (HTTP ${response.status})`);
    const data = new Uint8Array(await response.arrayBuffer());
    const actual = sha256(data);
    if (actual !== face.sha256) throw new CliError(`font ${face.file}: sha256 ${actual} does not match the pinned ${face.sha256}`);
    writeFileAtomic(join(dir, face.file), data);
    fetched.push(face.file);
  }
  writeFileAtomic(join(dir, 'fonts.css'), stylesheet());
  return { fetched, dir };
}
