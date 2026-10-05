/** Locating the external programs this tool drives, and asking them their versions. */

import { spawnSync } from 'node:child_process';
import { accessSync, constants } from 'node:fs';
import { delimiter, join } from 'node:path';
import { CliError } from './errors.js';

/** The absolute path of `name` on PATH, or undefined. */
export function whichProgram(name: string, path: string = process.env.PATH ?? ''): string | undefined {
  for (const dir of path.split(delimiter)) {
    if (!dir) continue;
    const candidate = join(dir, name);
    try {
      accessSync(candidate, constants.X_OK);
      return candidate;
    } catch {
      // not here
    }
  }
  return undefined;
}

const INSTALL_HINTS: Record<string, string> = {
  asciinema: 'install asciinema 3.x (cargo install asciinema, or your package manager)',
  ffmpeg: 'install ffmpeg built with libx264',
  node: 'install Node.js 22.12 or newer',
};

export function findProgram(name: string): string {
  const found = whichProgram(name);
  if (!found) throw new CliError(`${name} not found on PATH: ${INSTALL_HINTS[name] ?? `install ${name}`}`);
  return found;
}

/** Everything a program prints (stdout then stderr) for `args`. */
export function programOutput(program: string, args: string[]): string {
  const result = spawnSync(program, args, { encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 });
  return `${result.stdout ?? ''}${result.stderr ?? ''}`;
}

/** The first line a program prints for its version flag. */
export function programVersion(program: string, args: string[]): string {
  return programOutput(program, args).split('\n')[0]!.trim();
}
