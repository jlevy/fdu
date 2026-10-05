import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, it } from 'node:test';
import { CliError } from '../src/errors.js';
import { expandVars, parseScenario, stepLabel } from '../src/scenario.js';
import { DEFAULT_PROFILE } from '../src/typing.js';

describe('scenario', () => {
  const file = '/demos/fdu/take.yaml';

  it('fills defaults and resolves path entries against the scenario file', () => {
    const s = parseScenario('steps:\n  - run: ls\npath: [../bin]\n', file, {});
    assert.equal(s.terminal.cols, 100);
    assert.equal(s.typing.wpm, DEFAULT_PROFILE.wpm);
    assert.equal(s.terminal.font_size, 22);
    assert.deepEqual(s.resolvedPath, ['/demos/bin']);
    assert.equal(s.resolvedCwd, '/demos/fdu');
    assert.equal(s.steps[0]!.hold, 1.5);
  });

  it('expands environment variables and refuses unset ones', () => {
    assert.equal(expandVars('$HOME/x/${USER}', { HOME: '/h', USER: 'u' }), '/h/x/u');
    assert.throws(() => expandVars('$NOPE'), CliError);
  });

  it('names the field of each problem', () => {
    assert.throws(
      () => parseScenario('terminal: {cols: 5}\nsteps:\n  - run: ls\n    comment: hi\n', file, {}),
      (error: Error) => /terminal\.cols/.test(error.message) && /steps\.0/.test(error.message),
    );
  });

  it('rejects unknown keys', () => {
    assert.throws(() => parseScenario('steps: [{run: ls}]\ncolums: 3\n', file, {}), /colums/);
  });

  it('labels a step by its label, else its first words', () => {
    const s = parseScenario('steps: [{run: "fdu / --depth 1 --cache off"}, {comment: hi}]\n', file, {});
    assert.equal(stepLabel(s.steps[0]!, 0), 'fdu / --depth');
    assert.equal(stepLabel(s.steps[1]!, 1), 'step 2');
  });

  it('loads the shipped examples', () => {
    const dir = join(import.meta.dirname, '../../examples/fdu');
    for (const name of readdirSync(dir).filter((n) => n.endsWith('.yaml'))) {
      parseScenario(readFileSync(join(dir, name), 'utf8'), join(dir, name), { FDU_DEMO_TREES: '/trees', HOME: '/h' });
    }
  });
});
