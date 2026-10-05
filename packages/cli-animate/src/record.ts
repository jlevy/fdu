/**
 * `record`: run a scenario for real inside `asciinema rec --headless` and keep the result
 * as an asciicast with chapter markers, plus a receipt of what ran and how long it took.
 */

import { spawnSync } from 'node:child_process';
import { readFileSync, rmSync } from 'node:fs';
import { extractMarkers, formatCast, markerTimes, parseCast } from './cast.js';
import type { StepTiming } from './driver.js';
import { CliError } from './errors.js';
import { temporarySibling, writeFileAtomic, writeJsonAtomic } from './fsutil.js';
import { CLI_MAIN } from './paths.js';
import { loadScenario } from './scenario.js';
import { findProgram, programVersion } from './tools.js';

export interface RecordReceipt {
  kind: 'cli-animate.record';
  cast: string;
  scenario: string;
  title: string;
  terminal: string;
  duration_seconds: number;
  typing: { model: 'cli-animate digraph model'; wpm: number; seed: number };
  command_output: 'real execution in the recording PTY, real timing';
  steps: (StepTiming & { cast_seconds: number | null })[];
  tools: Record<string, string>;
}

/** Quote one argument for a POSIX shell. */
export function shellQuote(arg: string): string {
  return /^[\w@%+=:,./-]+$/.test(arg) ? arg : `'${arg.replace(/'/g, `'\\''`)}'`;
}

export function receiptPath(castPath: string): string {
  return castPath.replace(/\.cast$/, '') + '.receipt.json';
}

export function record(scenarioFile: string, castPath: string, options: { quiet?: boolean } = {}): RecordReceipt {
  const scenario = loadScenario(scenarioFile);
  const asciinema = findProgram('asciinema');
  const version = programVersion(asciinema, ['--version']);
  if (!/\b3\.\d+/.test(version)) throw new CliError(`asciinema 3.x is required; found "${version}"`);

  const raw = temporarySibling(castPath);
  const sidecar = temporarySibling(receiptPath(castPath));
  const inner = [process.execPath, CLI_MAIN, '__drive', scenario.file, '--sidecar', sidecar].map(shellQuote).join(' ');
  try {
    const result = spawnSync(
      asciinema,
      ['rec', '--headless', '--overwrite', '--window-size', `${scenario.terminal.cols}x${scenario.terminal.rows}`, '--command', inner, raw],
      { stdio: ['ignore', options.quiet ? 'ignore' : 2, 2], env: { ...process.env, TERM: 'xterm-256color' } },
    );
    if (result.status !== 0) throw new CliError(`asciinema exited with status ${String(result.status)}`, 1);

    const recorded = parseCast(readFileSync(raw, 'utf8'));
    const events = extractMarkers(recorded.events);
    const header = {
      version: 3 as const,
      term: { cols: scenario.terminal.cols, rows: scenario.terminal.rows, type: 'xterm-256color' },
      ...(recorded.header.timestamp === undefined ? {} : { timestamp: recorded.header.timestamp }),
      title: scenario.title,
      env: { TERM: 'xterm-256color' },
    };
    writeFileAtomic(castPath, formatCast({ header, events }));

    const timings = JSON.parse(readFileSync(sidecar, 'utf8')) as StepTiming[];
    const marks = markerTimes(events);
    const receipt: RecordReceipt = {
      kind: 'cli-animate.record',
      cast: castPath,
      scenario: scenario.file,
      title: scenario.title,
      terminal: `${scenario.terminal.cols}x${scenario.terminal.rows}`,
      duration_seconds: events.at(-1)?.time ?? 0,
      typing: { model: 'cli-animate digraph model', wpm: scenario.typing.wpm, seed: scenario.typing.seed },
      command_output: 'real execution in the recording PTY, real timing',
      steps: timings.map((step) => ({ ...step, cast_seconds: marks.get(step.label) ?? null })),
      tools: { asciinema: version, node: process.version },
    };
    writeJsonAtomic(receiptPath(castPath), receipt);
    return receipt;
  } finally {
    rmSync(raw, { force: true });
    rmSync(sidecar, { force: true });
  }
}
