/**
 * The half of a recording that runs inside asciinema's PTY.
 *
 * It prints a prompt, types each step with the keystroke model, runs the command in the
 * same PTY (so colours, widths and progress lines behave as they do for a person), and
 * writes an in-band marker at each step boundary. Command wall times go to a sidecar
 * file that `record` folds into the receipt.
 */

import { spawn, spawnSync } from 'node:child_process';
import { setTimeout as sleep } from 'node:timers/promises';
import pc from 'picocolors';
import { markerSequence } from './cast.js';
import { writeJsonAtomic } from './fsutil.js';
import { loadScenario, type Scenario, type Step, stepLabel } from './scenario.js';
import { DEFAULT_PROFILE, enterPause, keyIntervals, Rng, type TypingProfile } from './typing.js';

/** Home the cursor, clear the screen and the scrollback (ED 2 and ED 3). */
const CLEAR_SCREEN = '\u001b[H\u001b[2J\u001b[3J';

export interface StepTiming {
  label: string;
  run: string;
  wall_seconds: number;
  exit_code: number | null;
  signal: string | null;
}

const write = (text: string): void => {
  process.stdout.write(text);
};

function commandEnv(scenario: Scenario): NodeJS.ProcessEnv {
  return {
    ...process.env,
    COLUMNS: String(scenario.terminal.cols),
    LINES: String(scenario.terminal.rows),
    ...scenario.env,
    PATH: [...scenario.resolvedPath, process.env.PATH ?? ''].join(':'),
  };
}

/** Run hidden setup commands; their output never reaches the recording. */
function runHidden(scenario: Scenario, commands: string[]): void {
  for (const command of commands) {
    const result = spawnSync(scenario.shell, ['-c', command], {
      cwd: scenario.resolvedCwd,
      env: commandEnv(scenario),
      stdio: 'ignore',
    });
    if (result.status !== 0) {
      throw new Error(`hidden command failed (exit ${String(result.status)}): ${command}`);
    }
  }
}

/** Type `text` on an absolute schedule, so sleep overhead never accumulates. */
async function typeText(text: string, rng: Rng, profile: TypingProfile): Promise<void> {
  const intervals = keyIntervals(text, rng, profile);
  let due = performance.now();
  for (const [index, ch] of [...text].entries()) {
    due += intervals[index]! * 1000;
    const wait = due - performance.now();
    if (wait > 0) await sleep(wait);
    write(ch);
  }
}

async function runStep(scenario: Scenario, step: Step & { run: string }, label: string): Promise<StepTiming> {
  const started = performance.now();
  const child = spawn(scenario.shell, ['-c', step.run], {
    cwd: scenario.resolvedCwd,
    env: commandEnv(scenario),
    stdio: 'inherit',
  });
  const timers: NodeJS.Timeout[] = [];
  for (const action of step.during) {
    timers.push(
      setTimeout(() => {
        spawn(scenario.shell, ['-c', action.run], {
          cwd: scenario.resolvedCwd,
          env: commandEnv(scenario),
          stdio: 'ignore',
        });
      }, action.at * 1000),
    );
  }
  if (step.stop_after !== undefined) {
    timers.push(setTimeout(() => child.kill('SIGINT'), step.stop_after * 1000));
  }
  const [code, signal] = await new Promise<[number | null, NodeJS.Signals | null]>((done) =>
    child.on('exit', (c, s) => done([c, s])),
  );
  for (const timer of timers) clearTimeout(timer);
  return {
    label,
    run: step.run,
    wall_seconds: Math.round(performance.now() - started) / 1000,
    exit_code: code,
    signal,
  };
}

export async function drive(scenarioFile: string, sidecar: string): Promise<void> {
  const scenario = loadScenario(scenarioFile);
  const rng = new Rng(scenario.typing.seed);
  const profile: TypingProfile = { ...DEFAULT_PROFILE, wpm: scenario.typing.wpm };
  const colors = pc.createColors(true);
  const prompt = `${colors.bold(colors.green('❯'))} `;
  const timings: StepTiming[] = [];

  runHidden(scenario, scenario.setup);
  await sleep(scenario.lead_in * 1000);
  // As in a shell, the next prompt appears the moment a command exits, so the
  // recording shows when each command finished; the hold follows the prompt.
  let promptShown = false;
  for (const [index, step] of scenario.steps.entries()) {
    runHidden(scenario, step.before);
    const label = stepLabel(step, index);
    write(markerSequence(label));
    if (step.clear) {
      write(CLEAR_SCREEN);
      promptShown = false;
    }
    if (!promptShown) write(prompt);
    if (step.comment !== undefined) {
      // Style codes are written whole around the typed text, never typed key by key.
      const [dimOpen, dimClose] = colors.dim('\u0000').split('\u0000');
      write(dimOpen ?? '');
      await typeText(step.comment, rng, profile);
      write(`${dimClose ?? ''}\r\n`);
    } else if (step.run !== undefined) {
      await typeText(step.run, rng, profile);
      await sleep(enterPause(rng) * 1000);
      write('\r\n');
      timings.push(await runStep(scenario, { ...step, run: step.run }, label));
    }
    write(prompt);
    promptShown = true;
    await sleep(step.hold * 1000);
  }
  write(markerSequence('end'));
  await sleep(scenario.tail * 1000);
  writeJsonAtomic(sidecar, timings);
}
