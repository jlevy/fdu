/** Locations inside the installed package. Compiled modules live in dist/src/. */

import { fileURLToPath } from 'node:url';

export const PACKAGE_ROOT = fileURLToPath(new URL('../../', import.meta.url));
export const STAGE_DIR = fileURLToPath(new URL('../../stage/', import.meta.url));
export const FONTS_DIR = fileURLToPath(new URL('../../stage/fonts/', import.meta.url));
export const SKILL_FILE = fileURLToPath(new URL('../../skill/SKILL.md', import.meta.url));
export const CLI_MAIN = fileURLToPath(new URL('./cli/main.js', import.meta.url));
