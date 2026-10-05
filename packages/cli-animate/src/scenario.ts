/**
 * Scenario files: YAML describing a terminal take, validated on load.
 *
 * Paths in `path` resolve against the scenario file's directory; `cwd` and `path` expand
 * `$VAR` and `${VAR}` from the environment, and an unset variable is an error rather than
 * an empty string.
 */

import { readFileSync } from 'node:fs';
import { dirname, isAbsolute, resolve } from 'node:path';
import { parse as parseYaml } from 'yaml';
import { z } from 'zod';
import { CliError } from './errors.js';
import { DEFAULT_PROFILE } from './typing.js';

const Step = z
  .strictObject({
    label: z.string().optional(),
    run: z.string().optional(),
    comment: z.string().optional(),
    clear: z.boolean().default(false),
    before: z.array(z.string()).default([]),
    hold: z.number().nonnegative().default(1.5),
    stop_after: z.number().positive().optional(),
    during: z.array(z.strictObject({ at: z.number().nonnegative(), run: z.string() })).default([]),
  })
  .refine((step) => (step.run === undefined) !== (step.comment === undefined), {
    message: 'a step needs exactly one of `run` or `comment`',
  });

const ScenarioSchema = z.strictObject({
  title: z.string().default(''),
  terminal: z
    .strictObject({
      cols: z.number().int().min(20).max(400).default(100),
      rows: z.number().int().min(5).max(200).default(30),
      /** CSS pixels; smaller fits more columns into the same video width. */
      font_size: z.number().min(8).max(48).default(22),
    })
    .prefault({}),
  typing: z
    .strictObject({
      wpm: z.number().min(20).max(250).default(DEFAULT_PROFILE.wpm),
      seed: z.number().int().default(1),
    })
    .prefault({}),
  shell: z.string().default('/bin/bash'),
  cwd: z.string().default('.'),
  path: z.array(z.string()).default([]),
  env: z.record(z.string(), z.string()).default({}),
  setup: z.array(z.string()).default([]),
  lead_in: z.number().nonnegative().default(0.6),
  tail: z.number().nonnegative().default(2),
  steps: z.array(Step).min(1),
});

export type Step = z.infer<typeof Step>;
export type Scenario = z.infer<typeof ScenarioSchema> & {
  /** Absolute path of the scenario file. */
  file: string;
  /** Resolved working directory and PATH prefix. */
  resolvedCwd: string;
  resolvedPath: string[];
};

/** Expand `$VAR` and `${VAR}`; an unset variable is an error, not an empty string. */
export function expandVars(value: string, env: NodeJS.ProcessEnv = process.env): string {
  return value.replace(/\$(?:\{(\w+)\}|(\w+))/g, (_, braced: string | undefined, bare: string | undefined) => {
    const name = (braced ?? bare)!;
    const found = env[name];
    if (found === undefined) throw new CliError(`environment variable ${name} is not set (used in "${value}")`);
    return found;
  });
}

export function parseScenario(text: string, file: string, env: NodeJS.ProcessEnv = process.env): Scenario {
  let data: unknown;
  try {
    data = parseYaml(text);
  } catch (error) {
    throw new CliError(`${file}: not valid YAML: ${(error as Error).message}`);
  }
  const result = ScenarioSchema.safeParse(data);
  if (!result.success) {
    const problems = result.error.issues.map((issue) => `  ${issue.path.join('.') || '(top level)'}: ${issue.message}`);
    throw new CliError(`${file}: invalid scenario\n${problems.join('\n')}`);
  }
  const dir = dirname(resolve(file));
  const resolveFrom = (p: string): string => {
    const expanded = expandVars(p, env);
    return isAbsolute(expanded) ? expanded : resolve(dir, expanded);
  };
  return {
    ...result.data,
    file: resolve(file),
    resolvedCwd: resolveFrom(result.data.cwd),
    resolvedPath: result.data.path.map(resolveFrom),
  };
}

export function loadScenario(file: string, env: NodeJS.ProcessEnv = process.env): Scenario {
  let text: string;
  try {
    text = readFileSync(file, 'utf8');
  } catch {
    throw new CliError(`cannot read scenario ${file}`);
  }
  return parseScenario(text, file, env);
}

/** The label a step is known by in markers, chapters, and receipts. */
export function stepLabel(step: Step, index: number): string {
  return step.label ?? step.run?.split(/\s+/).slice(0, 3).join(' ') ?? `step ${index + 1}`;
}
