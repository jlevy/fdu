// Where cargo writes build output, asked of cargo rather than assumed to be `target/`.
//
// CARGO_TARGET_DIR and `build.target-dir` both move it, and AGENTS.md asks each worktree
// for a target directory of its own, so a script that assumes `<checkout>/target` either
// fails to find the build just made or, worse, finds an older one left there and tests
// that instead (fdu-bi9a, fdu-dfbu). The Makefile asks the same question the same way.

import { spawnSync } from 'node:child_process';
import { join, resolve } from 'node:path';

/**
 * The absolute target directory cargo uses for the workspace at `root`.
 *
 * Falls back to CARGO_TARGET_DIR, then `<root>/target`, only when cargo cannot answer;
 * the build that should precede any consumer then fails first and says why.
 */
export function cargoTargetDir(root, env = process.env) {
  const result = spawnSync('cargo', ['metadata', '--format-version', '1', '--no-deps'], {
    cwd: root,
    encoding: 'utf8',
    env,
    maxBuffer: 64 * 1024 * 1024,
  });
  if (result.status === 0) {
    try {
      const directory = JSON.parse(result.stdout).target_directory;
      if (typeof directory === 'string' && directory !== '') {
        return directory;
      }
    } catch {
      // Fall through to the environment's answer.
    }
  }
  return env.CARGO_TARGET_DIR ? resolve(root, env.CARGO_TARGET_DIR) : join(root, 'target');
}

/** The debug `fdu` executable cargo builds for the workspace at `root`. */
export function debugFdu(root, env = process.env) {
  const exe = process.platform === 'win32' ? '.exe' : '';
  return join(cargoTargetDir(root, env), 'debug', `fdu${exe}`);
}
