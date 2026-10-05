/**
 * Output for the command line. Results go to stdout (JSON with `--json`, else a human
 * summary); progress and diagnostics go to stderr, so stdout stays clean for pipes.
 */

import pc from 'picocolors';

export type ColorOption = 'auto' | 'always' | 'never';

export interface GlobalOptions {
  json: boolean;
  quiet: boolean;
  verbose: boolean;
  color: ColorOption;
}

export const ICONS = { SUCCESS: '✓', ERROR: '✗', WARN: '⚠' } as const;

/** Precedence: --color, then NO_COLOR, then FORCE_COLOR, then whether stderr is a TTY. */
export function shouldColorize(option: ColorOption, stream: NodeJS.WriteStream = process.stderr): boolean {
  if (option === 'always') return true;
  if (option === 'never') return false;
  if (process.env.NO_COLOR) return false;
  if (process.env.FORCE_COLOR && process.env.FORCE_COLOR !== '0') return true;
  return stream.isTTY === true;
}

export function createOutput(options: GlobalOptions) {
  const c = pc.createColors(shouldColorize(options.color));
  const colors = { success: c.green, error: c.red, warn: c.yellow, dim: c.dim, bold: c.bold, path: c.cyan };
  return {
    colors,
    /** Progress and notes: stderr, suppressed by --quiet and --json. */
    note(message: string): void {
      if (!options.quiet && !options.json) process.stderr.write(`${colors.dim(message)}\n`);
    },
    /** The command's result: JSON on stdout with --json, else the human summary. */
    result(value: unknown, human: string): void {
      if (options.json) process.stdout.write(`${JSON.stringify(value, null, 2)}\n`);
      else process.stdout.write(`${human}\n`);
    },
    /** Always shown, on stderr; `{"error": ...}` with --json. */
    error(message: string): void {
      if (options.json) process.stderr.write(`${JSON.stringify({ error: message })}\n`);
      else process.stderr.write(`${colors.error(`${ICONS.ERROR} error:`)} ${message}\n`);
    },
  };
}

export type Output = ReturnType<typeof createOutput>;
